use std::fmt::{Display, Formatter};

use actix_web::{get, web, Error, FromRequest, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db::{DbPool, WebError},
    models::Post,
    posts::repository::get_posts,
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

#[get("/test-pagination")]
pub async fn test_pagination(
    pagination_data: QueryPagination,
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl Responder> {
    let posts: Result<Vec<Post>, WebError> = web::block(move || {
        let mut conn = pool.get()?;
        let posts = get_posts(&mut conn, &pagination_data)?;

        Ok(posts)
    })
    .await?;

    match posts {
        Ok(posts) => {
            let json_value = json!(posts).to_string();

            // pagination_data.to_string()
            Ok(HttpResponse::Ok().json(json_value))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("error")),
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

pub fn handlebars_pagination_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let pagination_result = h.param(0).ok_or_else(|| {
        handlebars::RenderErrorReason::ParamNotFoundForIndex("pagination_result", 0)
    })?;

    let pagination_result_json = pagination_result.value();

    // pagination_result first param maybe null if no records
    if pagination_result_json.is_null() {
        return Ok(());
    }

    let pagination_result: HandlebarsPaginationResult =
        serde_json::from_value(pagination_result_json.clone())
            .map_err(|e| handlebars::RenderErrorReason::InvalidJsonIndex(e.to_string()))?;

    let mut output_html = String::new();

    output_html.push_str(
        "<div id=\"pagination\" class=\"mt-3\">
            <nav aria-label=\"Page navigation example\">
                <ul class=\"pagination justify-content-end\">
        ",
    );

    for page in 1..=pagination_result.total_pages {
        output_html.push_str(&format!(
            "<li class=\"page-item\"><a class=\"page-link\" href=\"?page={}&per_page={}\">{}</a></li>",
            page, pagination_result.limit, page,
        ));
    }

    output_html.push_str(&format!("
 <div class=\"px-2\">
    <form method=\"get\" action=\"\" class=\"input-group mb-3\">
        <input name=\"page\" type=\"number\" value=\"{}\" min=\"1\" class=\"form-control\" placeholder=\"Page\"
            style=\"width: 4rem;\" aria-label=\"Page\" aria-describedby=\"button-addon2\">

        <input name=\"per_page\" type=\"hidden\" value=\"{}\" style=\"display: none;\">

        <button class=\"btn btn-outline-primary\" type=\"submit\" id=\"button-addon2\">Go</button>
    </form>
</div>

<div class=\"px-2\">
    <div class=\"input-group\">
        <select class=\"form-select\" id=\"page_limit\" onchange=\"javascript:handleSelect(this)\" aria-label=\"page limit\" style=\"height: 2.5rem;\">
",
                            pagination_result.page, pagination_result.limit
    ));

    for limit in [10, 20, 50, 100].iter() {
        let should_selected = if pagination_result.limit == *limit {
            "selected"
        } else {
            ""
        };

        output_html.push_str(&format!(
            "<option value=\"?page={}&per_page={}\" {}>{}</option>",
            pagination_result.page, limit, should_selected, limit
        ));
    }

    output_html.push_str(
        "
    </select>
                        </div>
                    </div>

                </ul>
            </nav>
        </div>
        
<script type=\"text/javascript\">
    function handleSelect(elm){
        window.location = elm.value;
    }
</script>


",
    );

    out.write(&output_html)?;

    Ok(())
}
