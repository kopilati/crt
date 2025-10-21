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
    pub result: EvaluationResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyseWithFeedbackRequest {
    pub original_payload: AnalyseRequest,
    pub analysis_result: AnalysisResult,
    pub evaluation: EvaluationResult,
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

// Evaluation Response Types (based on analysis_evaluator.json schema)
#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluationMetadata {
    pub review_timestamp: String,
    pub reviewer: String,
    pub analysis_version_reviewed: String,
    pub review_iteration: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OverallAssessment {
    pub total_score: f64,
    pub recommendation: String,
    pub confidence: String,
    pub one_sentence_summary: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DimensionScore {
    pub score: f64,
    pub weight: String,
    pub weighted_score: f64,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DimensionScores {
    pub causal_logic_quality: DimensionScore,
    pub evidence_strength: DimensionScore,
    pub constraint_identification: DimensionScore,
    pub alternative_hypotheses: DimensionScore,
    pub data_quality: DimensionScore,
    pub completeness: DimensionScore,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CriticalIssue {
    pub issue_id: String,
    pub dimension: String,
    pub severity: String,
    pub issue: String,
    pub evidence: String,
    pub impact: String,
    pub recommendation: String,
    #[serde(default)]
    pub example: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogicalFlaw {
    pub flaw_id: String,
    pub r#type: String,
    pub location: String,
    pub description: String,
    pub why_it_matters: String,
    pub suggested_fix: String,
    pub validation_test: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvidenceGap {
    pub gap_id: String,
    pub claim: String,
    pub current_evidence: String,
    pub gap_type: String,
    pub impact: String,
    pub recommended_evidence: String,
    #[serde(default)]
    pub workaround: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlternativeHypothesis {
    pub hypothesis_id: String,
    pub alternative_explanation: String,
    pub supporting_evidence: String,
    pub how_to_test: String,
    #[serde(default)]
    pub if_true_impact: String,
    #[serde(default)]
    pub analysis_coverage: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImprovementRecommendation {
    pub rec_id: String,
    pub dimension: String,
    pub priority: String,
    pub current_state: String,
    pub proposed_change: String,
    pub rationale: String,
    #[serde(default)]
    pub expected_impact: String,
    #[serde(default)]
    pub effort: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Strength {
    pub strength: String,
    pub dimension: String,
    pub why_it_matters: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidationTest {
    pub test_id: String,
    pub purpose: String,
    pub test_description: String,
    pub expected_result_if_analysis_correct: String,
    pub expected_result_if_analysis_wrong: String,
    #[serde(default)]
    pub effort: String,
    #[serde(default)]
    pub when_to_run: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetricReliability {
    pub dora_metrics: String,
    pub extended_metrics: String,
    pub cultural_metrics: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CriticalDataGap {
    pub metric: String,
    pub impact: String,
    pub mitigation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataQualityAssessment {
    pub overall_data_completeness: String,
    pub metric_reliability: MetricReliability,
    #[serde(default)]
    pub critical_data_gaps: Vec<CriticalDataGap>,
    pub baseline_validity: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConstraintValidation {
    pub constraint_identified: String,
    pub constraint_type: String,
    pub constraint_clarity: String,
    pub bottleneck_evidence: String,
    pub exploitation_potential: String,
    pub impact_radius: String,
    pub confidence_in_identification: String,
    pub alternative_constraints_considered: String,
    pub recommendation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PotentialBias {
    pub bias_type: String,
    pub evidence_of_bias: String,
    pub impact: String,
    pub mitigation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BiasAssessment {
    #[serde(default)]
    pub potential_biases_detected: Vec<PotentialBias>,
    pub bias_awareness: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DecisionCriteria {
    #[serde(default)]
    pub approve_if: Vec<String>,
    #[serde(default)]
    pub revise_minor_if: Vec<String>,
    #[serde(default)]
    pub revise_major_if: Vec<String>,
    #[serde(default)]
    pub reject_if: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecommendedNextSteps {
    #[serde(default)]
    pub if_approved: Vec<String>,
    #[serde(default)]
    pub if_revise_minor: Vec<String>,
    #[serde(default)]
    pub if_revise_major: Vec<String>,
    #[serde(default)]
    pub if_rejected: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfidenceFactors {
    pub input_data_availability: String,
    pub analysis_clarity: String,
    pub domain_expertise: String,
    pub completeness_of_review: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReviewConfidenceAssessment {
    pub overall_confidence: String,
    pub confidence_factors: ConfidenceFactors,
    #[serde(default)]
    pub limitations: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluationResult {
    pub metadata: EvaluationMetadata,
    pub overall_assessment: OverallAssessment,
    pub dimension_scores: DimensionScores,
    #[serde(default)]
    pub critical_issues: Vec<CriticalIssue>,
    #[serde(default)]
    pub logical_flaws: Vec<LogicalFlaw>,
    #[serde(default)]
    pub evidence_gaps: Vec<EvidenceGap>,
    #[serde(default)]
    pub alternative_hypotheses: Vec<AlternativeHypothesis>,
    #[serde(default)]
    pub improvement_recommendations: Vec<ImprovementRecommendation>,
    #[serde(default)]
    pub strengths: Vec<Strength>,
    #[serde(default)]
    pub validation_tests: Vec<ValidationTest>,
    pub data_quality_assessment: DataQualityAssessment,
    pub constraint_validation: ConstraintValidation,
    pub bias_assessment: BiasAssessment,
    pub decision_criteria: DecisionCriteria,
    pub recommended_next_steps: RecommendedNextSteps,
    pub review_confidence_assessment: ReviewConfidenceAssessment,
}

// Agent Request Types
#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyserRequest {
    pub crt: String,
    pub dora_metrics: std::collections::HashMap<String, DoraMetric>,
    pub extended_engineering_metrics: std::collections::HashMap<String, DoraMetric>,
    pub westrum: Option<f32>,
    pub time_allocation: TimeAllocation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyserWithFeedbackRequest {
    pub crt: String,
    pub dora_metrics: std::collections::HashMap<String, DoraMetric>,
    pub extended_engineering_metrics: std::collections::HashMap<String, DoraMetric>,
    pub westrum: Option<f32>,
    pub time_allocation: TimeAllocation,
    pub analysis_result: AnalysisResult,
    pub evaluation: EvaluationResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluatorRequest {
    pub current_reality_tree: String,
    pub dora_metrics: std::collections::HashMap<String, DoraMetric>,
    pub extended_engineering_metrics: std::collections::HashMap<String, DoraMetric>,
    pub westrum_score: Option<f32>,
    pub time_allocation: TimeAllocation,
    pub analysis_result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoldrattRequest {
    pub message: String,
}
