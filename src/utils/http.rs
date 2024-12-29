use actix_web::{http::header, HttpResponse};

pub fn create_redirect(to_url: &str) -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, to_url))
        .finish()
}
