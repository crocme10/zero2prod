use cucumber::{then, when};
use speculoos::prelude::*;
use wiremock::{matchers::any, Mock, ResponseTemplate};

use crate::state;
use zero2prod::domain::subscriber_email::SubscriberEmail;
use zero2prod::email_service::Email;

#[when(regex = r#"the admin notifies subscribers of a new issue of the newsletter"#)]
async fn notify_newsletter(world: &mut state::TestWorld) {
    // Reset the MockServer (includes deletes all recorded requests that could have
    // occured prior to this step).
    let _ = &world.app.email_server.reset().await;

    // Arrange the behaviour of the MockServer adding a Mock:
    // we expect that no request is fired
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&world.app.email_server)
        .await;

    let newsletter = Email {
        to: SubscriberEmail::parse("bob@acme.com").unwrap(),
        subject: "Issue 42".to_string(),
        html_content: "<p>Newsletter body as HTML</p>".to_string(),
        text_content: "Newsletter body as plain text".to_string(),
    };

    let resp = world.app.send_newsletter(&newsletter).await;
    world.resp = Some(resp);
}

#[then(regex = r#"no newsletter are sent"#)]
async fn no_request_to_email_server(world: &mut state::TestWorld) {
    let emails = &world
        .app
        .email_server
        .received_requests()
        .await
        .expect("get email server received requests");

    assert_that(&emails.len()).is_equal_to(0);
}
