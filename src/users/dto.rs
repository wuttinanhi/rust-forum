use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserLoginFormData {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserRegisterFormData {
    pub name: String,
    pub email: String,
    pub password: String,
}
