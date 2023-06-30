use cucumber::World;
use reqwest::Response;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinHandle;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use hyper::Error;
use zero2prod::email_client::EmailClient;
use zero2prod::opts::{Command, Opts};
use zero2prod::postgres::PostgresStorage;
use zero2prod_common::settings::Settings;

/// The TestWorld contains both the context for every tests,
/// and information that needs to be kept between steps of a
/// scenario.
#[derive(World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub settings: Settings,
    /// TODO: Should we keep 'PostgresStorage', or should
    /// we use '<dyn Storage>' ?
    pub storage: Arc<PostgresStorage>,
    pub email: Arc<EmailClient>,
    pub email_server: MockServer,
    pub resp: Option<Response>,
    pub handle: Option<JoinHandle<Result<(), Error>>>,
}

impl fmt::Debug for TestWorld {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestWorld")
            .field("settings", &self.settings)
            .field("storage", &self.storage)
            .field("email", &self.email)
            .field("resp", &self.resp)
            .field("handle", &self.handle)
            .finish()
    }
}

impl TestWorld {
    /// Creates a new TestWorld, using a 'testing' configuration.
    pub async fn new() -> Self {
        // We are not using a real Email server, so we spawn a new wiremock server,
        // and then use this server's url in our configuration.
        let email_server = MockServer::start().await;

        // Arrange the behaviour of the MockServer adding a Mock:
        // when it receives a POST request on '/email' it will respond with a 200.
        Mock::given(method("POST"))
            .and(path("/email"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&email_server)
            .await;

        let override_email_server_url = format!("email_client.server_url='{}'", email_server.uri());
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("config"),
            run_mode: Some("testing".to_string()),
            settings: vec![override_email_server_url],
            cmd: Command::Run,
        };

        let settings: Settings = opts.try_into().expect("settings");

        let storage = Arc::new(
            PostgresStorage::new(settings.database.clone())
                .await
                .expect("Storage connection"),
        );

        let email = Arc::new(
            EmailClient::new(settings.email_client.clone())
                .await
                .expect("Email connection"),
        );

        TestWorld {
            settings,
            storage,
            email,
            email_server,
            resp: None,
            handle: None,
        }
    }
}
