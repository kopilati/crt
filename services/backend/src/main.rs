use std::{collections::HashSet, env, net::SocketAddr};

use anyhow::Context;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use crt_to_cypher::refinement::AgentRefinement;
use dotenvy::dotenv;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
struct RefineResponse {
    run_id: Option<String>,
    #[serde(flatten)]
    refinement: AgentRefinement,
}

#[derive(Debug, Deserialize, Serialize)]
struct EngineeringMetrics {
    commit_frequency: f32, // 0-1 scale
    branch_lifetime: f32,  // 0-1 scale
    pbis_delivered_per_sprint_per_team: f32
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
    deployment_frequency: f32,  // 0-1 scale
    lead_time: f32,            // 0-1 scale  
    change_failure_rate: f32,  // 0-1 scale
    mttr: f32,                 // 0-1 scale
}

#[derive(Debug, Deserialize, Serialize)]
struct DoraMetric {
    value: f32,
    unit: String,
}

#[derive(Debug, Clone)]
struct DoraMetricConfig {
    min_value: f32,
    max_value: f32,
    unit: &'static str,
    inverted: bool, // true if lower slider values are better (e.g., lead time, MTTR)
}

impl DoraMetricConfig {
    fn translate(&self, slider_value: f32) -> DoraMetric {
        let translated_value = if self.inverted {
            self.max_value - (self.max_value - self.min_value) * slider_value
        } else {
            self.min_value + (self.max_value - self.min_value) * slider_value
        };

        DoraMetric {
            value: (translated_value * 1000.0).round() / 1000.0, // Round to 3 decimal places
            unit: self.unit.to_string(),
        }
    }
}

// Configuration for DORA metric ranges
const DORA_METRIC_CONFIGS: &[(&str, DoraMetricConfig)] = &[
    ("deployment_frequency", DoraMetricConfig {
        min_value: 0.025,
        max_value: 10.0,
        unit: "deployments/day",
        inverted: false,
    }),
    ("lead_time", DoraMetricConfig {
        min_value: 0.1,
        max_value: 60.0,
        unit: "days",
        inverted: true,
    }),
    ("change_failure_rate", DoraMetricConfig {
        min_value: 0.0,
        max_value: 100.0,
        unit: "%",
        inverted: true,
    }),
    ("mttr", DoraMetricConfig {
        min_value: 0.05,
        max_value: 14.0,
        unit: "days",
        inverted: true,
    }),
    ("commit_frequency", DoraMetricConfig {
        min_value: 0.25,
        max_value: 10.0,
        unit: "commits/day/developer",
        inverted: false,
    }),
    ("branch_lifetime", DoraMetricConfig {
        min_value: 0.05,
        max_value: 30.0,
        unit: "days",
        inverted: true,
    }),
];

// Helper function to translate slider values to meaningful metrics for agents
fn translate_dora_metrics_for_agent(dora_metrics: &DoraMetrics) -> serde_json::Value {
    let get_config = |metric_name: &str| -> &DoraMetricConfig {
        DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == metric_name)
            .map(|(_, config)| config)
            .expect("Unknown DORA metric")
    };

    serde_json::json!({
        "deployment_frequency": get_config("deployment_frequency").translate(dora_metrics.deployment_frequency),
        "lead_time": get_config("lead_time").translate(dora_metrics.lead_time),
        "change_failure_rate": get_config("change_failure_rate").translate(dora_metrics.change_failure_rate),
        "mttr": get_config("mttr").translate(dora_metrics.mttr),
    })
}

fn translate_engineering_metrics_for_agent(engineering_metrics: &EngineeringMetrics) -> serde_json::Value {
    let get_config = |metric_name: &str| -> &DoraMetricConfig {
        DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == metric_name)
            .map(|(_, config)| config)
            .expect("Unknown DORA metric")
    };

    serde_json::json!({
        "commit_frequency": get_config("commit_frequency").translate(engineering_metrics.commit_frequency),
        "branch_lifetime": get_config("branch_lifetime").translate(engineering_metrics.branch_lifetime),
        "pbis_delivered_per_sprint_per_team": engineering_metrics.pbis_delivered_per_sprint_per_team,
    })
}

