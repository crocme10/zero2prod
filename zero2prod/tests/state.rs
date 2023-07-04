use cucumber::World;
use once_cell::sync::Lazy;
use reqwest::Response;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinHandle;
use wiremock::MockServer;

use zero2prod::application::{Application, Error};
use zero2prod::email_service::EmailService;
use zero2prod::opts::{Command, Opts};
use zero2prod::storage::Storage;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod_common::settings::Settings;

/// The TestWorld contains both the context for every tests
/// and information that needs to be kept between steps of a
/// scenario.
#[derive(World, Debug)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub app: TestApp,
    pub resp: Option<Response>,
    pub confirmation_link: Option<reqwest::Url>,
}

impl TestWorld {
    /// Creates a new TestWorld, using a 'testing' configuration.
    pub async fn new() -> Self {
        let app = spawn_app().await;

        TestWorld {
            app,
            resp: None,
            confirmation_link: None,
        }
    }
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub storage: Arc<dyn Storage + Send + Sync>,
    pub email_server: MockServer,
    pub email_client: Arc<dyn EmailService + Send + Sync>,
    pub api_client: reqwest::Client,
    pub server_handle: Option<JoinHandle<Result<(), Error>>>,
}

impl fmt::Debug for TestApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestApp")
            .field("address", &self.address)
            .field("port", &self.port)
            .finish()
    }
}

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            "test".into(),
            "zero2prod=debug,info".into(),
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber =
            get_subscriber("test".into(), "zero2prod=debug,info".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    // Set up subscriber for logging, only first time per run.
    // Other times use existing subscriber.
    Lazy::force(&TRACING);

    // We are not using a real Email server, so we spawn a new wiremock server,
    // and then use this server's url in our configuration.
    let email_server = MockServer::start().await;

    // We use the email mock server's address in the configuration.
    // This syntax is what would be used on the command line to override the
    // email service's url.
    let override_email_server_url = format!("email_client.server_url='{}'", email_server.uri());

    // Now build the command line arguments
    let opts = Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![override_email_server_url],
        cmd: Command::Run,
    };

    // And then build the configuration that would come from the command line arguments.
    let settings: Settings = opts.try_into().expect("settings");

    let builder = Application::builder()
        .storage(settings.database.clone())
        .await
        .expect("getting storage")
        .email(settings.email_client.clone())
        .await
        .expect("getting email client")
        .listener(settings.application.clone())
        .expect("getting listener")
        .url(settings.application.base_url.clone());

    // Before building the app, we extract a copy of storage and email.
    let storage = builder.storage.clone().unwrap();
    let email_client = builder.email.clone().unwrap();

    // Now build the app, and launch it.
    let app = builder.build();
    let port = app.port();
    let address = format!("{}:{}", settings.application.base_url, app.port());
    let handle = tokio::spawn(app.run_until_stopped());

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestApp {
        address,
        port,
        storage,
        email_server,
        api_client,
        email_client,
        server_handle: Some(handle),
    }
}

impl TestApp {
    /// Get the confirmation links from the mock email.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();

            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            // Make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            // Rewrite URL to include the port
            confirmation_link.set_port(Some(self.port)).unwrap();

            confirmation_link
        };

        let html = get_link(body["HtmlContent"].as_str().unwrap());
        let text = get_link(body["TextContent"].as_str().unwrap());

        ConfirmationLinks { html, text }
    }

    /// Send a post request to the subscriptions endpoint.
    pub async fn post_subscriptions(&self, map: HashMap<&str, String>) -> reqwest::Response {
        let url = format!("{}/subscriptions", self.address);
        self.api_client
            .post(url)
            .json(&map)
            .send()
            .await
            .expect("failed to execute request")
    }
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}
