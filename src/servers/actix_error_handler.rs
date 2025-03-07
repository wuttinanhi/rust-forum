use crate::utils::flash::{set_flash_message, FLASH_ERROR};
use actix_session::SessionExt;
use actix_web::error::UrlencodedError;
use actix_web::{
    dev::ServiceResponse,
    http::{
        header::{LOCATION, REFERER},
        StatusCode,
    },
    middleware::ErrorHandlerResponse,
    Result,
};
use actix_web::{HttpRequest, HttpResponse};

pub fn actix_error_handler<B>(service_res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let (req, mut res) = service_res.into_parts();
    let response_error = res.error();
    let session = req.get_session();

    if let Some(response_error) = response_error {
        dbg!(&response_error);

        // Create redirect response
        let referer = req
            .headers()
            .get(REFERER)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/");

        // dbg!(&referer);

        let response_error_message = if response_error.as_error::<UrlencodedError>().is_some() {
            "url encode error".to_string()
        } else {
            response_error.to_string()
        };

        set_flash_message(&session, FLASH_ERROR, &response_error_message)?;

        *res.status_mut() = StatusCode::FOUND;
        res.headers_mut().insert(LOCATION, referer.parse().unwrap());

        return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req,
            res.map_into_left_body(),
        )));
    }

    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        req,
        res.map_into_left_body(),
    )))
}

// apply large file upload fix from https://github.com/actix/actix-web/issues/3152#issuecomment-2539018905
pub fn handle_multipart_error(
    err: actix_multipart::MultipartError,
    _req: &HttpRequest,
) -> actix_web::Error {
    let response = HttpResponse::BadRequest().force_close().finish();
    actix_web::error::InternalError::from_response(err, response).into()
}
