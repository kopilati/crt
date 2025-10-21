use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn, trace};

use crt_core::{
    types::*,
    validation::Validate,
    dora::*,
};

#[derive(Clone)]
struct AppState {
    agent_base_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let agent_base_url = std::env::var("AGENT_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let state = AppState { agent_base_url };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/api/analyse", post(analyse))
        .route("/api/refine", post(refine))
        .route("/api/evaluate_analysis", post(evaluate_analysis))
        .route("/api/analyse_with_feedback", post(analyse_with_feedback))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "CRT Backend API"
}

async fn analyse(
    State(state): State<AppState>,
    Json(request): Json<AnalyseRequest>,
) -> Result<Json<AnalysisResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    // Translate metrics for agent consumption
    let agent_payload = AnalyserRequest {
        crt: request.crt,
        dora_metrics: translate_dora_metrics_for_agent(&request.dora_metrics),
        extended_engineering_metrics: translate_engineering_metrics_for_agent(&request.extended_engineering_metrics),
        westrum: Some(request.westrum),
        time_allocation: request.time_allocation,
    };

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
                        executive_summary: json_value
                            .get("executive_summary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Analysis completed")
                            .to_string(),
                        core_systemic_issues: json_value
                            .get("core_systemic_issues")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        leverage_points: json_value
                            .get("leverage_points")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        systemic_relationships: json_value
                            .get("systemic_relationships")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        assumptions: json_value
                            .get("assumptions")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        analysis_confidence: json_value
                            .get("analysis_confidence")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        analysis_metadata: json_value
                            .get("analysis_metadata")
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

async fn refine(
    State(state): State<AppState>,
    Json(request): Json<RefineRequest>,
) -> Result<Json<RefineResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    let goldratt_request = GoldrattRequest {
        message: request.content,
    };
    
    let body = serde_json::to_string(&goldratt_request)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let AgentResponse {
        output_text: refinement,
        run_id: refiner_run_id,
    } = call_agent(&state, "goldratt", &body).await?;

    // Try to parse the structured response from the agent
    let structured_response = serde_json::from_str::<serde_json::Value>(&refinement).ok();

    let response = RefineResponse {
        run_id: Some(refiner_run_id),   
        output_text: refinement,
        structured_response,
    };

    Ok(Json(response))
}

