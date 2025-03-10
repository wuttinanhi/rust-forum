use std::fmt::{Display, Formatter};

use actix_web::{Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HandlebarsPaginationResult {
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

pub fn build_handlebars_pagination_result(
    total_entity: i64,
    pagination: &QueryPagination,
) -> HandlebarsPaginationResult {
    HandlebarsPaginationResult {
        page: pagination.page,
        limit: pagination.limit,
        total_pages: (total_entity as f64 / pagination.limit as f64).ceil() as i64,
    }
}

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HandleBarsPaginationPerPage {
    pub option_tag_attr: String,
    pub limit: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HandlebarsPaginationRenderContext {
    pub page: i64,
    pub per_page: i64,

    pub pages: Vec<i64>,
    pub per_pages: Vec<HandleBarsPaginationPerPage>,
}

pub fn handlebars_pagination_helper(
    h: &Helper,
    hb_registry: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    output: &mut dyn Output,
) -> HelperResult {
    let pagination_result = h.param(0).ok_or_else(|| {
        handlebars::RenderErrorReason::ParamNotFoundForIndex("pagination_result", 0)
    })?;

    let pagination_result_json = pagination_result.value();

    // pagination_result first param maybe null if no records then just exit
    if pagination_result_json.is_null() {
        return Ok(());
    }

    let pagination_result: HandlebarsPaginationResult =
        serde_json::from_value(pagination_result_json.clone())
            .map_err(|e| handlebars::RenderErrorReason::InvalidJsonIndex(e.to_string()))?;

    let pages: Vec<i64> = (1..=pagination_result.total_pages).collect();

    let mut per_pages: Vec<HandleBarsPaginationPerPage> = vec![];
    for limit in [10, 20, 50, 100].into_iter() {
        let option_select_attr = if pagination_result.limit == limit {
            "selected"
        } else {
            ""
        };

        per_pages.push(HandleBarsPaginationPerPage {
            option_tag_attr: option_select_attr.to_string(),
            limit,
        });
    }

    let hb_pagination_render_context = HandlebarsPaginationRenderContext {
        page: pagination_result.page,
        per_page: pagination_result.limit,
        pages,
        per_pages,
    };
    let json_value = json!({ "pagination": hb_pagination_render_context });

    dbg!(&json_value);

    let output_html = hb_registry.render("pagination", &json_value)?;

    output.write(&output_html)?;

    Ok(())
}
