use cucumber::when;
use reqwest::StatusCode;

use crate::state;

// Note: I prepend the word 'subscriber' with 'valid' (or 'invalid'), so that I can set
// expectations on the email server before knowing if the registration is valid or not.
#[when(regex = r#"a valid subscriber with username "(\S*)" and email "(\S*)" registers"#)]
async fn subscribes_valid(world: &mut state::TestWorld, username: String, email: String) {
    let state::SubscriptionResponse { status, subscriber } =
        world.app.register_subscriber(username, email, 1).await;
    world.status = Some(status);
    if status == StatusCode::OK {
        world.subscribers.push(subscriber);
    }
}

#[when(regex = r#"an invalid subscriber with username "(\S*)" and email "(\S*)" registers"#)]
async fn subscribes_invalid(world: &mut state::TestWorld, username: String, email: String) {
    let state::SubscriptionResponse { status, subscriber } =
        world.app.register_subscriber(username, email, 0).await;
    world.status = Some(status);
    if status == StatusCode::OK {
        world.subscribers.push(subscriber);
    }
}

#[when("a new subscriber registers")]
async fn register_random_subscriber(world: &mut state::TestWorld) {
    let state::SubscriptionResponse { status, subscriber } =
        world.app.register_random_subscriber().await;
    world.subscribers.push(subscriber);
    if status == StatusCode::OK {
        world.status = Some(status);
    }
}
