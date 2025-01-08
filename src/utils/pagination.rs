use actix_web::{get, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db::{DbError, DbPool},
    models::Post,
    posts::repository::list_posts,
};

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

impl ToString for QueryPagination {
    fn to_string(&self) -> String {
        format!("page: {}, per_page: {}", self.page, self.limit)
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
                            if limit >= 1 && limit <= 100 {
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

    match posts {
        Ok(posts) => {
            let json_value = json!(posts).to_string();

            // pagination_data.to_string()
            return Ok(HttpResponse::Ok().json(json_value));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().body("error"));
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HandlebarsPaginationResult {
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

pub fn build_handlebars_pagination_result(
    total_entity: i64,
    page: i64,
    limit: i64,
) -> HandlebarsPaginationResult {
    HandlebarsPaginationResult {
        limit,
        page,
        total_pages: (total_entity as f64 / limit as f64).ceil() as i64,
    }
}

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

pub fn handlebars_pagination_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let pagination_result = h
        .param(0)
        .ok_or_else(|| handlebars::RenderError::new("Param 0 is required for pagination_helper"))?;

    let pagination_result: HandlebarsPaginationResult =
        serde_json::from_value(pagination_result.value().clone())
            .map_err(|e| handlebars::RenderErrorReason::InvalidJsonIndex(e.to_string()))?;

    let mut output_html = String::new();

    output_html.push_str(&format!(
        " <div id=\"pagination\" class=\"mt-3\">
            <nav aria-label=\"Page navigation example\">
                <ul class=\"pagination justify-content-end\">"
    ));

    for page in 1..=pagination_result.total_pages {
        output_html.push_str(&format!(
            "<li class=\"page-item\"><a class=\"page-link\" href=\"?page={}&per_page={}\">{}</a></li>",
            page, pagination_result.limit, page,
        ));
    }

    output_html.push_str(&format!(
        " <div class=\"px-2\">
                        <div class=\"input-group mb-3\">
                            <input name=\"goto_page\" type=\"number\" value=\"{}\" class=\"form-control\" placeholder=\"Page\" aria-label=\"Page\"
                                aria-describedby=\"button-addon2\" style=\"width: 5em;\">

                            <button class=\"btn btn-outline-primary\" type=\"button\" id=\"button-addon2\">Go</button>
                        </div>
                    </div>

                    <div class=\"px-2\">
                        <div class=\"input-group\">
                            <select class=\"form-select\" id=\"page_limit\" aria-label=\"page limit\" style=\"height: 2.5rem;\">",
                            pagination_result.page
    ));

    for limit in [10, 20, 50, 100].iter() {
        let should_selected = if pagination_result.limit == *limit {
            "selected"
        } else {
            ""
        };

        output_html.push_str(&format!(
            "<option value=\"{}\" {}>{}</option>",
            limit, should_selected, limit
        ));
    }

    output_html.push_str(
        "</select>
                        </div>
                    </div>

                </ul>
            </nav>
        </div>",
    );

    out.write(&output_html)?;

    Ok(())
}
