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
use indicatif::{ProgressBar, ProgressStyle};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

mod cli;
mod formatter;
mod output;

#[derive(Debug, Clone)]
struct AppState {
    args: Arc<Cli>,
    spinner: Arc<Mutex<ProgressBar>>,
    plugin_connected: Arc<Mutex<bool>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner());
    spinner.set_message("waiting for plugin");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let state = AppState {
        args: Arc::new(args),
        spinner: Arc::new(Mutex::new(spinner)),
        plugin_connected: Arc::new(Mutex::new(false)),
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

            let spinner = state.spinner.lock().await;
            spinner.finish_and_clear();

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
    let mut plugin_connected = state.plugin_connected.lock().await;
    if !*plugin_connected {
        *plugin_connected = true;

        let spinner = state.spinner.lock().await;
        spinner.set_message("waiting for test results");

        return (StatusCode::OK, Json((*state.args).clone())).into_response();
    }

    (StatusCode::BAD_REQUEST, "already connected").into_response()
}

async fn output(State(state): State<AppState>, Json(output): Json<Output>) -> impl IntoResponse {
    let spinner = state.spinner.lock().await;
    spinner.finish_and_clear();

    let formatter = JestFormatter::new(state.args.options.verbose.unwrap_or_default());
    let text = formatter.format_output(&output);

    print!("{}", text);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(if output.results.success { 0 } else { 1 });
    });

    (StatusCode::OK, ())
}
