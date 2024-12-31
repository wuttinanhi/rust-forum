use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserLoginFormData {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserRegisterFormData {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserChangePasswordFormData {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Deserialize, Debug)]
pub struct UserUpdateFormData {
    pub new_name: String,
}
