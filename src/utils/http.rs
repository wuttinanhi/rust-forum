use actix_web::{http::header, HttpRequest, HttpResponse};

pub fn create_redirect(to_url: &str) -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, to_url))
        .finish()
}

pub fn redirect_back(req: &HttpRequest) -> HttpResponse {
    let previous_url = req
        .headers()
        .get(header::REFERER)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("/");

    create_redirect(previous_url)
}
