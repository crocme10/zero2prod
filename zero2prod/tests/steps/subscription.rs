use cucumber::{then, when};
use std::collections::HashMap;
use zero2prod::storage::Storage;

use crate::state;
use crate::utils::testing_url_for_endpoint;

#[when(regex = r#"the user subscribes with username "(\S*)" and email "(\S*)""#)]
async fn subscribes_full(world: &mut state::TestWorld, username: String, email: String) {
    let url = testing_url_for_endpoint("subscriptions");
    let mut map = HashMap::new();
    map.insert("username", username);
    map.insert("email", email);
    let client = reqwest::Client::new();
    let resp = client.post(url).json(&map).send().await.expect("response");
    world.resp = Some(resp);
}

#[then(regex = r#"the database stored the username "(\S+)" and the email "(\S+)""#)]
async fn query_database(world: &mut state::TestWorld, username: String, email: String) {
    let subscription = world
        .storage
        .get_subscription_by_username(&username)
        .await
        .expect("get subscription");
    let subscription = subscription.expect("subscription");
    assert_eq!(subscription.email, email);
    assert_eq!(subscription.username, username);
    assert_eq!(subscription.status, "pending_confirmation");
}
