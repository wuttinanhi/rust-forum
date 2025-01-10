use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatePostFormData {
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
pub struct UpdatePostFormData {
    pub title: String,
    pub body: String,
}
