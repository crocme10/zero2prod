use cucumber::then;

use crate::state;

#[then(
    regex = r#"the database stored the username "(\S+)" and the email "(\S+)" with status "(\S+)""#
)]
async fn query_subscription_status(
    world: &mut state::TestWorld,
    username: String,
    email: String,
    status: String,
) {
    let subscription = world
        .app
        .storage
        .get_subscription_by_email(&email)
        .await
        .expect("get subscription");
    let subscription = subscription.expect("subscription");
    assert_eq!(subscription.email.as_ref(), email);
    assert_eq!(subscription.username.as_ref(), username);
    assert_eq!(subscription.status.as_str(), status);
}
