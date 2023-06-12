use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use cucumber::{then, when};
use zero2prod::server::State;
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

#[then(regex = r#"the database stored the username "(\S+)" and the email "(\S+)""#)]
async fn query_database(world: &mut state::TestWorld, username: String, email: String) {
    // To run this test, we have to stop the server, take the transaction, and query the database.
    // After the checks, we put the transaction back in its world, so that it is available for
    // more tests.
    let _ = world.tx.take().expect("take tx shutdown").send(());
    // FIXME Should we wait a bit ?
    sleep(Duration::from_millis(1000)).await;
    println!("1000 ms have elapsed");
    let state = world.exec.take().expect("take executor");
    let mut exec = Arc::into_inner(state).expect("extract state from arc").into_inner().exec;
    let saved = sqlx::query!(
        r#"SELECT email, username FROM subscriptions WHERE username = $1"#,
        username
    )
    .fetch_one(&mut exec)
    .await
    .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, email);
    assert_eq!(saved.username, username);
    let state = Arc::new(Mutex::new(State { exec }));
    world.exec = Some(state);
}
