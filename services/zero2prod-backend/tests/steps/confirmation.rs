use cucumber::{then, when};
use speculoos::prelude::*;

use crate::state;

#[then(regex = r#"the new subscriber receives an email with a confirmation link"#)]
async fn verify_confirmation_link(world: &mut state::TestWorld) {
    if let Some(app) = &world.app {
        let email_request = &app
            .email_server
            .received_requests()
            .await
            .expect("get email server received requests")[0];
        let confirmation_links = app.get_confirmation_links(email_request);
        // FIXME Other features to assert
        assert_eq!(
            confirmation_links.html.path(),
            "/api/subscriptions/confirmation"
        );
        assert!(confirmation_links
            .html
            .query()
            .unwrap()
            .starts_with("token"));
        world.subscribers[0].confirmation_link = Some(confirmation_links.html);
    }
}

#[when(regex = r#"the new subscriber retrieves the confirmation link"#)]
async fn store_confirmation_link(world: &mut state::TestWorld) {
    if let Some(app) = &world.app {
        let email_request = &app
            .email_server
            .received_requests()
            .await
            .expect("get email server received requests")[0];
        let confirmation_links = app.get_confirmation_links(email_request);
        world.subscribers[0].confirmation_link = Some(confirmation_links.html);
    }
}

#[when(regex = r#"the new subscriber confirms his subscription with the confirmation link"#)]
async fn post_confirmation_link(world: &mut state::TestWorld) {
    let confirmation_link = world.subscribers[0].confirmation_link.clone().unwrap();
    if let Some(app) = &world.app {
        let resp = app
            .api_client
            .post(confirmation_link)
            .send()
            .await
            .expect("failed to execute request");
        world.status_code = Some(resp.status());
        world.subscribers[0].status = "confirmed".to_string();
    }
}

#[then(regex = r#"the user receives two confirmation emails"#)]
async fn verify_two_confirmation_emails(world: &mut state::TestWorld) {
    if let Some(app) = &world.app {
        let emails = app
            .email_server
            .received_requests()
            .await
            .expect("get email server received requests");

        assert_that(&emails.len()).is_equal_to(2);
    }
}
