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
            http_client: Client::builder().build().unwrap(),
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
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use zero2prod_common::settings::EmailClientSettings;

    #[tokio::test]
    async fn send_email_should_fire_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let sender_email = SafeEmail().fake();
        let authorization_token = Faker.fake::<String>();
        let email_settings = EmailClientSettings {
            base_url,
            sender_email,
            authorization_token,
        };
        let email_client = EmailClient::new(email_settings)
            .await
            .expect("email client");
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let html_content: String = Paragraph(1..10).fake();
        let text_content: String = Paragraph(1..10).fake();
        // Act
        let _ = email_client
            .send_email(&subscriber_email, &subject, &html_content, &text_content)
            .await;
        // Assert
    }
}
