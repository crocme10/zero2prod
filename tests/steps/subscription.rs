use cucumber::when;
use std::collections::HashMap;

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