trait Validate {
    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyseRequest {
    crt: String,
    dora_metrics: DoraMetrics,
    extended_engineering_metrics: EngineeringMetrics,
    westrum: f32,
    time_allocation: TimeAllocation,
}

impl Validate for AnalyseRequest {
    fn validate(&self) -> Result<(), String> {
        if self.crt.is_empty() {
            return Err("CRT is required".to_string());
        }
        if self.dora_metrics.deployment_frequency < 0.0 || self.dora_metrics.deployment_frequency > 1.0 {
            return Err("Deployment frequency must be between 0 and 1".to_string());
        }
        if self.dora_metrics.lead_time < 0.0 || self.dora_metrics.lead_time > 1.0 {
            return Err("Lead time must be between 0 and 1".to_string());
        }
        if self.dora_metrics.change_failure_rate < 0.0 || self.dora_metrics.change_failure_rate > 1.0 {
            return Err("Change failure rate must be between 0 and 1".to_string());
        }
        if self.dora_metrics.mttr < 0.0 || self.dora_metrics.mttr > 1.0 {
            return Err("MTTR must be between 0 and 1".to_string());
        }
        if self.extended_engineering_metrics.commit_frequency < 0.0 || self.extended_engineering_metrics.commit_frequency > 1.0 {
            return Err("Commit frequency must be between 0 and 1".to_string());
        }
        if self.extended_engineering_metrics.branch_lifetime < 0.0 || self.extended_engineering_metrics.branch_lifetime > 1.0 {
            return Err("Branch lifetime must be between 0 and 1".to_string());
        }
        if self.extended_engineering_metrics.pbis_delivered_per_sprint_per_team < 0.0 || self.extended_engineering_metrics.pbis_delivered_per_sprint_per_team > 1.0 {
            return Err("PBIs delivered per sprint per team must be between 0 and 1".to_string());
        }
        if self.westrum < 0.0 || self.westrum > 7.0 {
            return Err("Westrum must be between 0 and 7".to_string());
        }
        if self.time_allocation.meetings < 0 || self.time_allocation.unplanned < 0 || self.time_allocation.bugs < 0 || self.time_allocation.feature < 0 || self.time_allocation.tech_debt < 0 {
            return Err("Time allocation must be greater than 0".to_string());
        }
        if self.time_allocation.meetings + self.time_allocation.unplanned + self.time_allocation.bugs + self.time_allocation.feature + self.time_allocation.tech_debt != 100 {
            return Err("Time allocation must sum to 100".to_string());
        } 
        Ok(())
    }
}

impl Validate for RefineRequest {
    fn validate(&self) -> Result<(), String> {
        if self.content.trim().is_empty() {
            return Err("Content must not be empty".to_string());
        }
        if self.content.len() > 100_000 {
            return Err("Content is too large (max 100,000 characters)".to_string());
        }
        Ok(())
    }
}

impl Validate for EvaluateRequest {
    fn validate(&self) -> Result<(), String> {
        // Validate the original payload
        self.original_payload.validate()?;
        
        // Validate analysis result has required fields
        if self.analysis_result.executive_summary.is_empty() {
            return Err("Analysis result must have an executive summary".to_string());
        }
        if self.analysis_result.core_systemic_issues.is_empty() {
            return Err("Analysis result must have at least one core systemic issue".to_string());
        }
        if self.analysis_result.leverage_points.is_empty() {
            return Err("Analysis result must have at least one leverage point".to_string());
        }
        if self.analysis_result.analysis_confidence.is_empty() {
            return Err("Analysis result must have analysis confidence".to_string());
        }
        Ok(())
    }
}

impl Validate for AnalyseWithFeedbackRequest {
    fn validate(&self) -> Result<(), String> {
        // Validate the original payload
        self.original_payload.validate()?;
        
        // Validate analysis result has required fields
        if self.analysis_result.executive_summary.is_empty() {
            return Err("Analysis result must have an executive summary".to_string());
        }
        if self.analysis_result.core_systemic_issues.is_empty() {
            return Err("Analysis result must have at least one core systemic issue".to_string());
        }
        if self.analysis_result.leverage_points.is_empty() {
            return Err("Analysis result must have at least one leverage point".to_string());
        }
        if self.analysis_result.analysis_confidence.is_empty() {
            return Err("Analysis result must have analysis confidence".to_string());
        }
        
        // Validate evaluation is not empty (it should contain user feedback)
        if self.evaluation.is_null() {
            return Err("Evaluation feedback is required".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct AgentResponse {
    output_text: String,
    run_id: String,
}

#[derive(Debug, Serialize)]
struct AgentRequest {
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalysisResult {
    executive_summary: String,
    core_systemic_issues: Vec<CoreSystemicIssue>,
    leverage_points: Vec<LeveragePoint>,
    systemic_relationships: Vec<String>,
    assumptions: Vec<String>,
    analysis_confidence: String,
    analysis_metadata: Option<AnalysisMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CoreSystemicIssue {
    issue: String,
    causes: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LeveragePoint {
    constraint: String,
    rationale: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalysisMetadata {
    confidence_score: String,
    data_completeness: String,
    analysis_timestamp: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct EvaluateRequest {
    original_payload: AnalyseRequest,
    analysis_result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyseWithFeedbackRequest {
    original_payload: AnalyseRequest,
    analysis_result: AnalysisResult,
    evaluation: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalysisResponse {
    run_id: String,
    result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
struct EvaluationResponse {
    run_id: String,
    result: serde_json::Value,
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

    let evaluator_agent =
        env::var("ANALYSIS_EVALUATOR_AGENT_NAME").unwrap_or_else(|_| "analysis_evaluator".to_string());

    info!(
        %agent_base,
        %agent_name,
        %analyser_agent,
        %evaluator_agent,
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
        .route("/api/evaluate_analysis", post(evaluate_analysis))
        .route("/api/analyse_with_feedback", post(analyse_with_feedback))
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
) -> Result<Json<RefineResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    
    let payload = request.content.trim();
    let existing_entity_ids = extract_entity_ids(payload);
    let existing_link_ids = extract_link_ids(payload);

    let existing_ids = existing_entity_ids;
    let url = format!(
        "{}/agents/{}/run",
        state.agent_base.trim_end_matches('/'),
        state.agent_name
    );
    info!(%url, "Forwarding refine request to agent service");

    let agent_resp = call_agent(&state, "goldratt", payload).await?;

    return match serde_json::from_str::<AgentRefinement>(&agent_resp.output_text) {
        Ok(mut refinement) => {
            refinement.run_id = Some(agent_resp.run_id.clone());
            refinement.sanitize(&existing_ids, &existing_link_ids);
            
            let response = RefineResponse {
                run_id: refinement.run_id.clone(),
                refinement,
            };
            
            trace!("Sanitised result {:?} ", response);
            Ok(Json(response))
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

async fn analyse(
    State(state): State<AppState>,
    Json(request): Json<AnalyseRequest>,
) -> Result<Json<AnalysisResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    
    // Translate metrics for agent consumption
    let agent_payload = serde_json::json!({
        "crt": request.crt,
        "dora_metrics": translate_dora_metrics_for_agent(&request.dora_metrics),
        "extended_engineering_metrics": translate_engineering_metrics_for_agent(&request.extended_engineering_metrics),
        "westrum": request.westrum,
        "time_allocation": request.time_allocation,
    });

    let body = serde_json::to_string(&agent_payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let AgentResponse {
        output_text: analyser_text,
        run_id: analyser_run_id,
    } = call_agent(&state, "analyser", &body).await?;

    let analysis_result = match serde_json::from_str::<AnalysisResult>(&analyser_text) {
        Ok(result) => result,
        Err(err) => {
            warn!(?err, "Analysis output was not valid AnalysisResult JSON");
            // Try to parse as Value and extract fields manually
            match serde_json::from_str::<serde_json::Value>(&analyser_text) {
                Ok(json_value) => {
                    AnalysisResult {
                        executive_summary: json_value.get("executive_summary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Analysis completed")
                            .to_string(),
                        core_systemic_issues: json_value.get("core_systemic_issues")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        leverage_points: json_value.get("leverage_points")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        systemic_relationships: json_value.get("systemic_relationships")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        assumptions: json_value.get("assumptions")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        analysis_confidence: json_value.get("analysis_confidence")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        analysis_metadata: json_value.get("analysis_metadata")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    }
                }
                Err(_) => {
                    // Complete fallback
                    AnalysisResult {
                        executive_summary: analyser_text.clone(),
                        core_systemic_issues: vec![],
                        leverage_points: vec![],
                        systemic_relationships: vec![],
                        assumptions: vec![],
                        analysis_confidence: "Unknown".to_string(),
                        analysis_metadata: None,
                    }
                }
            }
        }
    };

    let response = AnalysisResponse {
        run_id: analyser_run_id,
        result: analysis_result,
    };

    Ok(Json(response))
}

async fn evaluate_analysis(
    State(state): State<AppState>,
    Json(request): Json<EvaluateRequest>,
) -> Result<Json<EvaluationResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    
    // Create the evaluator payload with translated metrics for agent consumption
    let evaluator_payload = serde_json::json!({
        "current_reality_tree": request.original_payload.crt,
        "dora_metrics": translate_dora_metrics_for_agent(&request.original_payload.dora_metrics),
        "extended_engineering_metrics": translate_engineering_metrics_for_agent(&request.original_payload.extended_engineering_metrics),
        "westrum_score": request.original_payload.westrum,
        "time_allocation": request.original_payload.time_allocation,
        "analysis_result": request.analysis_result,
    });

    let body = serde_json::to_string(&evaluator_payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let AgentResponse { output_text, run_id } = call_agent(&state, "analysis_evaluator", &body).await?;

    let evaluation_value = match serde_json::from_str::<serde_json::Value>(&output_text) {
        Ok(value) => value,
        Err(_) => serde_json::Value::String(output_text),
    };

    let response = EvaluationResponse {
        run_id,
        result: evaluation_value,
    };
    Ok(Json(response))
}

async fn analyse_with_feedback(
    State(state): State<AppState>,
    Json(request): Json<AnalyseWithFeedbackRequest>,
) -> Result<Json<AnalysisResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    
    // Translate metrics for agent consumption
    let agent_payload = serde_json::json!({
        "crt": request.original_payload.crt,
        "dora_metrics": translate_dora_metrics_for_agent(&request.original_payload.dora_metrics),
        "extended_engineering_metrics": translate_engineering_metrics_for_agent(&request.original_payload.extended_engineering_metrics),
        "westrum": request.original_payload.westrum,
        "time_allocation": request.original_payload.time_allocation,
        "analysis_result": request.analysis_result,
        "evaluation": request.evaluation,
    });

    let body = serde_json::to_string(&agent_payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let AgentResponse { output_text, run_id } = call_agent(&state, "analyser", &body).await?;

    let analysis_result = match serde_json::from_str::<AnalysisResult>(&output_text) {
        Ok(result) => result,
        Err(err) => {
            warn!(?err, "Analysis output was not valid AnalysisResult JSON");
            // Try to parse as Value and extract fields manually
            match serde_json::from_str::<serde_json::Value>(&output_text) {
                Ok(json_value) => {
                    AnalysisResult {
                        executive_summary: json_value.get("executive_summary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Analysis completed")
                            .to_string(),
                        core_systemic_issues: json_value.get("core_systemic_issues")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        leverage_points: json_value.get("leverage_points")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        systemic_relationships: json_value.get("systemic_relationships")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        assumptions: json_value.get("assumptions")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_else(|| vec![]),
                        analysis_confidence: json_value.get("analysis_confidence")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        analysis_metadata: json_value.get("analysis_metadata")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()),
                    }
                }
                Err(_) => {
                    // Complete fallback
                    AnalysisResult {
                        executive_summary: output_text.clone(),
                        core_systemic_issues: vec![],
                        leverage_points: vec![],
                        systemic_relationships: vec![],
                        assumptions: vec![],
                        analysis_confidence: "Unknown".to_string(),
                        analysis_metadata: None,
                    }
                }
            }
        }
    };

    let response = AnalysisResponse {
        run_id,
        result: analysis_result,
    };
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
    let agent_request = AgentRequest {
        message: message.to_string(),
    };
    let response = state
        .client
        .post(&url)
        .json(&agent_request)
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

#[cfg(test)]
mod tests;
