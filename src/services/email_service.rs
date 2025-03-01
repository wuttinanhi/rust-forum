use crate::{db::WebError, models::User};

pub trait EmailService: Send + Sync {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<User, WebError>;
}
