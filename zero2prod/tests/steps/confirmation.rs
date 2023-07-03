use cucumber::then;

use crate::state;

#[then(regex = r#"the user receives an email with a confirmation link"#)]
async fn confirm(world: &mut state::TestWorld) {
    let email_request = &world
        .app
        .email_server
        .received_requests()
        .await
        .expect("get email server received requests")[0];
    let confirmation_links = world
        .app
        .get_confirmation_links(email_request);
    // FIXME Other features to assert
    assert_eq!(confirmation_links.html.path(), "/subscriptions/confirmation");
    assert!(confirmation_links.html.query().unwrap().starts_with("subscription_token"));
}
