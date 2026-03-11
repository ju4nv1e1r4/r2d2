mod client;
mod short_term_context_deq;
mod dynamic_prompt;
mod utils;

use axum::{
    routing::post,
    http::StatusCode,
    response::IntoResponse,
    Json,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/generate", post(generate_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn generate_handler(Json(messages): Json<Vec<client::Messages>>) -> impl IntoResponse {
    match client::ModelResponse::generate(messages).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
