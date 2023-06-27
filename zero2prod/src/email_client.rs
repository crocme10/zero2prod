use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use zero2prod_common::err_context::ErrorContextExt;

use crate::domain::SubscriberEmail;
use crate::email::{Email, Error};

// use zero2prod_common::err_context::ErrorContextExt;
use zero2prod_common::settings::EmailClientSettings;

#[derive(Debug, Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: String,
}

impl EmailClient {
    pub async fn new(settings: EmailClientSettings) -> Result<EmailClient, Error> {
        let sender =
            SubscriberEmail::parse(settings.sender_email).map_err(|err| Error::Configuration {
                context: format!("Could not parse Email Client Service Sender: {err}"),
            })?;
        Ok(EmailClient {
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(settings.timeout))
                .build()
                .unwrap(),
            base_url: settings.base_url,
            sender,
            authorization_token: settings.authorization_token,
        })
    }
}

#[async_trait]
impl Email for EmailClient {
    async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), Error> {
        //TODO: Replace this with Url::join() eventually
        let url = format!("{}/email", self.base_url);

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(&url)
            .header("X-Postmark-Server-Token", &self.authorization_token)
            .json(&request_body)
            .send()
            .await
            .context("http client request to email service".to_string())?
            .error_for_status()
            .context("http client response".to_string())?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email::Email;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use speculoos::prelude::*;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use zero2prod_common::settings::EmailClientSettings;

    // Used by wiremock to ensure that our request sent
    // to the email service has all the fields required.
    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body's json
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                // Ensure mandatory fields are populated
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// Generate some random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    #[tokio::test]
    async fn send_email_should_fire_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_settings = EmailClientSettings {
            base_url,
            sender_email: SafeEmail().fake(),
            authorization_token: Faker.fake::<String>(),
            timeout: 10, // sec
        };
        let email_client = EmailClient::new(email_settings)
            .await
            .expect("email client");
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
        // Assert
        // wiremock asserts on drop
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_settings = EmailClientSettings {
            base_url,
            sender_email: SafeEmail().fake(),
            authorization_token: Faker.fake::<String>(),
            timeout: 10, // sec
        };
        let email_client = EmailClient::new(email_settings)
            .await
            .expect("email client");

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_that(&outcome).is_ok();
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_settings = EmailClientSettings {
            base_url,
            sender_email: SafeEmail().fake(),
            authorization_token: Faker.fake::<String>(),
            timeout: 10, // sec
        };
        let email_client = EmailClient::new(email_settings)
            .await
            .expect("email client");

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_that(&outcome).is_err();
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // In this test, we have the email_client with a
        // short timeout, _shorter_ than the response from the
        // mock_server, to test the response
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let email_settings = EmailClientSettings {
            base_url,
            sender_email: SafeEmail().fake(),
            authorization_token: Faker.fake::<String>(),
            timeout: 3, // sec
        };
        let email_client = EmailClient::new(email_settings)
            .await
            .expect("email client");

        // Delay 3 minutes before responding
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(6));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_that(&outcome).is_err();
    }
}
