use cucumber::{then, when};
use speculoos::prelude::*;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::state;
use zero2prod::domain::{BodyData, Content};

#[when(regex = r#"the admin notifies subscribers of a new issue of the newsletter"#)]
async fn notify_newsletter(world: &mut state::TestWorld) {
    if let Some(app) = &world.app {
        // Reset the MockServer (includes deletes all recorded requests that could have
        // occured prior to this step).
        let _ = &app.email_server.reset().await;

        let count_confirmed = u64::try_from(world.count_confirmed_subscribers()).unwrap();

        // Arrange the behaviour of the MockServer adding a Mock:
        Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(count_confirmed)
            .mount(&app.email_server)
            .await;

        for _ in world.subscribers.clone() {
            let data = BodyData {
                title: "New Issue".to_string(),
                content: Content {
                    html: "<p>Newsletter body as HTML</p>".to_string(),
                    text: "Newsletter body as plain text".to_string(),
                },
            };
            // FIXME Note that the world.status will only be the status of the last subscriber!
            let resp = &app.send_newsletter(&data).await;
            world.status_code = Some(resp.status());
        }
    }
}

#[then(regex = r#"no newsletter are sent"#)]
async fn no_request_to_email_server(world: &mut state::TestWorld) {
    if let Some(app) = &world.app {
        let emails = &app
            .email_server
            .received_requests()
            .await
            .expect("get email server received requests");

        assert_that(&emails.len()).is_equal_to(0);
    }
}

#[then(regex = r#"the new subscriber receives a notification of a new issue of the newsletter"#)]
async fn one_request_to_email_server(world: &mut state::TestWorld) {
    if let Some(app) = &world.app {
        let emails = &app
            .email_server
            .received_requests()
            .await
            .expect("get email server received requests");

        assert_that(&emails.len()).is_equal_to(1);
    }
}
