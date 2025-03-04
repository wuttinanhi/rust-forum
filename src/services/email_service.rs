use std::fmt::Display;
use std::time::Duration;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message, SmtpTransport, Transport};

pub trait EmailService: Send + Sync {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailServiceError>;
}

pub enum EmailServiceError {
    ErrorEmail(String),
}

impl Display for EmailServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            EmailServiceError::ErrorEmail(reason) => write!(f, "{}", reason),
        }
    }
}

pub struct BasedEmailService {}

impl BasedEmailService {
    pub fn new() -> Self {
        Self {}
    }
}

impl EmailService for BasedEmailService {
    fn send_email(&self, to: &str, header: &str, body: &str) -> Result<(), EmailServiceError> {
        let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_email = std::env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set");
        let smptp_password = std::env::var("SMPTP_PASSWORD").expect("SMPTP_PASSWORD must be set");

        let email = Message::builder()
            .from(smtp_email.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(header)
            .header(ContentType::TEXT_PLAIN)
            .body(String::from(body))
            .unwrap();

        let creds = Credentials::new(smtp_email.to_owned(), smptp_password.to_owned());

        let tls_parameters = TlsParameters::builder(smtp_host.to_string())
            .build()
            .unwrap();

        let mailer = SmtpTransport::relay(&smtp_host)
            .unwrap()
            .credentials(creds)
            .tls(Tls::Required(tls_parameters))
            .port(587)
            .timeout(Some(Duration::from_secs(60)))
            .build();

        // Send the email
        let send_email_result = mailer.send(&email);

        match send_email_result {
            Ok(_) => Ok(()),
            Err(e) => Err(EmailServiceError::ErrorEmail(e.to_string())),
        }
    }
}
