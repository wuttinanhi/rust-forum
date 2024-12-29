use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateCommentFormData {
    pub post_id: i32,
    pub body: String,
}
