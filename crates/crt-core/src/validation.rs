use crate::types::*;

pub trait Validate {
    fn validate(&self) -> Result<(), String>;
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
        self.original_payload.validate()?;
        
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
        self.original_payload.validate()?;
        
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
        
        if !self.evaluation.overall_assessment.total_score.is_finite() {
            return Err("Evaluation feedback must include a valid overall score".to_string());
        }
        Ok(())
    }
}
