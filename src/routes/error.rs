use crate::utils::flash::{set_flash_message, FLASH_ERROR};
use actix_session::SessionExt;
use actix_web::{
    dev::ServiceResponse,
    http::header::{LOCATION, REFERER},
    middleware::ErrorHandlerResponse, HttpResponse, Result,
};

pub fn fallback_error_handler<B: actix_web::body::MessageBody + 'static>(
    res: ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let req = res.request().clone();
    let session = &req.get_session();
    let response_error = res.response().error();

    dbg!(&response_error);

    if let Some(error) = response_error {
        let error_message = error.to_string();
        dbg!(&error_message);

        set_flash_message(session, FLASH_ERROR, &error_message)?;

        // Create redirect response
        let referer = req
            .headers()
            .get(REFERER)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("/");

        let mut response = HttpResponse::Found();
        response.append_header((LOCATION, referer));

        return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req,
            response.finish().map_into_right_body(),
        )));

        // set_flash_message(&session, FLASH_ERROR, &error_message)?;

        // // Create redirect response
        // let referer = req
        //     .headers()
        //     .get(REFERER)
        //     .and_then(|h| h.to_str().ok())
        //     .unwrap_or("/");

        // let mut response = HttpResponse::Found();
        // response.append_header((LOCATION, referer));

        // return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        //     req,
        //     response
        //         .finish()
        //         .map_into_boxed_body()
        //         .map_into_right_body(),
        // )));

        // Redirect to the previous page or a fallback
        // let referer = req.headers().get(REFERER).and_then(|h| h.to_str().ok());
        // let redirect_url = referer.unwrap_or("/");
        // res.response_mut().headers_mut().insert(
        //     LOCATION,
        //     redirect_url.parse().expect("Invalid redirect URL"),
        // );

        // res.response_mut()
        //     .headers_mut()
        //     .insert(header::LOCATION, HeaderValue::from_static(""));
        // *res.response_mut().status_mut() = actix_web::http::StatusCode::FOUND;
        // let mut response = HttpResponse::Found();
        // response.append_header((LOCATION, ""));

        // let response_finish = response.finish().map_into_right_body();

        // return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        //     req,
        //     response_finish,
        // )));
    }

    let boxed = res.map_into_boxed_body().map_into_right_body();
    Ok(ErrorHandlerResponse::Response(boxed))

    // let boxed = res.map_into_boxed_body().map_into_right_body();
    // return Ok(ErrorHandlerResponse::Response(boxed));
}
