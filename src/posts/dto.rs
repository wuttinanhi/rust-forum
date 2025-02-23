use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct PostFormData {
    #[validate(length(min = 1, max = 100, message = "Title cannot be empty"))]
    pub title: String,

    #[validate(length(min = 1, max = 5000, message = "Body cannot be empty"))]
    pub body: String,
}
