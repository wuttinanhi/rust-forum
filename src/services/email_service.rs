use crate::{db::WebError, models::User};

pub trait EmailService: Send + Sync {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<User, WebError>;
}

pub struct BasedEmailService {}

impl Default for BasedEmailService {
    fn default() -> Self {
        Self::new()
    }
}

impl BasedEmailService {
    pub fn new() -> Self {
        Self {}
    }
}

impl EmailService for BasedEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<User, WebError> {
        todo!()
    }
}
