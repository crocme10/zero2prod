use cucumber::{then, when};
use reqwest::StatusCode;

use crate::state::{self, User};

#[tracing::instrument(name = "Testing random user registration")]
#[when(regex = r#"a user registers"#)]
async fn random_user_registration(world: &mut state::TestWorld) {
    let user = world.app.as_ref().map(|app| app.generate_random_user()).expect("Could not generate random user");
    let User { username, email, password } = user.clone();
    tracing::info!("Generated user: {:?}", user);
    if let Some(app) = &world.app {
        let state::RegistrationResponse {
            status_code,
            status: _,
            message: _ } = app.register_user(username, email, password).await;
        world.status_code = Some(status_code);
        if status_code == StatusCode::OK {
            world.users.push(user);
        }
    }
}

#[then(regex = r#"a user can successfully login"#)]
async fn successful_login(world: &mut state::TestWorld) {
    let user = world.users.pop().unwrap();
    if let Some(app) = &world.app {
        let state::LoginResponse {
            status_code,
            status: _,
            message: _,
        } = app.login_user(user.username, user.password).await;
        assert_eq!(status_code, StatusCode::OK);

    }
}
