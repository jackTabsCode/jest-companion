use crate::{
    cli::{Cli, JestOptions},
    config::Config,
    output::Output,
    resolver::resolve_path,
};
use anyhow::Context;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use clap::Parser;
use colored::*;
use fs_err::tokio as fs;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

mod cli;
mod config;
mod log;
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

    let config = fs::read_to_string(args.path.join("jest-companion.toml")).await?;
    let config: Config = toml::from_str(&config).context("Failed to parse config file")?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner());
    spinner.set_message("Waiting for plugin");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let state = AppState {
        args: Arc::new(args),
        config: Arc::new(config),
        spinner: Arc::new(Mutex::new(spinner)),
        plugin_connected: Arc::new(Mutex::new(false)),
    };

    let app = Router::new()
        .route("/output", post(output))
        .route("/poll", post(poll))
        .route("/run-error", post(run_error))
        .route("/fs/file/{*path}", put(fs_write))
        .route("/fs/dir/{*path}", put(fs_create_dir_all))
        .route("/fs/exists/{*path}", get(fs_exists))
        .route("/fs/file/{*path}", delete(fs_delete))
        .with_state(state.clone())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:28860").await?;

    {
        let state = state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(state.args.server_timeout)).await;

            let spinner = state.spinner.lock().await;
            spinner.finish_and_clear();

            error!("No places have reported anything. Studio might not be open?");
            std::process::exit(1);
        })
    };

    axum::serve(listener, app).await?;

    Ok(())
}

const PROTOCOL_VERSION: &str = "1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PollRequestBody {
    protocol_version: String,
    rojo_connected: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PollResponseBody {
    projects: Vec<String>,
    options: JestOptions,
}

async fn poll(
    State(state): State<AppState>,
    Json(body): Json<PollRequestBody>,
) -> impl IntoResponse {
    let mut plugin_connected = state.plugin_connected.lock().await;
    if *plugin_connected {
        warn!("A plugin tried to connect while we are already listening to one.");
        return (StatusCode::BAD_REQUEST, "Already connected").into_response();
    }

    if body.protocol_version != PROTOCOL_VERSION {
        warn!(
            "The plugin tried to connect with protocol version {} but we are expecting {PROTOCOL_VERSION}. Make sure your versions align.",
            body.protocol_version
        );

        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Incorrect protocol version: expected {PROTOCOL_VERSION}, got {}",
                body.protocol_version
            ),
        )
            .into_response();
    }

    *plugin_connected = true;

    if !body.rojo_connected {
        warn!("Rojo is not connected on the running Studio instance, just so you know!");
    }

    let spinner = state.spinner.lock().await;
    spinner.set_message("Waiting for test results");

    let projects: Vec<String> = state.config.projects.keys().cloned().collect();

    let body = PollResponseBody {
        projects,
        options: state.args.options.clone(),
    };

    (StatusCode::OK, Json(body)).into_response()
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

async fn run_error(State(state): State<AppState>) -> impl IntoResponse {
    let spinner = state.spinner.lock().await;
    spinner.finish_and_clear();

    error!("The test runner encountered an error. See the Studio output for more details.");

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(1);
    });

    (StatusCode::OK, ())
}

async fn fs_write(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
    body: String,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, &state.args.path) {
        Some(real_path) => {
            if let Some(parent) = real_path.parent()
                && let Err(e) = fs::create_dir_all(parent).await
            {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            match fs::write(&real_path, body).await {
                Ok(_) => {
                    log!("File written: {}", real_path.display());
                    (StatusCode::OK, ()).into_response()
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, "Could not resolve path").into_response(),
    }
}

async fn fs_create_dir_all(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, &state.args.path) {
        Some(real_path) => match fs::create_dir_all(&real_path).await {
            Ok(_) => {
                log!("Directory created: {}", real_path.display());
                (StatusCode::OK, ()).into_response()
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "Could not resolve path").into_response(),
    }
}

async fn fs_exists(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, &state.args.path) {
        Some(real_path) => match fs::metadata(&real_path).await {
            Ok(_) => (StatusCode::OK, ()).into_response(),
            Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "Could not resolve path").into_response(),
    }
}

async fn fs_delete(
    State(state): State<AppState>,
    AxumPath(virtual_path): AxumPath<String>,
) -> impl IntoResponse {
    match resolve_path(&state.config, &virtual_path, &state.args.path) {
        Some(real_path) => match fs::remove_file(&real_path).await {
            Ok(_) => {
                log!("File deleted: {}", real_path.display());
                (StatusCode::OK, ()).into_response()
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        None => (StatusCode::NOT_FOUND, "Could not resolve path").into_response(),
    }
}
