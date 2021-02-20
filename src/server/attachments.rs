//! Functions for dealing w/ GET/POST/HEAD of /u/:userID/i/:signature/files/* endpoints.


use std::io::{BufReader, BufWriter, Seek, SeekFrom};

use actix_web::{HttpRequest, HttpResponse, Responder, client::HttpError, dev::{SizedStream}, http::{HeaderName, HeaderValue, header::{self, CONTENT_LENGTH}}, web::{Data, Path, Payload}};
use failure::ResultExt;
use futures::{AsyncSeekExt, AsyncWriteExt, StreamExt};
use mime_guess::mime;
use sodiumoxide::crypto::hash::sha512;
use tempfile::tempfile;
use log::{debug};

use crate::backend::{SHA512, Signature, UserID};

use super::{AppData, Error, PLAINTEXT, file_not_found};

pub(crate) async fn get_file(
    req: HttpRequest,
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature, String)>,
) -> Result<HttpResponse, Error> {
    let backend = data.backend_factory.open().compat()?;

    let contents = backend.get_contents(user_id, signature, file_name.as_str()).compat()?;
    let contents = match contents {
        None => return Ok(
            file_not_found("File not found").await
            .respond_to(&req).await?
        ),
        Some(c) => c,
    };

    let mut mime_type = format!("{}", mime_guess::from_path(&file_name).first_or_octet_stream());

    // FeoBlog is not meant to be a general web server.
    // Plus, since the client also runs in the browser, these could be a security risk:
    if mime_type.contains("html") || mime_type.contains("javascript") {
        mime_type = mime::TEXT_PLAIN.to_string();
    }
    let response = HttpResponse::Ok()
        .content_type(mime_type)

        // no_chunking() sets the content-length, so this is redundant:
        // .set_header(CONTENT_LENGTH, contents.size)
        .no_chunking(contents.size)
        .streaming(contents.stream);

        // Note: Above we could've used a SizedStream, but it explicitly requires an actix::Error, not an Into<actix::Error>,
        // as streaming does. But actix::Error is not Send, which is required by blocking::Unblock.

    Ok(response)
}

pub(crate) async fn put_file(
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature, String)>,
    req: HttpRequest,
    mut body: Payload,
) -> Result<HttpResponse, Error> {
    let backend = data.backend_factory.open().compat()?;

    let metadata = backend.get_attachment_meta(&user_id, &signature, &file_name).compat()?;

    let metadata = match metadata {
        Some(d) => d,
        None => {
            // If we don't yet have the metadata for a file (provided in its Item), then you can't upload yet.
            return Ok(
                HttpResponse::Forbidden()
                .content_type(PLAINTEXT)
                .body("No such attachment for this Item, or no such Item.")
            );
        }
    };
    
    if metadata.exists {
        return Ok(
            HttpResponse::Accepted()
            .content_type(PLAINTEXT)
            .body("Attachment already exists")
        );
    }

    if metadata.quota_exceeded {
        return Ok(
            HttpResponse::Forbidden()
            .content_type(PLAINTEXT)
            .body("Uploading this attachment would voilate the users's quota.")
        );
    }

    let length = match req.headers().get("content-length") {
        Some(length) => length,
        None => {
            return Ok(
                HttpResponse::LengthRequired()
                .content_type(PLAINTEXT)
                .body("Must include length header.".to_string())
                // ... so that we can reject things that are the wrong size outright.
            );
        }
    };

    let size = length
        .to_str().context("Parsing http Length header").compat()?
        .parse::<u64>().context("Parsing http Length header").compat()?;

    if metadata.size != size {
        return Ok(
            HttpResponse::BadRequest()
            .content_type(PLAINTEXT)
            .body(format!("File should be {} bytes but received {}", metadata.size, size))
        ); 
    }

    // Collect the file bytes into a temp file so that we're not using the backend while we wait for the upload:

    // Drop our pooled connection while we wait for bytes, which could be slow:
    drop(backend);

    let file = tempfile().context("Error opening temp file").compat()?;

    // Unblock's default buffer for I/O is *8MiB*!?  32k at a time seems fine.
    let mut file = blocking::Unblock::with_capacity(32 * 1024, file);

    let mut written: u64 = 0;
    let mut hasher = sha512::State::new();

    debug!("Receiving and hashing file: {}", &file_name);
    
    while let Some(chunk) = body.next().await {
        let chunk = chunk.context("Error parsing chunk").compat()?;

        file.write_all(&chunk).await?;
        written += chunk.len() as u64;
        hasher.update(&chunk);
        if written > size { break; }
    }

    if written != size {
        return Ok(
            HttpResponse::BadRequest()
            .body(format!(
                "Expected {} bytes but received {}",
                size,
                written
            ))
        );
    }

    let hash = SHA512::from_digest(hasher.finalize());
    if hash != metadata.hash {
        return Ok(
            HttpResponse::BadRequest()
            .content_type(PLAINTEXT)
            .body(format!(
                "Invalid data. Expected {}", metadata.hash
            ))
        );
    }
    debug!("Received correct hash: {}", &hash);

    // Got here, file upload is good.
    // Copy the file into the appropriate BLOB.
    // Need to start from the beginning:
    file.flush().await?;

    // Just grab the inner file to simplify types for the Backend:
    let mut file = file.into_inner().await;

    blocking::unblock(move || -> Result<(), failure::Error> {
        file.seek(SeekFrom::Start(0))?;
        let backend = data.backend_factory.open().compat()?;
        backend.save_attachment(metadata.size, &metadata.hash, &mut file)?;
        Ok(())
    }).await.compat()?;

    return Ok(
        HttpResponse::Created()
        .body("")
    );
}

pub(crate) async fn head_file(
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let backend = data.backend_factory.open().compat()?;

    let metadata = backend.get_attachment_meta(&user_id, &signature, &file_name).compat()?;
    let metadata = backend.get_attachment_meta(&user_id, &signature, &file_name).compat()?;

    let metadata = match metadata {
        Some(d) => d,
        None => {
            // Note: a 404 doesn't necessarily mean that you can upload.
            // The item doesn't yet exist, you can't upload a file here.
            return Ok(
                HttpResponse::NotFound().finish()
            );
        }
    };
    
    if metadata.exists {
        // I'd love to set a content-length here, but apparently Actix just won't let you for a HEAD?
        // See: https://github.com/actix/actix-web/issues/1439
        let response = HttpResponse::Ok().finish();
        return Ok(response);
    }

    let exceeded: u64 = if metadata.quota_exceeded { 1 } else { 0 };

    let response = HttpResponse::NotFound()
        // You can treat a 0 here as a "Yes, we would like this file".
        // i.e.: It's not a plain 404. We have metadata for it, and uploading it wouldn't exceed quota.
        .set_header("X-FB-Quota-Exceeded", exceeded)
        .finish();

    Ok(response)
}