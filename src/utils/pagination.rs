use std::fmt::{Display, Formatter};

use actix_web::{Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct QueryPagination {
    pub page: i64,
    pub limit: i64,
}

impl Default for QueryPagination {
    fn default() -> Self {
        QueryPagination { page: 1, limit: 10 }
    }
}

impl Display for QueryPagination {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "page: {}, per_page: {}", self.page, self.limit)
    }
}

impl FromRequest for QueryPagination {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Default values
        let mut pagination = QueryPagination::default();

        let query = req.query_string();

        // Extract query parameters
        for pair in query.split('&') {
            let mut parts = pair.split('=');

            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match key {
                    "page" => {
                        if let Ok(parsed_page) = value.parse::<i64>() {
                            if parsed_page >= 1 {
                                pagination.page = parsed_page;
                            }
                        }
                    }

                    "per_page" => {
                        if let Ok(limit) = value.parse::<i64>() {
                            if (1..=100).contains(&limit) {
                                pagination.limit = limit;
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        ready(Ok(pagination))
    }
}

impl QueryPagination {
    pub fn get_offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }

    pub fn get_limit(&self) -> i64 {
        self.limit
    }
}
