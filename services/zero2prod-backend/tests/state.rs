use cucumber::World;
use fake::faker::internet::en::{Password, SafeEmail};
use fake::faker::name::en::Name;
use fake::locales::Data;
use fake::locales::*;
use fake::Dummy;
use fake::Fake;
use once_cell::sync::Lazy;
use rand::prelude::SliceRandom;
use reqwest::header;
use secrecy::Secret;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time;
use tokio::task::JoinHandle;
use uuid::Uuid;
use wiremock::MockServer;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use common::settings::Settings;
use zero2prod::application::{Application, Error};
use zero2prod::domain::ports::secondary::{AuthenticationStorage, SubscriptionStorage};
use zero2prod::domain::Credentials;
use zero2prod::domain::ports::secondary::EmailService;
use zero2prod::opts::{Command, Opts};
use zero2prod::routes::newsletter::BodyData;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

/// The TestWorld contains both the context for every tests
/// and information that needs to be kept between steps of a
/// scenario.
#[derive(World, Debug)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub app: TestApp,
    // We store an optional response's status code. The response, typically, can be set in
    // a 'when' step, and checked in a following 'then' step.
    pub status_code: Option<reqwest::StatusCode>,
    // We store a representation of subscribers. This is not necessarily
    // what is in storage, but it is used to keep track of information
    // between steps.
    pub subscribers: Vec<Subscriber>,
    pub users: Vec<User>,
}

impl TestWorld {
    /// Creates a new TestWorld, using a 'testing' configuration.
    pub async fn new() -> Self {
        let app = spawn_app().await;

        TestWorld {
            app,
            status_code: None,
            subscribers: vec![],
            users: vec![],
        }
    }

    pub fn count_confirmed_subscribers(&self) -> usize {
        self.subscribers
            .iter()
            .filter(|subscriber| subscriber.status == "confirmed")
            .count()
    }
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub authentication: Arc<dyn AuthenticationStorage + Send + Sync>,
    pub subscription: Arc<dyn SubscriptionStorage + Send + Sync>,
    // A Mock Server, so we don't have to use a real email server.
    pub email_server: MockServer,
    // The interface to email
    pub email_client: Arc<dyn EmailService + Send + Sync>,
    // The API to access the server.
    pub api_client: reqwest::Client,
    // The server handle, so that it can be killed.
    pub server_handle: Option<JoinHandle<Result<(), Error>>>,
    // A fake user, used to test authentication.
    pub user: Option<TestUser>,
}

pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub credentials: Credentials,
}

pub struct TestUserGenerator<L>(pub L);