async fn evaluate_analysis(
    State(state): State<AppState>,
    Json(request): Json<EvaluateRequest>,
) -> Result<Json<EvaluationResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    // Create a flattened payload for the evaluator
    let evaluator_payload = EvaluatorRequest {
        current_reality_tree: request.original_payload.crt,
        dora_metrics: translate_dora_metrics_for_agent(&request.original_payload.dora_metrics),
        extended_engineering_metrics: translate_engineering_metrics_for_agent(&request.original_payload.extended_engineering_metrics),
        westrum_score: Some(request.original_payload.westrum),
        time_allocation: request.original_payload.time_allocation,
        analysis_result: request.analysis_result,
    };

    let body = serde_json::to_string(&evaluator_payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let AgentResponse {
        output_text: evaluator_text,
        run_id: evaluator_run_id,
    } = call_agent(&state, "analysis_evaluator", &body).await?;

    let evaluation_result = match serde_json::from_str::<EvaluationResult>(&evaluator_text) {
        Ok(result) => result,
        Err(err) => {
            warn!(?err, "Evaluation output was not valid EvaluationResult JSON");
            // Fallback to a minimal evaluation result
            EvaluationResult {
                metadata: EvaluationMetadata {
                    review_timestamp: "2025-01-01T00:00:00Z".to_string(),
                    reviewer: "Analysis Reviewer Agent v2.0".to_string(),
                    analysis_version_reviewed: "unknown".to_string(),
                    review_iteration: "1".to_string(),
                },
                overall_assessment: OverallAssessment {
                    total_score: 0.0,
                    recommendation: "REJECT".to_string(),
                    confidence: "low".to_string(),
                    one_sentence_summary: "Failed to parse evaluation result".to_string(),
                },
                dimension_scores: DimensionScores {
                    causal_logic_quality: DimensionScore {
                        score: 0.0,
                        weight: "30%".to_string(),
                        weighted_score: 0.0,
                        status: "critical_issue".to_string(),
                    },
                    evidence_strength: DimensionScore {
                        score: 0.0,
                        weight: "25%".to_string(),
                        weighted_score: 0.0,
                        status: "critical_issue".to_string(),
                    },
                    constraint_identification: DimensionScore {
                        score: 0.0,
                        weight: "20%".to_string(),
                        weighted_score: 0.0,
                        status: "critical_issue".to_string(),
                    },
                    alternative_hypotheses: DimensionScore {
                        score: 0.0,
                        weight: "10%".to_string(),
                        weighted_score: 0.0,
                        status: "critical_issue".to_string(),
                    },
                    data_quality: DimensionScore {
                        score: 0.0,
                        weight: "10%".to_string(),
                        weighted_score: 0.0,
                        status: "critical_issue".to_string(),
                    },
                    completeness: DimensionScore {
                        score: 0.0,
                        weight: "5%".to_string(),
                        weighted_score: 0.0,
                        status: "critical_issue".to_string(),
                    },
                },
                critical_issues: vec![],
                logical_flaws: vec![],
                evidence_gaps: vec![],
                alternative_hypotheses: vec![],
                improvement_recommendations: vec![],
                strengths: vec![],
                validation_tests: vec![],
                data_quality_assessment: DataQualityAssessment {
                    overall_data_completeness: "0%".to_string(),
                    metric_reliability: MetricReliability {
                        dora_metrics: "unknown".to_string(),
                        extended_metrics: "unknown".to_string(),
                        cultural_metrics: "unknown".to_string(),
                    },
                    critical_data_gaps: vec![],
                    baseline_validity: "unknown".to_string(),
                },
                constraint_validation: ConstraintValidation {
                    constraint_identified: "unknown".to_string(),
                    constraint_type: "unknown".to_string(),
                    constraint_clarity: "unclear".to_string(),
                    bottleneck_evidence: "unknown".to_string(),
                    exploitation_potential: "unknown".to_string(),
                    impact_radius: "unknown".to_string(),
                    confidence_in_identification: "low".to_string(),
                    alternative_constraints_considered: "no".to_string(),
                    recommendation: "need_more_data".to_string(),
                },
                bias_assessment: BiasAssessment {
                    potential_biases_detected: vec![],
                    bias_awareness: "low".to_string(),
                },
                decision_criteria: DecisionCriteria {
                    approve_if: vec![],
                    revise_minor_if: vec![],
                    revise_major_if: vec![],
                    reject_if: vec![],
                },
                recommended_next_steps: RecommendedNextSteps {
                    if_approved: vec![],
                    if_revise_minor: vec![],
                    if_revise_major: vec![],
                    if_rejected: vec![],
                },
                review_confidence_assessment: ReviewConfidenceAssessment {
                    overall_confidence: "low".to_string(),
                    confidence_factors: ConfidenceFactors {
                        input_data_availability: "unknown".to_string(),
                        analysis_clarity: "unknown".to_string(),
                        domain_expertise: "unknown".to_string(),
                        completeness_of_review: "unknown".to_string(),
                    },
                    limitations: vec!["Failed to parse evaluation result".to_string()],
                },
            }
        }
    };

    let response = EvaluationResponse {
        run_id: evaluator_run_id,
        result: evaluation_result,
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
    let agent_payload = AnalyserWithFeedbackRequest {
        crt: request.original_payload.crt,
        dora_metrics: translate_dora_metrics_for_agent(&request.original_payload.dora_metrics),
        extended_engineering_metrics: translate_engineering_metrics_for_agent(&request.original_payload.extended_engineering_metrics),
        westrum: Some(request.original_payload.westrum),
        time_allocation: request.original_payload.time_allocation,
        analysis_result: request.analysis_result,
        evaluation: request.evaluation,
    };

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
                        executive_summary: json_value
                            .get("executive_summary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Analysis completed")
                            .to_string(),
                        core_systemic_issues: json_value
                            .get("core_systemic_issues")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        leverage_points: json_value
                            .get("leverage_points")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        systemic_relationships: json_value
                            .get("systemic_relationships")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        assumptions: json_value
                            .get("assumptions")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default(),
                        analysis_confidence: json_value
                            .get("analysis_confidence")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        analysis_metadata: json_value
                            .get("analysis_metadata")
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

#[derive(Debug, serde::Deserialize)]
struct AgentResponse {
    output_text: String,
    run_id: String,
}

async fn call_agent(
    state: &AppState,
    agent_name: &str,
    message: &str,
) -> Result<AgentResponse, (StatusCode, String)> {
    let client = reqwest::Client::new();
    let url = format!("{}/agents/{}/run", state.agent_base_url, agent_name);
    let agent_request = GoldrattRequest {
        message: message.to_string(),
    };
    let body = serde_json::to_string(&agent_request).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    trace!("Calling {} with body {}", url, body);
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        trace!("Error calling {}: {}", url, error_text);
        return Err((axum::http::StatusCode::from_u16(status.as_u16()).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), error_text));
    }

    let agent_response: AgentResponse = response
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    trace!("Agent response: {:?}", agent_response);
    Ok(agent_response)
}
