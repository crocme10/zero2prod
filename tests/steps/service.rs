use cucumber::given;
use tokio::sync::oneshot;

use zero2prod::listener::listen_with_host_port;
use zero2prod::server;

use crate::state;

#[given("the service has been started")]
async fn start_service(world: &mut state::TestWorld) {
    let state = world.exec.take().expect("take transaction");
    let (tx, rx) = oneshot::channel::<()>();
    let listener = listen_with_host_port(world.host.as_str(), world.port).unwrap_or_else(|err| {
        panic!(
            "Could not create a listener for {}:{} => {err}",
            world.host, world.port
        )
    });
    world.tx = Some(tx);
    let _ = tokio::spawn(server::run(listener, state.clone(), rx));
    world.exec = Some(state);
}
