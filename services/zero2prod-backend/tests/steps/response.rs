use cucumber::then;
use reqwest::StatusCode;
use speculoos::prelude::*;

use crate::state::TestWorld;

#[then("the response is 200 OK")]
fn response_is_ok(world: &mut TestWorld) {
    assert_that(&world.resp.as_ref().unwrap().status()).is_equal_to(StatusCode::OK);
}

#[then("the response is 400 Bad Request")]
fn response_is_bad_request(world: &mut TestWorld) {
    assert_that(&world.resp.as_ref().unwrap().status()).is_equal_to(StatusCode::BAD_REQUEST);
}

#[then("the response is 500 Internal Server Error")]
fn response_is_internal_server_error(world: &mut TestWorld) {
    assert_that(&world.resp.as_ref().unwrap().status())
        .is_equal_to(StatusCode::INTERNAL_SERVER_ERROR);
}

#[then("the response is 401 Unauthorized")]
fn response_is_unauthorized(world: &mut TestWorld) {
    assert_that(&world.resp.as_ref().unwrap().status()).is_equal_to(StatusCode::UNAUTHORIZED);
}
