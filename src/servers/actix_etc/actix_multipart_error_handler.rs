use actix_web::{HttpRequest, HttpResponse};

// apply large file upload fix from https://github.com/actix/actix-web/issues/3152#issuecomment-2539018905
pub fn actix_multipart_error_handler(
    err: actix_multipart::MultipartError,
    _req: &HttpRequest,
) -> actix_web::Error {
    let response = HttpResponse::BadRequest().force_close().finish();
    actix_web::error::InternalError::from_response(err, response).into()
}
