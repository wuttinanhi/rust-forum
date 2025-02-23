use std::borrow::BorrowMut;

use crate::utils::flash::{set_flash_message, FLASH_ERROR};
use actix_multipart::MultipartError;
use actix_session::SessionExt;
use actix_web::{
    dev::ServiceResponse,
    http::{
        header::{self, LOCATION, REFERER},
        StatusCode,
    },
    middleware::ErrorHandlerResponse,
    HttpMessage, HttpResponse, Result,
};

pub fn error_handler<B>(mut service_res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let (mut req, mut res) = service_res.into_parts();
    let response_error = res.error();
    let mut session = req.get_session();

    dbg!(&response_error);

    if let Some(response_error) = response_error {
        // Create redirect response
        let referer = req
            .headers()
            .get(REFERER)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/");

        dbg!(&referer);

        let session_result =
            set_flash_message(&mut session, FLASH_ERROR, &response_error.to_string())?;

        dbg!(&session_result);

        // let new_res = HttpResponse::Found()
        //     .append_header((LOCATION, referer))
        //     .finish();

        *res.status_mut() = StatusCode::FOUND;
        res.headers_mut().insert(LOCATION, referer.parse().unwrap());

        // let modified_headers = res.headers();

        // dbg!(&modified_headers);

        // let mut response = HttpResponse::Found();
        // response.append_header((LOCATION, referer));
        // let response_finish = response.finish();

        // return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        //     req,
        //     new_res.map_into_right_body(),
        // )));

        return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req,
            res.map_into_left_body(),
        )));
    }

    // Reassemble ServiceResponse from the modified parts
    let new_service_res = ServiceResponse::new(req, res);

    // body is unchanged, map to "left" slot
    Ok(ErrorHandlerResponse::Response(
        new_service_res.map_into_left_body(),
    ))
}
