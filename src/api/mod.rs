use crate::{download::DownloadEngine, models::DownloadRequest};
use anyhow::Result;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateDownload {
    pub url: String,
    pub output: String,
}

pub async fn serve(bind_addr: String, engine: Arc<DownloadEngine>) -> Result<()> {
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/downloads", post(create_download))
        .route("/downloads/:id", get(get_download))
        .with_state(engine);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("API server listening on {}", bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_download(
    axum::extract::State(engine): axum::extract::State<Arc<DownloadEngine>>,
    Json(payload): Json<CreateDownload>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let req = DownloadRequest {
        url: payload.url,
        output: payload.output.into(),
    };

    let id = engine
        .enqueue(req.url, req.output.to_string_lossy().to_string())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "id": id })))
}

async fn get_download(
    axum::extract::State(engine): axum::extract::State<Arc<DownloadEngine>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let status = engine
        .status(&id)
        .await
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::to_value(status).map_err(|_| {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?))
}
