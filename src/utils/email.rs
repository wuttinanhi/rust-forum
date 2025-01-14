use std::time::Duration;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message, SmtpTransport, Transport};

#[allow(non_snake_case)]
pub fn send_email(
    to: &str,
    subject: &str,
    body: &str,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let SMTP_HOST = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let SMTP_EMAIL = std::env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set");
    let SMPTP_PASSWORD = std::env::var("SMPTP_PASSWORD").expect("SMPTP_PASSWORD must be set");

    let email = Message::builder()
        .from(SMTP_EMAIL.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(String::from(body))
        .unwrap();

    let creds = Credentials::new(SMTP_EMAIL.to_owned(), SMPTP_PASSWORD.to_owned());

    let tls_parameters = TlsParameters::builder(SMTP_HOST.to_string())
        .build()
        .unwrap();

    let mailer = SmtpTransport::relay(&SMTP_HOST)
        .unwrap()
        .credentials(creds)
        .tls(Tls::Required(tls_parameters))
        .port(587)
        .timeout(Some(Duration::from_secs(60)))
        .build();

    // Send the email
    mailer.send(&email)
}
