use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use mongodb::{bson::doc, bson::oid::ObjectId, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Quote {
    book: String,
    quote: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Quote {
    fn new(book: String, quote: String) -> Self {
        let now = Utc::now();
        Self {
            book,
            quote,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateQuote {
    book: String,
    quote: String,
}

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn create_quote(
    State(client): State<Client>,
    Json(payload): Json<CreateQuote>,
) -> Result<(StatusCode, Json<Quote>), StatusCode> {
    let quote = Quote::new(payload.book, payload.quote);
    let collection = client.database("vinod").collection::<Quote>("quotes");
    let _ = collection.insert_one(&quote, None).await;

    Ok((StatusCode::CREATED, axum::Json(quote)))
}

pub async fn read_quotes(State(client): State<Client>) -> Result<Json<Vec<Quote>>, StatusCode> {
    let collection = client.database("vinod").collection::<Quote>("quotes");

    let mut quote_cursor = collection
        .find(None, None)
        .await
        .expect("could not load quotes data.");
    let mut quotes = Vec::new();

    while quote_cursor.advance().await.unwrap() {
        quotes.push(quote_cursor.deserialize_current().unwrap());
    }

    Ok(axum::Json(quotes))
}

pub async fn update_quote(
    State(client): State<Client>,
    Path(id): Path<ObjectId>,
    Json(payload): Json<CreateQuote>,
) -> StatusCode {
    // TODO: remove repeated collection and database call
    let collection = client.database("vinod").collection::<Quote>("quotes");
    let filter = doc! { "_id": id };
    let update = doc! { "$set": { "book": payload.book, "quote": payload.quote } };
    let res = collection
        .update_one(filter, update, None)
        .await
        .expect("Failed to update");

    if let Some(updated_count) = Some(res.modified_count) {
        if updated_count > 0 {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn delete_quote(State(client): State<Client>, Path(id): Path<ObjectId>) -> StatusCode {
    // TODO: remove repeated collection and database call
    let collection = client.database("vinod").collection::<Quote>("quotes");
    let filter = doc! {"_id": id};
    let res = collection.delete_one(filter, None).await.unwrap();
    match res.deleted_count {
        0 => StatusCode::NOT_MODIFIED,
        _ => StatusCode::OK,
    }
}
