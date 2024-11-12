mod handlers;
mod utils;
use axum::{
    http::{header, HeaderValue},
    routing::{delete, get, post, put},
    Router,
};
use dotenv::dotenv;
use std::{env, time::Duration};
use tower_http::{
    limit::RequestBodyLimitLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utils::connection::dbconnect;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), axum::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let (_,mongo_db) = dbconnect().await.unwrap();

    let app = Router::new()
        .route("/", get(handlers::health))
        .route("/quotes", post(handlers::create_quote))
        .route("/quotes", get(handlers::read_quotes))
        .route("/quotes/:id", put(handlers::update_quote))
        .route("/quotes/:id", delete(handlers::delete_quote))
        // timeout requests after 10 secs, returning 408 status code
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        // don't allow request bodies larger than 1024 bytes, returning 413 status code
        .layer(RequestBodyLimitLayer::new(1024))
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::SERVER,
            HeaderValue::from_static("rust-axum"),
        )).with_state(mongo_db);
    let app = app.fallback(handlers::handler_404);

    println!("🚀 Starting server on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
