use actix_web::{get, web, HttpResponse, Responder};
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

#[get("/test-pagination")]
pub async fn test_pagination(
    pagination_data: web::Query<Pagination>,
) -> actix_web::Result<impl Responder> {
    dbg!(pagination_data.unwrap_or_default());

    Ok(HttpResponse::Ok().body("test"))
}
