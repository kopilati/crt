use crate::types::{DoraMetric, DoraMetrics, EngineeringMetrics};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DoraMetricConfig {
    pub min_value: f32,
    pub max_value: f32,
    pub unit: &'static str,
    pub inverted: bool,
}

impl DoraMetricConfig {
    pub fn translate(&self, slider_value: f32) -> DoraMetric {
        let translated_value = if self.inverted {
            self.max_value - (self.max_value - self.min_value) * slider_value
        } else {
            self.min_value + (self.max_value - self.min_value) * slider_value
        };

        // Format value based on the unit
        let formatted_value = if self.unit == "%" {
            // For percentages, round to nearest integer but keep as f32
            translated_value.round()
        } else {
            // For days, show 2 decimal places
            (translated_value * 1000.0).round() / 1000.0
        };

        DoraMetric {
            value: formatted_value,
            unit: self.unit.to_string(),
        }
    }
}

pub const DORA_METRIC_CONFIGS: &[(&str, DoraMetricConfig)] = &[
    ("deployment_frequency", DoraMetricConfig {
        min_value: 0.001,
        max_value: 10.0,
        unit: "deployments/day",
        inverted: false,
    }),
    ("lead_time", DoraMetricConfig {
        min_value: 0.04,
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
        min_value: 0.0125,
        max_value: 14.0,
        unit: "days",
        inverted: true,
    }),
    ("commit_frequency", DoraMetricConfig {
        min_value: 0.0625,
        max_value: 10.0,
        unit: "commits/day per developer",
        inverted: false,
    }),
    ("branch_lifetime", DoraMetricConfig {
        min_value: 0.0125,
        max_value: 30.0,
        unit: "days",
        inverted: true,
    }),
];

pub fn translate_dora_metrics_for_agent(dora_metrics: &DoraMetrics) -> HashMap<String, DoraMetric> {
    let get_config = |metric_name: &str| -> &DoraMetricConfig {
        DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == metric_name)
            .map(|(_, config)| config)
            .expect("Unknown DORA metric")
    };

    let mut result = HashMap::new();
    result.insert("deployment_frequency".to_string(), get_config("deployment_frequency").translate(dora_metrics.deployment_frequency));
    result.insert("lead_time".to_string(), get_config("lead_time").translate(dora_metrics.lead_time));
    result.insert("change_failure_rate".to_string(), get_config("change_failure_rate").translate(dora_metrics.change_failure_rate));
    result.insert("mttr".to_string(), get_config("mttr").translate(dora_metrics.mttr));
    result
}

pub fn translate_engineering_metrics_for_agent(engineering_metrics: &EngineeringMetrics) -> HashMap<String, DoraMetric> {
    let get_config = |metric_name: &str| -> &DoraMetricConfig {
        DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == metric_name)
            .map(|(_, config)| config)
            .expect("Unknown DORA metric")
    };

    let mut result = HashMap::new();
    result.insert("commit_frequency".to_string(), get_config("commit_frequency").translate(engineering_metrics.commit_frequency));
    result.insert("branch_lifetime".to_string(), get_config("branch_lifetime").translate(engineering_metrics.branch_lifetime));
    result.insert("pbis_delivered_per_sprint_per_team".to_string(), DoraMetric {
        value: engineering_metrics.pbis_delivered_per_sprint_per_team,
        unit: "PBIs/sprint/team".to_string(),
    });
    result
}
