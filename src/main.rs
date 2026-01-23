use axum::{
    Router,
    extract::{Form, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use maud::{DOCTYPE, Markup, html};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

const CONFIG_PATH: &str = "config.json";

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<Config>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    #[serde(default = "default_title")]
    title: String,
    links: Vec<LinkItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinkItem {
    name: String,
    url: String,
    #[serde(rename = "altName")]
    alt_name: Option<String>,
    icon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SaveForm {
    json: String,
}

#[derive(Debug, Deserialize)]
struct AdminQuery {
    saved: Option<String>,
}

#[tokio::main]
async fn main() {
    let config = load_config(CONFIG_PATH).expect("No config file found in root");
    let state = AppState {
        config: Arc::new(RwLock::new(config)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/admin", get(admin))
        .route("/admin/save", post(save_config))
        .with_state(state)
        .nest_service("/static", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let config = state.config.read().await;
    Html(render_index(&config).into_string())
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
                header class="page-header" {
                    h1 { (cfg.title) }
                    a class="settings-link" href="/admin" { "Edit" }
                }
                ul {
                    @for link in &cfg.links {
                        (card(link))
                    }
                }
            }
        }
    }
}

async fn admin(State(state): State<AppState>, Query(query): Query<AdminQuery>) -> Html<String> {
    let config = state.config.read().await;
    let json_text = serde_json::to_string_pretty(&*config).unwrap_or_else(|_| "{}".to_string());
    let message = if query.saved.is_some() {
        Some("Saved.")
    } else {
        None
    };
    Html(render_admin(&json_text, message).into_string())
}

async fn save_config(
    State(state): State<AppState>,
    Form(form): Form<SaveForm>,
) -> impl IntoResponse {
    let parsed: Config = match serde_json::from_str(&form.json) {
        Ok(config) => config,
        Err(err) => {
            let message = format!("Invalid JSON: {err}");
            let body = render_admin(&form.json, Some(&message)).into_string();
            return (StatusCode::BAD_REQUEST, Html(body)).into_response();
        }
    };

    let mut serialized = match serde_json::to_string_pretty(&parsed) {
        Ok(text) => text,
        Err(err) => {
            let message = format!("Failed to serialize config: {err}");
            let body = render_admin(&form.json, Some(&message)).into_string();
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(body)).into_response();
        }
    };
    serialized.push('\n');

    if let Err(err) = std::fs::write(CONFIG_PATH, serialized) {
        let message = format!("Failed to write config.json: {err}");
        let body = render_admin(&form.json, Some(&message)).into_string();
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(body)).into_response();
    }

    let mut config = state.config.write().await;
    *config = parsed;

    Redirect::to("/admin?saved=1").into_response()
}

fn render_admin(json_text: &str, message: Option<&str>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Homey Settings" }
                link rel="stylesheet" href="/static/styles.css";
            }
            body {
                header class="page-header" {
                    h1 { "Settings" }
                    a class="settings-link" href="/" { "Back" }
                }
                @if let Some(message) = message {
                    p class="notice" { (message) }
                }
                form class="editor" method="post" action="/admin/save" {
                    label for="json-editor" { "config.json" }
                    textarea id="json-editor" name="json" spellcheck="false" { (json_text) }
                    div class="editor-actions" {
                        button type="submit" { "Save" }
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

fn default_title() -> String {
    "Homey".to_string()
}
