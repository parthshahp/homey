use axum::{Router, extract::State, response::Html, routing::get};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use std::sync::Arc;

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
    icon: Option<String>,
}

#[tokio::main]
async fn main() {
    let config = load_config("config.json").expect("No config file found in root");
    let state = AppState {
        config: Arc::new(config),
    };

    let app = Router::new().route("/", get(index)).with_state(state);

    // run our app with hyper, listening globally on port 3000
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
            }
            body {
                h1 { (cfg.title) }
                ul {
                    @for link in &cfg.links {
                        li {
                            a href=(link.url) { (link.name) }
                        }
                    }
                }
            }
        }
    }
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
