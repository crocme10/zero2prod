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
        .get_subscription_by_username(&username)
        .await
        .expect("get subscription");
    let subscription = subscription.expect("subscription");
    assert_eq!(subscription.email, email);
    assert_eq!(subscription.username, username);
    assert_eq!(subscription.status, status);
}
