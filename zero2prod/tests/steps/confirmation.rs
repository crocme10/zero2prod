use cucumber::when;

use crate::state;
use crate::utils::testing_url_for_endpoint;

#[when(regex = r#"the user calls the confirmation endpoint"#)]
async fn confirm(world: &mut state::TestWorld) {
    let url = testing_url_for_endpoint("subscription/confirmation");
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await.expect("response");
    world.resp = Some(resp);
}
