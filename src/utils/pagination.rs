use actix_web::{get, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: usize,
    pub per_page: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            page: 1,
            per_page: 10,
        }
    }
}

impl FromRequest for Pagination {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Default values
        let mut pagination = Pagination::default();

        // Extract query parameters
        if let Some(query) = req.query_string().split('&').next() {
            for pair in query.split('&') {
                let mut parts = pair.split('=');

                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    match key {
                        "page" => {
                            if let Ok(parsed_page) = value.parse::<usize>() {
                                pagination.page = parsed_page;
                            }
                        }

                        "per_page" => {
                            if let Ok(parsed_per_page) = value.parse::<usize>() {
                                pagination.per_page = parsed_per_page;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }

        ready(Ok(pagination))
    }
}

#[get("/test-pagination")]
pub async fn test_pagination(pagination_data: Pagination) -> actix_web::Result<impl Responder> {
    dbg!(pagination_data);

    Ok(HttpResponse::Ok().body("test"))
}
