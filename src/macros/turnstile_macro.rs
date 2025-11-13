/// Validates the `cf_turnstile_response` field from an *already extracted* form struct.
///
/// This macro assumes you have used `web::Form<YourStruct>` to get a `form` variable.
///
/// On success, this macro does nothing and code execution continues.
///
/// On failure:
/// 1.  **Flash message error:** Propagates the error via `?`.
/// 2.  **Turnstile validation error:** Sets a flash message and returns an `Ok(redirect)`
///     response immediately.
///
/// # Arguments
///
/// * `$form_struct`: The variable holding the extracted form (e.g., `form`).
/// * `$session`: The `Session` object (e.g., `session`).
/// * `$req`: The `HttpRequest` object (e.g., `req`).
///
/// Add this field to your form struct:
///
/// #[validate(length(
///         min = 1,
///         message = "cf-turnstile-response must not be empty if provided"
///     ))]
/// #[serde(rename = "cf-turnstile-response")]
/// pub cf_turnstile_response: Option<String>,
#[macro_export]
macro_rules! validate_turnstile_field {
    ($form_struct:expr, $session:expr, $req:expr) => {
        // We wrap the logic in a block to scope the internal `let` bindings.
        // This macro will expand to a series of statements.
        {
            // Line 1: Get the turnstile response from the struct.
            // This handles `Option<String>` by unwrapping or providing an empty string.
            let cf_turnstile_response = $form_struct
                .cf_turnstile_response
                .to_owned()
                .unwrap_or("".to_string());

            // Line 2: Validate the token
            let turnstile_result =
                crate::utils::turnstile::validate_turnstile_wrapper(&cf_turnstile_response).await;

            // Line 3: Handle the error
            if let Err(turnstile_error) = turnstile_result {
                // Line 4: Set flash message.
                // This `?` will propagate if `set_flash_message` fails.
                crate::utils::flash::set_flash_message(
                    &$session,
                    crate::utils::flash::FLASH_ERROR, // Using the constant from your snippet
                    &turnstile_error.message,
                )?;

                // Line 5: Return a redirect response.
                // This exits the calling function.
                return Ok(crate::utils::http::redirect_back(&$req));
            }
        }
    };
}