impl<L: Data> Dummy<TestUserGenerator<L>> for TestUser {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_config: &TestUserGenerator<L>, rng: &mut R) -> Self {
        let username = *L::NAME_FIRST_NAME.choose(rng).unwrap();
        let email = *L::INTERNET_FREE_EMAIL_PROVIDER.choose(rng).unwrap();
        let password = *L::LOREM_WORD.choose(rng).unwrap();
        TestUser {
            id: Uuid::new_v4(),
            email: email.into(),
            credentials: Credentials {
                username: username.into(),
                password: Secret::new(password.to_string()),
            },
        }
    }
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
    // Lazy::force(&TRACING);

    tracing::info!("Spawning new app");
    // We are not using a real Email server, so we spawn a wiremock server,
    // and then use this server's url in our configuration.
    // We don't attach any expectation yet to that MockServer, this will
    // be done later when executing steps.
    let email_server = MockServer::start().await;

    // We use the email mock server's address in the configuration.
    // This syntax is what would be used on the command line to override the
    // email service's url.
    let override_email_server_url = format!("email_client.server_url='{}'", email_server.uri());

    // Now build the command line arguments
    let opts = Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![override_email_server_url],
        cmd: Command::Run,
    };

    // And then build the configuration that would come from the command line arguments.
    let settings: Settings = opts.try_into().expect("settings");

    let builder = Application::builder()
        .authentication(settings.database.clone())
        .await
        .expect("authentication service")
        .subscription(settings.database.clone())
        .await
        .expect("subscription service")
        .email(settings.email_client.clone())
        .await
        .expect("getting email client")
        .listener(settings.application.clone())
        .expect("getting listener")
        .url(settings.application.base_url.clone())
        .static_dir(settings.application.static_dir.clone())
        .expect("getting static dir")
        .secret("secret".to_string());

    // Before building the app, we extract a copy of storage and email.
    let authentication = builder.authentication.clone().unwrap();
    let subscription = builder.subscription.clone().unwrap();
    let email_client = builder.email.clone().unwrap();

    // Now build the app, and launch it.
    let app = builder.build();
    let port = app.port();
    let address = format!("{}:{}", settings.application.base_url, app.port());
    let handle = tokio::spawn(app.run_until_stopped());

    let api_client = reqwest::Client::builder()
        .timeout(time::Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("api client build");

    // This is a user whose credentials are stored in the database, and it will be used
    // to authorize the API calls that need to have valid credentials.
    let user: TestUser = TestUserGenerator(EN).fake();

    authentication
        .store_credentials(user.id, &user.email, &user.credentials)
        .await
        .expect("Store credentials");

    TestApp {
        address,
        port,
        authentication,
        subscription,
        email_server,
        api_client,
        email_client,
        server_handle: Some(handle),
        user: Some(user),
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
        let url = format!("{}/api/subscriptions", self.address);
        self.api_client
            .post(url)
            .json(&map)
            .send()
            .await
            .expect("failed to post on subscriptions endpoint")
    }

    /// Send a post request to the user registration endpoint.
    pub async fn post_registration(&self, map: HashMap<&str, String>) -> reqwest::Response {
        let url = format!("{}/api/v1/register", self.address);
        self.api_client
            .post(url)
            .json(&map)
            .send()
            .await
            .expect("failed to post on user registration endpoint")
    }

    /// Send a post request to the user login endpoint.
    pub async fn post_credentials(&self, map: HashMap<&str, String>) -> reqwest::Response {
        let url = format!("{}/api/v1/login", self.address);
        self.api_client
            .post(url)
            .json(&map)
            .send()
            .await
            .expect("failed to post on user login endpoint")
    }

    /// Send a newsletter
    pub async fn send_newsletter(&self, newsletter: &BodyData) -> reqwest::Response {
        let url = format!("{}/api/newsletter", self.address);
        self.api_client
            .post(url)
            .json(&newsletter)
            .header(
                header::AUTHORIZATION,
                format!(
                    "Basic {}",
                    self.user.as_ref().expect("user").credentials.encode()
                ),
            )
            .send()
            .await
            .expect("failed to post on newsletter endpoint")
    }

    /// Register a random subscriber.
    pub async fn register_random_subscriber(&self) -> SubscriptionResponse {
        // We draw random information to define the subscription.
        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        self.register_subscriber(username, email, 1).await
    }

    pub async fn register_subscriber(
        &self,
        username: String,
        email: String,
        expect: u64,
    ) -> SubscriptionResponse {
        // Then we setup the application email server so that it must
        // receive an email (the confirmation email sent to a new subscriber)
        let _mock_guard = Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .named("Register new subscriber")
            .expect(expect)
            .mount_as_scoped(&self.email_server)
            .await;

        // Then we send the subscription information
        let mut map = HashMap::new();
        map.insert("username", username.clone());
        map.insert("email", email.clone());
        let resp = self.post_subscriptions(map).await;
        // Finally we return the subscription information so that
        // the caller can make something with it.
        SubscriptionResponse {
            status_code: resp.status(),
            subscriber: Subscriber {
                username,
                email,
                status: "pending_confirmation".to_string(),
                confirmation_link: None,
            },
        }
    }

    /// Register a random user.
    pub fn generate_random_user(&self) -> User {
        // We draw random information to define the subscription.
        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let password = Password(20..32).fake::<String>();
        User {
            username,
            email,
            password,
        }
    }

    pub async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> RegistrationResponse {
        // Then we send the registration information
        let mut map = HashMap::new();
        map.insert("username", username.clone());
        map.insert("email", email.clone());
        map.insert("password", password.clone());
        tracing::info!("Posting registration");
        let resp = self.post_registration(map).await;
        // Finally we return the subscription information so that
        // the caller can make something with it.
        RegistrationResponse {
            status_code: resp.status(),
            status: "foo".to_string(),
            message: "foo".to_string(),
        }
    }

    pub async fn login_user(&self, username: String, password: String) -> LoginResponse {
        // Then we send the registration information
        let mut map = HashMap::new();
        map.insert("username", username.clone());
        map.insert("password", password.clone());
        tracing::info!("Posting credentials");
        let resp = self.post_credentials(map).await;
        // Finally we return the subscription information so that
        // the caller can make something with it.
        tracing::info!("Logging response: {:?}", resp);
        LoginResponse {
            status_code: resp.status(),
            status: "foo".to_string(),
            message: "foo".to_string(),
        }
    }

    // pub async fn add_test_user(&mut self) {
    //     let credentials: Credentials = C(EN).fake();
    //     let id = Uuid::new_v4();
    //     self.storage.create_user(id, &credentials).await.expect("Create test user");
    //     self.user = Some(credentials);
    // }
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct RegistrationResponse {
    pub status_code: reqwest::StatusCode,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub status_code: reqwest::StatusCode,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub username: String,
    pub email: String,
    pub status: String,
    pub confirmation_link: Option<reqwest::Url>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionResponse {
    // Note we can't store the response as it is not Clone.
    // For now, just store the status code
    pub status_code: reqwest::StatusCode,
    pub subscriber: Subscriber,
}
