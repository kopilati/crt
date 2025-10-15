use std::{collections::HashSet, env, net::SocketAddr};

use anyhow::Context;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use crt_to_cypher::refinement::AgentRefinement;
use dotenvy::dotenv;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_value, Value};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn, trace};


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

#[derive(Debug, Deserialize, Serialize)]
struct EngineeringMetrics {
    commit_frequency: DoraCategory,
    branch_lifetime: DoraCategory,
    pbis_delivered_per_sprint_per_team: f32
}

#[derive(Debug, Deserialize, Serialize)]
enum DoraCategory {
    Low,
    Medium,
    High,
    Elite
}

#[derive(Debug, Deserialize, Serialize)]
struct TimeAllocation{
    meetings: i32,
    unplanned: i32,
    bugs: i32,
    feature: i32,
    tech_debt: i32
}

#[derive(Debug, Deserialize, Serialize)]
struct DoraMetrics{
    mttr: DoraCategory 
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyseRequest {
    crt: String,
    dora_metrics: DoraMetrics,
    extended_engineering_metrics: EngineeringMetrics,
    westrum: f32,
    time_allocation: TimeAllocation,
}

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
    
    let analyser_agent =
        env::var("ANALYSER_AGENT_NAME").unwrap_or_else(|_| "analyser".to_string());

    info!(
        %agent_base,
        %agent_name,
        %analyser_agent,
        "Configured agent service"
    );

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
        .route("/api/analyse", post(analyse))
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

    let existing_ids = existing_entity_ids;
    let url = format!(
        "{}/agents/{}/run",
        state.agent_base.trim_end_matches('/'),
        state.agent_name
    );
    info!(%url, "Forwarding refine request to agent service");

    let agent_resp = send_agent_request(state, request, url).await?;

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

async fn send_agent_request(state: AppState, request: RefineRequest, url: String) -> Result<AgentResponse, (StatusCode, String)> {
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
    Ok(agent_resp)
}

async fn analyse(
    State(state): State<AppState>,
    Json(request): Json<AnalyseRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let body = serde_json::to_string(&request)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let AgentResponse {
        output_text: analyser_text,
        run_id: analyser_run_id,
    } = call_agent(&state, "analyser", &body).await?;

    let analysis_value = match serde_json::from_str::<Value>(&analyser_text) {
        Ok(value) => value,
        Err(err) => {
            warn!(?err, "Analysis output was not valid JSON");
            Value::String(analyser_text)
        }
    };

    let response = json!({
            "run_id": analyser_run_id,
            "result": analysis_value,
    });

    Ok(Json(response))
}

fn extract_entity_ids(content: &str) -> HashSet<String> {
    let regex = Regex::new(r"E\d+").expect("valid regex");
    regex
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

async fn call_agent(
    state: &AppState,
    agent: &str,
    message: &str,
) -> Result<AgentResponse, (StatusCode, String)> {
    let url = format!(
        "{}/agents/{}/run",
        state.agent_base.trim_end_matches('/'),
        agent
    );
    info!(%url, agent, "Forwarding request to agent service");
    let response = state
        .client
        .post(&url)
        .json(&json!({ "message": message }))
        .send()
        .await
        .map_err(|err| {
            error!(?err, agent, "Failed to contact agent service");
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
        error!(%status, agent, body = %text, "Agent service returned error");
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("Agent service error (status {}): {}", status, text),
        ));
    }

    response.json::<AgentResponse>().await.map_err(|err| {
        error!(?err, agent, "Failed to deserialize agent response");
        (
            StatusCode::BAD_GATEWAY,
            "Invalid agent response".to_string(),
        )
    })
}

fn extract_link_ids(content: &str) -> HashSet<String> {
    let regex = Regex::new(r"L\d+").expect("valid regex");
    regex
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}
