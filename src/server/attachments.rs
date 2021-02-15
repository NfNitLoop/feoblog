//! Functions for dealing w/ GET/POST/HEAD of /u/:userID/i/:signature/files/* endpoints.


use actix_web::{HttpRequest, HttpResponse, Responder, dev::SizedStream, http::header::CONTENT_LENGTH, web::{Data, Path}};
use failure::ResultExt;

use crate::backend::{Signature, UserID};

use super::{AppData, Error, file_not_found};

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

    let mime_type = format!("{}", mime_guess::from_path(&file_name).first_or_octet_stream());
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
) -> Result<HttpResponse, Error> {
    let backend = data.backend_factory.open().compat()?;

    todo!()
}

pub(crate) async fn head_file(
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature, String)>,
) -> Result<HttpResponse, Error> {
    let backend = data.backend_factory.open().compat()?;

    todo!()
}