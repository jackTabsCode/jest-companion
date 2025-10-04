use crate::cli::Cli;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use clap::Parser;
use std::sync::Arc;

mod cli;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arc::new(Cli::parse());

    let app = Router::new().route("/poll", get(poll)).with_state(args);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:28860").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn poll(State(args): State<Arc<Cli>>) -> impl IntoResponse {
    println!("I have in fact been polled!");
    Json(args.as_ref().clone())
}
