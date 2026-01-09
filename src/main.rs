use axum::{Router, extract::State, response::Html, routing::get};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    title: String,
    links: Vec<LinkItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct LinkItem {
    name: String,
    url: String,
    alt_name: Option<String>,
    icon: Option<String>,
}

#[tokio::main]
async fn main() {
    let config = load_config("config.json").expect("No config file found in root");
    let state = AppState {
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/", get(index))
        .with_state(state)
        .nest_service("/static", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(render_index(&state.config).into_string())
}

fn render_index(cfg: &Config) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (cfg.title) }
                link rel="stylesheet" href="/static/styles.css";
            }
            body {
                h1 { (cfg.title) }
                ul {
                    @for link in &cfg.links {
                        (card(link))
                    }
                }
            }
        }
    }
}

fn card(link: &LinkItem) -> Markup {
    let src = link.icon.clone().unwrap_or(format!(
        "https://cdn.jsdelivr.net/gh/selfhst/icons@main/webp/{}.webp",
        parse_icon_name(link.name.clone())
    ));
    let name = link.alt_name.clone().unwrap_or(link.name.clone());

    html! {
        li {
            a href=(link.url) {
                img class="icon" src=((src));
                span class="label" { (name) }
            }
        }
    }
}

fn parse_icon_name(name: String) -> String {
    name.to_lowercase().replace(" ", "-")
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
