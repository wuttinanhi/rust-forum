use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateCommentFormData {
    #[validate(range(min = 1, max = 1000000000, message = "Invalid post ID"))]
    pub post_id: i32,

    #[validate(length(
        min = 1,
        max = 5000,
        message = "Comment must be at least 1 character and max 5000"
    ))]
    pub body: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateCommentFormData {
    #[validate(length(
        min = 1,
        max = 5000,
        message = "Comment must be at least 1 character and max 5000"
    ))]
    pub body: String,
}
