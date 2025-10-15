use crate::{
    cli::{Cli, JestOptions},
    config::Config,
    output::Output,
    resolver::resolve_path,
};
use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{fs, sync::Mutex};

mod cli;
mod config;
mod output;
mod resolver;

#[derive(Debug, Clone)]
struct AppState {
    args: Arc<Cli>,
    config: Arc<Config>,
    spinner: Arc<Mutex<ProgressBar>>,
    plugin_connected: Arc<Mutex<bool>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let config = fs::read_to_string("jest-companion.toml")
        .await
        .context("Failed to read config file")?;

    let config: Config = toml::from_str(&config).context("Failed to parse config file")?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner());
    spinner.set_message("waiting for plugin");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let state = AppState {
        args: Arc::new(args),
        config: Arc::new(config),
        spinner: Arc::new(Mutex::new(spinner)),
        plugin_connected: Arc::new(Mutex::new(false)),
    };

    let app = Router::new()
        .route("/poll", get(poll))
        .route("/output", post(output))
        .route("/fs/file/{*path}", put(fs_write))
        .route("/fs/dir/{*path}", put(fs_create_dir_all))
        .route("/fs/exists/{*path}", get(fs_exists))
        .route("/fs/file/{*path}", delete(fs_delete))
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

#[derive(Serialize)]
struct PollBody {
    projects: Vec<String>,
    options: JestOptions,
}

async fn poll(State(state): State<AppState>) -> impl IntoResponse {
    let mut plugin_connected = state.plugin_connected.lock().await;
    if !*plugin_connected {
        *plugin_connected = true;

        let spinner = state.spinner.lock().await;
        spinner.set_message("waiting for test results");

        let projects: Vec<String> = state.config.projects.keys().cloned().collect();

        let body: PollBody = PollBody {
            projects,
            options: state.args.options.clone(),
        };

        return (StatusCode::OK, Json(body)).into_response();
    }

    (StatusCode::BAD_REQUEST, "already connected").into_response()
}

async fn output(State(state): State<AppState>, Json(output): Json<Output>) -> impl IntoResponse {
    let spinner = state.spinner.lock().await;
    spinner.finish_and_clear();

    let formatter = output::Formatter::new(state.args.options.verbose.unwrap_or_default());
    let text = formatter.format_output(&output);

    print!("{}", text);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(if output.was_successful() { 0 } else { 1 });
    });

    (StatusCode::OK, ())
}

async fn fs_write(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
    body: String,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, Path::new(".")) {
        Some(real_path) => {
            if let Some(parent) = real_path.parent()
                && let Err(e) = fs::create_dir_all(parent).await
            {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            match fs::write(&real_path, body).await {
                Ok(_) => (StatusCode::OK, ()).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, "could not resolve path").into_response(),
    }
}

async fn fs_create_dir_all(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, Path::new(".")) {
        Some(real_path) => match fs::create_dir_all(&real_path).await {
            Ok(_) => (StatusCode::OK, ()).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "could not resolve path").into_response(),
    }
}

async fn fs_exists(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, Path::new(".")) {
        Some(real_path) => match fs::metadata(&real_path).await {
            Ok(_) => (StatusCode::OK, ()).into_response(),
            Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "could not resolve path").into_response(),
    }
}

async fn fs_delete(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, Path::new(".")) {
        Some(real_path) => match fs::remove_file(&real_path).await {
            Ok(_) => (StatusCode::OK, ()).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "could not resolve path").into_response(),
    }
}
