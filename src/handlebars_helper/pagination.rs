use crate::utils::pagination::QueryPagination;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde::{Deserialize, Serialize};
use serde_json::json;

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

    // dbg!(&json_value);

    let output_html = hb_registry.render("pagination", &json_value)?;

    output.write(&output_html)?;

    Ok(())
}
