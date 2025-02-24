use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PostFormData {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Title must be at least 1 character and max 100"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 5000,
        message = "Body must be at least 1 character and max 5000"
    ))]
    pub body: String,
}
