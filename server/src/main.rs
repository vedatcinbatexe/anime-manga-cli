mod allanime;

use axum::{extract::Query, http::Method, response::Json, routing::get, Router};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct EpisodesQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct StreamQuery {
    pub id: String,
    pub episode: String,
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .unwrap();

    let client = std::sync::Arc::new(client);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/search",
            get({
                let client = client.clone();
                move |Query(params): Query<SearchQuery>| {
                    let client = client.clone();
                    async move {
                        match allanime::search(&client, &params.q).await {
                            Ok(results) => Json(serde_json::json!({ "results": results })),
                            Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
                        }
                    }
                }
            }),
        )
        .route(
            "/episodes",
            get({
                let client = client.clone();
                move |Query(params): Query<EpisodesQuery>| {
                    let client = client.clone();
                    async move {
                        match allanime::episodes(&client, &params.id).await {
                            Ok(eps) => Json(serde_json::json!({ "episodes": eps })),
                            Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
                        }
                    }
                }
            }),
        )
        .route(
            "/links",
            get({
                let client = client.clone();
                move |Query(params): Query<StreamQuery>| {
                    let client = client.clone();
                    async move {
                        match allanime::stream_links(&client, &params.id, &params.episode).await {
                            Ok(links) => Json(serde_json::json!({ "links": links })),
                            Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
                        }
                    }
                }
            }),
        )
        .layer(cors);

    println!("🚀 Server running on http://localhost:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
