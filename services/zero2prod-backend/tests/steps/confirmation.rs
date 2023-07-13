use cucumber::{then, when};
use speculoos::prelude::*;

use crate::state;

#[then(regex = r#"the user receives an email with a confirmation link"#)]
async fn verify_confirmation_link(world: &mut state::TestWorld) {
    let email_request = &world
        .app
        .email_server
        .received_requests()
        .await
        .expect("get email server received requests")[0];
    let confirmation_links = world.app.get_confirmation_links(email_request);
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
    world.confirmation_link = Some(confirmation_links.html);
}

#[when(regex = r#"the user retrieves the confirmation link"#)]
async fn store_confirmation_link(world: &mut state::TestWorld) {
    let email_request = &world
        .app
        .email_server
        .received_requests()
        .await
        .expect("get email server received requests")[0];
    let confirmation_links = world.app.get_confirmation_links(email_request);
    world.confirmation_link = Some(confirmation_links.html);
}

#[when(regex = r#"the user confirms his subscription with the confirmation link"#)]
async fn post_confirmation_link(world: &mut state::TestWorld) {
    let resp = world
        .app
        .api_client
        .post(world.confirmation_link.clone().unwrap())
        .send()
        .await
        .expect("failed to execute request");
    world.resp = Some(resp);
}

#[then(regex = r#"the user receives two confirmation emails"#)]
async fn verify_two_confirmation_emails(world: &mut state::TestWorld) {
    let emails = &world
        .app
        .email_server
        .received_requests()
        .await
        .expect("get email server received requests");

    assert_that(&emails.len()).is_equal_to(2);
}
