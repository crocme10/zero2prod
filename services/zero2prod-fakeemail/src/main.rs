use axum::{extract::Json, response::IntoResponse, routing::post, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/email", post(email))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}

async fn email(Json(payload): Json<SendEmailRequest>) -> impl IntoResponse {
    let link = get_url_link(&payload.html_content);
    log::info!(
        "Sending an email from {} to {}, including a link {}",
        payload.from,
        payload.to,
        link
    );
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}

fn get_url_link(s: &str) -> String {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(), 1);
    links[0].as_str().to_owned()
}
