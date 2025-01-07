use actix_web::{get, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use serde::Deserialize;
use serde_json::json;

use crate::{
    db::{DbError, DbPool},
    models::Post,
    posts::repository::list_posts,
};

#[derive(Debug, Deserialize)]
pub struct QueryPagination {
    pub page: i64,
    pub per_page: i64,
}

impl Default for QueryPagination {
    fn default() -> Self {
        QueryPagination {
            page: 1,
            per_page: 10,
        }
    }
}

impl ToString for QueryPagination {
    fn to_string(&self) -> String {
        format!("page: {}, per_page: {}", self.page, self.per_page)
    }
}

impl FromRequest for QueryPagination {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Default values
        let mut pagination = QueryPagination::default();

        // Extract query parameters
        if let Some(query) = req.query_string().split('&').next() {
            for pair in query.split('&') {
                let mut parts = pair.split('=');

                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    match key {
                        "page" => {
                            if let Ok(parsed_page) = value.parse::<i64>() {
                                pagination.page = parsed_page;
                            }
                        }

                        "per_page" => {
                            if let Ok(parsed_per_page) = value.parse::<i64>() {
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
pub async fn test_pagination(
    pagination_data: QueryPagination,
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl Responder> {
    dbg!(&pagination_data);

    let posts: Result<Vec<Post>, DbError> = web::block(move || {
        let mut conn = pool.get()?;
        let posts = list_posts(&mut conn, &pagination_data)?;

        Ok(posts)
    })
    .await?;

    // dbg!(&posts);

    match posts {
        Ok(posts) => {
            let json_value = json!(posts).to_string();

            // pagination_data.to_string()
            return Ok(HttpResponse::Ok().body(json_value));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("error"));
        }
    }
}
