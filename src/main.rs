mod settings;
mod templates;

use settings::Settings;
use templates::{FourOhFour, MainPage};

use askama::Template;
use axum::{
    extract::{Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use std::net::SocketAddr;

use tracing::{info, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::FmtSubscriber;

use serde_json;
use std::fs;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref SETTINGS: Settings = match Settings::new() {
        Some(s) => s,
        _ => {
            warn!("Failed to parse settings, defaults will be used instead");
            Settings::from_str("").unwrap()
        }
    };
}

#[tokio::main]
async fn main() {
    // Initialize logging subsystem.
    let trace_sub = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new("link_shortener=debug"))
        .finish();
    tracing::subscriber::set_global_default(trace_sub).unwrap();

    let app = Router::new()
        .route("/", get(handle_main))
        .route("/_assets/*path", get(handle_assets))
        .route("/_assets/new", post(handle_new_link))
        .route("/_assets/del", get(handle_del_link))
        .route("/*path", get(handle_link));

    let listen_addr: SocketAddr = format!("{}:{}", SETTINGS.ip, SETTINGS.port)
        .parse()
        .unwrap();

    info!("Listening on http://{}", listen_addr);

    axum::Server::bind(&listen_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

static THEME_CSS: &str = include_str!("../assets/theme.css");
static FAVICON: &str = include_str!("../assets/favicon.svg");

async fn handle_assets(Path(path): Path<String>) -> impl IntoResponse {
    info!("Got request for {}", path);

    let mut headers = HeaderMap::new();

    if path == "theme.css" {
        headers.insert(header::CONTENT_TYPE, "text/css".parse().unwrap());
        (StatusCode::OK, headers, THEME_CSS)
    } else if path == "favicon.svg" {
        (StatusCode::OK, headers, FAVICON)
    } else {
        (StatusCode::NOT_FOUND, headers, "")
    }
}

async fn handle_main() -> impl IntoResponse {
    info!("Got request for /");

    let template: MainPage =
        serde_json::from_str(&fs::read_to_string("links.json").expect("Unable to read file"))
            .unwrap();
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response())
}

async fn handle_link(Path(path): Path<String>) -> impl IntoResponse {
    info!("Got request for {}", path);
    let links: MainPage =
        serde_json::from_str(&fs::read_to_string("links.json").expect("Unable to read file"))
            .unwrap();

    if let Some(link) = links.links.iter().find(|l| l.src == path) {
        Redirect::permanent(&link.dst).into_response()
    } else {
        let template = FourOhFour {};
        let reply_html = template.render().unwrap();
        (StatusCode::NOT_FOUND, Html(reply_html).into_response()).into_response()
    }
}

async fn handle_new_link(Form(new_link): Form<templates::Link>) -> impl IntoResponse {
    info!("Adding link {:?}", new_link);

    let mut links: MainPage =
        serde_json::from_str(&fs::read_to_string("links.json").expect("Unable to read file"))
            .unwrap();

    // Poor-man's overwrite
    links.links.retain(|x| x.src != new_link.src);
    links.links.push(new_link);

    fs::write("links.json", serde_json::to_string_pretty(&links).unwrap())
        .expect("Unable to write file");

    Redirect::to("/").into_response()
}

async fn handle_del_link(Query(del_link): Query<templates::Link>) -> impl IntoResponse {
    info!("Removing link {:?}", del_link);

    let mut links: MainPage =
        serde_json::from_str(&fs::read_to_string("links.json").expect("Unable to read file"))
            .unwrap();

    links.links.retain(|x| x.src != del_link.src);

    fs::write("links.json", serde_json::to_string_pretty(&links).unwrap())
        .expect("Unable to write file");

    Redirect::to("/").into_response()
}
