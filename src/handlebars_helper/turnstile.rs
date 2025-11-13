use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TurnstileHandlebarsRenderValueContext {
    pub turnstile_site_key: String,
}

pub fn handlebars_turnstile_helper(
    _h: &Helper,
    hb_registry: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    output: &mut dyn Output,
) -> HelperResult {
    let turnstile_site_key = std::env::var("CLOUDFLARE_TURNSTILE_SITE_KEY");

    if let Ok(turnstile_site_key) = turnstile_site_key {
        let hb_render_value_context = TurnstileHandlebarsRenderValueContext { turnstile_site_key };
        let hb_render_value_json = json!({ "turnstile": hb_render_value_context });

        // dbg!(&json_render_value);

        let output_html = hb_registry.render("utils/turnstile", &hb_render_value_json)?;

        output.write(&output_html)?;
    }

    Ok(())
}
