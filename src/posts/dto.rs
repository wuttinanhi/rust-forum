use serde::Deserialize;


#[derive(Deserialize)]
pub struct CreatePostFormData {
    pub title: String,
    pub body: String,
}
