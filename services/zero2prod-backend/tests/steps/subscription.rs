use cucumber::when;
use std::collections::HashMap;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::state;

#[when(regex = r#"the user subscribes with username "(\S*)" and email "(\S*)""#)]
async fn subscribes_full(world: &mut state::TestWorld, username: String, email: String) {
    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a POST request on '/email' it will respond with a 200.
    Mock::given(method("POST"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&world.app.email_server)
        .await;

    let mut map = HashMap::new();
    map.insert("username", username);
    map.insert("email", email);

    let resp = world.app.post_subscriptions(map).await;
    world.resp = Some(resp);
}
