use crate::{cli::Cli, formatter::JestFormatter, output::Output};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use clap::Parser;
use colored::*;
use std::{sync::Arc, time::Duration};

mod cli;
mod formatter;
mod output;

#[derive(Debug, Clone)]
struct AppState {
    args: Arc<Cli>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let state = AppState {
        args: Arc::new(args),
    };

    let app = Router::new()
        .route("/poll", get(poll))
        .route("/output", post(output))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:28860").await?;

    {
        let state = state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(state.args.server_timeout)).await;
            eprintln!(
                "{}",
                "No places have reported anything. Studio might not be open?".red()
            );
            std::process::exit(1);
        })
    };

    axum::serve(listener, app).await?;

    Ok(())
}

async fn poll(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json((*state.args).clone()))
}

async fn output(State(state): State<AppState>, Json(output): Json<Output>) -> impl IntoResponse {
    let formatter = JestFormatter::new(state.args.options.verbose);
    let text = formatter.format_output(&output);

    println!("{}", text);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(if output.results.success { 0 } else { 1 });
    });

    (StatusCode::OK, ())
}
