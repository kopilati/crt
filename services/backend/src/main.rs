use std::{collections::HashSet, env, net::SocketAddr};

use anyhow::Context;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use crt_to_cypher::refinement::AgentRefinement;
use dotenvy::dotenv;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
#[cfg(not(feature = "stub"))]
use serde_json::json;
use serde_json::{to_value, Value};
use tower_http::cors::{Any, CorsLayer};
#[cfg(not(feature = "stub"))]
use tracing::trace;
use tracing::{error, info};

#[cfg(feature = "stub")]
const STUB_DATA: &str = include_str!("stub.json");

#[cfg_attr(feature = "stub", allow(dead_code))]
#[derive(Clone)]
struct AppState {
    client: Client,
    agent_base: String,
    agent_name: String,
}

#[derive(Debug, Deserialize)]
struct RefineRequest {
    content: String,
}

#[cfg(not(feature = "stub"))]
#[derive(Debug, Deserialize)]
struct AgentResponse {
    output_text: String,
    run_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let agent_base =
        env::var("AGENT_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let agent_name = env::var("AGENT_NAME").unwrap_or_else(|_| "goldratt".to_string());

    info!(%agent_base, %agent_name, "Configured agent service");

    let addr: SocketAddr = env::var("BACKEND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .context("Invalid BACKEND_ADDR")?;

    let state = AppState {
        client: Client::new(),
        agent_base,
        agent_name,
    };

    let app = Router::new()
        .route("/api/refine", post(refine))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([axum::http::Method::POST, axum::http::Method::OPTIONS])
                .allow_headers(Any),
        );

    info!(%addr, "Starting Rust backend");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn refine(
    State(state): State<AppState>,
    Json(request): Json<RefineRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let payload = request.content.trim();
    if payload.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "content must not be empty".to_string(),
        ));
    }
    let existing_entity_ids = extract_entity_ids(payload);
    let existing_link_ids = extract_link_ids(payload);

    #[cfg(feature = "stub")]
    {
        let _ = &state;
        return match load_stub_response(&existing_entity_ids, &existing_link_ids) {
            Ok(response) => {
                info!("Serving stub refine response");
                Ok(Json(response))
            }
            Err(err) => {
                error!(?err, "Failed to load stub response");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load stub response".to_string(),
                ))
            }
        };
    }

    #[cfg(not(feature = "stub"))]
    {
        let existing_ids = existing_entity_ids;
        let url = format!(
            "{}/agents/{}/run",
            state.agent_base.trim_end_matches('/'),
            state.agent_name
        );
        info!(%url, "Forwarding refine request to agent service");

        let response = state
            .client
            .post(&url)
            .json(&json!({ "message": request.content }))
            .send()
            .await
            .map_err(|err| {
                error!(?err, "Failed to contact agent service");
                (
                    StatusCode::BAD_GATEWAY,
                    "Failed to contact agent service".to_string(),
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "<unable to read response body>".to_string());
            error!(%status, body = %text, "Agent service returned error");
            return Err((
                StatusCode::BAD_GATEWAY,
                format!("Agent service error (status {}): {}", status, text),
            ));
        }

        let agent_resp: AgentResponse = response.json().await.map_err(|err| {
            error!(?err, "Failed to deserialize agent response");
            (
                StatusCode::BAD_GATEWAY,
                "Invalid agent response".to_string(),
            )
        })?;

        return match serde_json::from_str::<AgentRefinement>(&agent_resp.output_text) {
            Ok(mut refinement) => {
                refinement.run_id = Some(agent_resp.run_id.clone());
                refinement.sanitize(&existing_ids, &existing_link_ids);
                let sanitized = to_value(refinement).map_err(|err| {
                    error!(?err, "Failed to serialize sanitized agent response");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to sanitize agent response".to_string(),
                    )
                })?;
                trace!("Sanitised resul {:?} ", sanitized);
                Ok(Json(sanitized))
            }
            Err(err) => {
                error!(?err, "Agent response failed typed parsing");
                Err((
                    StatusCode::BAD_GATEWAY,
                    "Failed to deserialise agent response".to_string(),
                ))
            }
        };
    }
}

#[cfg(feature = "stub")]
fn load_stub_response(
    existing_entity_ids: &HashSet<String>,
    existing_link_ids: &HashSet<String>,
) -> anyhow::Result<Value> {
    let mut refinement: AgentRefinement =
        serde_json::from_str(STUB_DATA).context("stub.json contains invalid refinement data")?;
    refinement.sanitize(existing_entity_ids, existing_link_ids);

    to_value(refinement).context("Failed to serialize stub refinement")
}

fn extract_entity_ids(content: &str) -> HashSet<String> {
    let regex = Regex::new(r"E\d+").expect("valid regex");
    regex
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn extract_link_ids(content: &str) -> HashSet<String> {
    let regex = Regex::new(r"L\d+").expect("valid regex");
    regex
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}
