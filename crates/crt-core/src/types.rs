use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DoraMetrics {
    pub deployment_frequency: f32,
    pub lead_time: f32,
    pub change_failure_rate: f32,
    pub mttr: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineeringMetrics {
    pub commit_frequency: f32,
    pub branch_lifetime: f32,
    pub pbis_delivered_per_sprint_per_team: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeAllocation {
    pub meetings: i32,
    pub unplanned: i32,
    pub bugs: i32,
    pub feature: i32,
    pub tech_debt: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyseRequest {
    pub crt: String,
    pub dora_metrics: DoraMetrics,
    pub extended_engineering_metrics: EngineeringMetrics,
    pub westrum: f32,
    pub time_allocation: TimeAllocation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DoraMetric {
    pub value: f32,
    pub unit: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CoreSystemicIssue {
    pub issue: String,
    pub causes: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeveragePoint {
    pub constraint: String,
    pub rationale: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisMetadata {
    pub confidence_score: String,
    pub data_completeness: String,
    pub analysis_timestamp: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisResult {
    pub executive_summary: String,
    pub core_systemic_issues: Vec<CoreSystemicIssue>,
    pub leverage_points: Vec<LeveragePoint>,
    pub systemic_relationships: Vec<String>,
    pub assumptions: Vec<String>,
    pub analysis_confidence: String,
    pub analysis_metadata: Option<AnalysisMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisResponse {
    pub run_id: String,
    pub result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluateRequest {
    pub original_payload: AnalyseRequest,
    pub analysis_result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluationResponse {
    pub run_id: String,
    pub result: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyseWithFeedbackRequest {
    pub original_payload: AnalyseRequest,
    pub analysis_result: AnalysisResult,
    pub evaluation: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefineRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefineResponse {
    pub run_id: Option<String>,
    pub output_text: String,
    pub structured_response: Option<serde_json::Value>,
}
