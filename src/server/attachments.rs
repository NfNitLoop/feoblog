//! Functions for dealing w/ GET/POST/HEAD of /u/:userID/i/:signature/files/* endpoints.


use actix_web::{HttpResponse, web::{Data, Path}};

use crate::backend::{Signature, UserID};

use super::{AppData, Error};

pub(crate) async fn get_file(
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature,String)>,
) -> Result<HttpResponse, Error> {
    todo!()
}

pub(crate) async fn put_file(
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature,String)>,
) -> Result<HttpResponse, Error> {
    todo!()
}

pub(crate) async fn head_file(
    data: Data<AppData>,
    Path((user_id, signature, file_name)): Path<(UserID, Signature,String)>,
) -> Result<HttpResponse, Error> {
    todo!()
}