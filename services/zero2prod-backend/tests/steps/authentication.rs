use cucumber::{then, when};
use reqwest::StatusCode;

use crate::state;

// #[tracing::instrument( name = "Testing user registration")]
// #[when(regex = r#"a user with username "(\S*)", email "(\S*)", and password "(\S*)" registers"#)]
// async fn user_registration(world: &mut state::TestWorld, username: String, email: String, password: String) {
//     let state::RegistrationResponse { status_code, status: _, message: _ } =
//         world.app.register_user(username.clone(), email.clone(), password.clone()).await;
//     world.status_code = Some(status_code);
//     if status_code == StatusCode::OK {
//         let user = state::User {
//             username, email, password
//         };
//         world.users.push(user);
//     }
// }

#[tracing::instrument(name = "Testing random user registration")]
#[when(regex = r#"a user registers"#)]
async fn random_user_registration(world: &mut state::TestWorld) {
    let user = world.app.generate_random_user();
    let state::RegistrationResponse {
        status_code,
        status: _,
        message: _,
    } = world
        .app
        .register_user(
            user.username.clone(),
            user.email.clone(),
            user.password.clone(),
        )
        .await;
    world.status_code = Some(status_code);
    if status_code == StatusCode::OK {
        world.users.push(user);
    }
}

// #[then(regex = r#"a user with username "(\S*)" and password "(\S*)" can successfully login"#)]
// async fn successful_login(world: &mut state::TestWorld, username: String, password: String) {
//     let state::LoginResponse { status_code, status: _, message: _ } =
//         world.app.login_user(username.clone(), password.clone()).await;
//     assert_eq!(status_code, StatusCode::OK);
// }

#[then(regex = r#"a user can successfully login"#)]
async fn successful_login(world: &mut state::TestWorld) {
    let user = world.users.pop().unwrap();
    let state::LoginResponse {
        status_code,
        status: _,
        message: _,
    } = world.app.login_user(user.username, user.password).await;
    assert_eq!(status_code, StatusCode::OK);
}
