use cucumber::then;

use crate::state;

#[then(regex = r#"the database stored the username and the email with status "(\S+)""#)]
async fn check_stored_subscription(world: &mut state::TestWorld, status: String) {
    let subscriber = world.subscribers.clone().pop().expect("subscriber");
    check_stored_subscription_for_username_email(
        world,
        subscriber.username,
        subscriber.email,
        status,
    )
    .await
}

#[then(
    regex = r#"the database stored the username "(\S+)" and the email "(\S+)" with status "(\S+)""#
)]
async fn check_stored_subscription_for_username_email(
    world: &mut state::TestWorld,
    username: String,
    email: String,
    status: String,
) {
    if let Some(app) = &world.app {
        let subscription = &app
            .subscription
            .get_subscription_by_email(&email)
            .await
            .expect("Could not get subscription")
            .expect("No subscription available");
        assert_eq!(subscription.email.as_ref(), email);
        assert_eq!(subscription.username.as_ref(), username);
        assert_eq!(subscription.status.as_str(), status);
    }
}
