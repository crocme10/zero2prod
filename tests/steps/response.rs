use cucumber::then;
use reqwest::StatusCode;

use crate::state::TestWorld;

#[then("the response is 200 OK")]
fn response_is_ok(world: &mut TestWorld) {
    assert!(world.resp.as_ref().unwrap().status() == StatusCode::OK);
}

#[then("the response is 400 Bad Request")]
fn response_is_bad_request(world: &mut TestWorld) {
    assert!(world.resp.as_ref().unwrap().status() == StatusCode::BAD_REQUEST);
}
