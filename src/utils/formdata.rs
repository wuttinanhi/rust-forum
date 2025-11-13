use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    web, HttpRequest, HttpResponse,
};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
};
// --- 1. Custom Error (Largely Unchanged) ---

/// Error type for our form data extraction function.
#[derive(Debug)]
pub enum FormDataExtractError {
    /// The request's Content-Type header was missing or not
    /// `application/x-www-form-urlencoded`.
    UnsupportedContentType(String),

    /// The request body could not be read.
    BodyReadError(actix_web::Error),

    /// The request body was not valid URL-encoded data.
    ParseError(serde_urlencoded::de::Error),
}

/// Implement Display so we can have a human-readable error.
impl Display for FormDataExtractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FormDataExtractError::UnsupportedContentType(content_type) => write!(
                f,
                "Expected `application/x-www-form-urlencoded`, but got `{}`",
                content_type
            ),
            FormDataExtractError::BodyReadError(e) => {
                write!(f, "Failed to read request body: {}", e)
            }
            FormDataExtractError::ParseError(e) => write!(f, "Failed to parse form data: {}", e),
        }
    }
}

/// Implement `ResponseError` so Actix can convert this error into a response.
/// This allows our handler to return `Result<impl Responder, FormDataExtractError>`.
impl ResponseError for FormDataExtractError {
    fn status_code(&self) -> StatusCode {
        match self {
            FormDataExtractError::UnsupportedContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            FormDataExtractError::BodyReadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FormDataExtractError::ParseError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

// --- 2. Standalone Extractor Function ---

/// A standalone function to extract form data from a request.
pub async fn extract_form_data(
    req: &HttpRequest,
    payload: web::Payload,
) -> Result<HashMap<String, String>, FormDataExtractError> {
    // 1. Check Content-Type
    let headers = req.headers();
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());

    if content_type != Some("application/x-www-form-urlencoded") {
        return Err(FormDataExtractError::UnsupportedContentType(
            content_type.unwrap_or("").to_string(),
        ));
    }

    // 2. Buffer the request body
    // `to_bytes()` is provided by the `StreamExt` trait.
    let body_bytes = payload
        .to_bytes()
        .await
        .map_err(FormDataExtractError::BodyReadError)?;

    // 3. Parse the bytes
    let data: HashMap<String, String> =
        serde_urlencoded::from_bytes(&body_bytes).map_err(FormDataExtractError::ParseError)?;

    // 4. Return the map
    Ok(data)
}

// /// This handler now calls our standalone function.
// /// It takes `HttpRequest` and `web::Payload`, which Actix injects.
// async fn manual_form_handler(
//     req: HttpRequest,
//     mut payload: web::Payload,
// ) -> Result<impl Responder, FormDataExtractError> {
//     // Call our new function and propagate errors with `?`
//     let data = extract_form_data(&req, &mut payload).await?;
//
//     // Now you can use `data` just like a HashMap!
//     let email = data.get("email").map_or("Not found", |v| v.as_str());
//     let turnstile_token = data
//         .get("cf-turnstile-response")
//         .map_or("Not found", |v| v.as_str());
//
//     let response_body = format!(
//         "Extracted data:\n - email: {}\n - cf-turnstile-response: {}",
//         email, turnstile_token
//     );
//
//     // If successful, return an Ok response.
//     // If extract_form_data failed, the `?` will return the Err,
//     // which Actix will convert to an error response.
//     Ok(HttpResponse::Ok().body(response_body))
// }
//
// // --- 4. Main Function (Unchanged from before) ---
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     let addr = "127.0.0.1:3000";
//     println!("Listening on http://{}", addr);
//
//     HttpServer::new(|| App::new().route("/login", web::post().to(manual_form_handler)))
//         .bind(addr)?
//         .run()
//         .await
// }
