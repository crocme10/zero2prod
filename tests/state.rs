use cucumber::World;
use reqwest::Response;

#[derive(Debug, Default, World)]
pub struct TestWorld {
    // Wrapped in Option so that TestWorld could derive Default.
    pub resp: Option<Response>,
}
