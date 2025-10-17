#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use js_sys::{Array, Object, Reflect};
#[cfg(feature = "wasm")]
use crate::types::*;
#[cfg(feature = "wasm")]
use crate::validation::Validate;
#[cfg(feature = "wasm")]
use crate::dora::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmAnalyseRequest {
    inner: AnalyseRequest,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmAnalyseRequest {
    #[wasm_bindgen(constructor)]
    pub fn new(
        crt: String,
        deployment_frequency: f32,
        lead_time: f32,
        change_failure_rate: f32,
        mttr: f32,
        commit_frequency: f32,
        branch_lifetime: f32,
        pbis_delivered_per_sprint_per_team: f32,
        meetings: i32,
        unplanned: i32,
        bugs: i32,
        feature: i32,
        tech_debt: i32,
        westrum: Option<f32>,
    ) -> WasmAnalyseRequest {
        WasmAnalyseRequest {
            inner: AnalyseRequest {
                crt,
                dora_metrics: DoraMetrics {
                    deployment_frequency,
                    lead_time,
                    change_failure_rate,
                    mttr,
                },
                extended_engineering_metrics: EngineeringMetrics {
                    commit_frequency,
                    branch_lifetime,
                    pbis_delivered_per_sprint_per_team,
                },
                westrum: westrum.unwrap_or(0.0),
                time_allocation: TimeAllocation {
                    meetings,
                    unplanned,
                    bugs,
                    feature,
                    tech_debt,
                },
            },
        }
    }

    #[wasm_bindgen]
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner.validate().map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn translate_dora_metric(metric_name: &str, slider_value: f32) -> Result<JsValue, JsValue> {
    let config = DORA_METRIC_CONFIGS
        .iter()
        .find(|(name, _)| *name == metric_name)
        .map(|(_, config)| config)
        .ok_or_else(|| JsValue::from_str(&format!("Unknown metric: {}", metric_name)))?;

    let result = config.translate(slider_value);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_dora_metric_config(metric_name: &str) -> Result<JsValue, JsValue> {
    let config = DORA_METRIC_CONFIGS
        .iter()
        .find(|(name, _)| *name == metric_name)
        .map(|(_, config)| config)
        .ok_or_else(|| JsValue::from_str(&format!("Unknown metric: {}", metric_name)))?;

    let config_info = serde_json::json!({
        "min_value": config.min_value,
        "max_value": config.max_value,
        "unit": config.unit,
        "inverted": config.inverted,
    });

    Ok(serde_wasm_bindgen::to_value(&config_info)?)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn validate_analyse_request_json(json_str: &str) -> Result<bool, JsValue> {
    match serde_json::from_str::<AnalyseRequest>(json_str) {
        Ok(request) => {
            match request.validate() {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_all_dora_metric_configs() -> Result<JsValue, JsValue> {
    let configs: std::collections::HashMap<String, serde_json::Value> = DORA_METRIC_CONFIGS
        .iter()
        .map(|(name, config)| {
            let config_info = serde_json::json!({
                "min_value": config.min_value,
                "max_value": config.max_value,
                "unit": config.unit,
                "inverted": config.inverted,
            });
            (name.to_string(), config_info)
        })
        .collect();
    
    Ok(serde_wasm_bindgen::to_value(&configs)?)
}

// Business Logic Functions

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn clamp_percentage(value: f32) -> i32 {
    if value.is_nan() {
        return 0;
    }
    value.max(0.0).min(100.0).round() as i32
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn to_int_percentage(value: f32) -> i32 {
    if value.is_nan() {
        return 0;
    }
    value.round() as i32
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn is_time_allocation_valid(meetings: i32, unplanned: i32, bugs: i32, feature: i32, tech_debt: i32) -> bool {
    let total = meetings + unplanned + bugs + feature + tech_debt;
    (total - 100).abs() <= 1 // Allow 1% tolerance
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn is_dora_complete(deployment_frequency: Option<f32>, lead_time: Option<f32>, 
                       change_failure_rate: Option<f32>, mttr: Option<f32>) -> bool {
    deployment_frequency.is_some() && lead_time.is_some() && 
    change_failure_rate.is_some() && mttr.is_some()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn is_westrum_provided(westrum_score: Option<f32>) -> bool {
    westrum_score.is_some() && westrum_score.unwrap().is_finite()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn is_analysis_ready(crt_content: &str, deployment_frequency: Option<f32>, 
                        lead_time: Option<f32>, change_failure_rate: Option<f32>, 
                        mttr: Option<f32>, meetings: i32, unplanned: i32, 
                        bugs: i32, feature: i32, tech_debt: i32, 
                        westrum_score: Option<f32>) -> bool {
    // Check CRT content
    if crt_content.trim().is_empty() {
        return false;
    }
    
    // Check DORA metrics
    if !is_dora_complete(deployment_frequency, lead_time, change_failure_rate, mttr) {
        return false;
    }
    
    // Check time allocation
    if !is_time_allocation_valid(meetings, unplanned, bugs, feature, tech_debt) {
        return false;
    }
    
    // Check Westrum score
    if !is_westrum_provided(westrum_score) {
        return false;
    }
    
    true
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_westrum_descriptor(score: f32) -> String {
    if !score.is_finite() {
        return "Unknown".to_string();
    }
    
    let rounded = score.round() as i32;
    match rounded {
        1 => "Pathological".to_string(),
        2 => "Bureaucratic".to_string(), 
        3 => "Generative".to_string(),
        4 => "Performance".to_string(),
        5 => "Learning".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn to_dora_category_name(value: f32) -> String {
    if value <= 0.25 {
        "Elite".to_string()
    } else if value <= 0.5 {
        "High".to_string()
    } else if value <= 0.75 {
        "Medium".to_string()
    } else {
        "Low".to_string()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn validate_refine_response_json(json_str: &str) -> Result<bool, JsValue> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(response) => {
            // Check if it has the expected structure for refine response
            let has_restatement = response.get("CRT Restatement").is_some();
            let has_entities = response.get("CRT Restatement")
                .and_then(|r| r.get("Entities"))
                .and_then(|e| e.as_array())
                .map_or(false, |arr| !arr.is_empty());
            let has_links = response.get("CRT Restatement")
                .and_then(|r| r.get("Links"))
                .and_then(|l| l.as_array())
                .map_or(false, |arr| !arr.is_empty());
            
            Ok(has_restatement && has_entities && has_links)
        }
        Err(_) => Ok(false),
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn extract_refine_entities(json_str: &str) -> Result<JsValue, JsValue> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(response) => {
            if let Some(entities) = response.get("CRT Restatement")
                .and_then(|r| r.get("Entities"))
                .and_then(|e| e.as_array()) {
                
                let added_entities: Vec<_> = entities.iter()
                    .filter(|entity| entity.get("added").and_then(|a| a.as_bool()).unwrap_or(false))
                    .collect();
                
                Ok(serde_wasm_bindgen::to_value(&added_entities)?)
            } else {
                Ok(serde_wasm_bindgen::to_value(&Vec::<serde_json::Value>::new())?)
            }
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn extract_refine_links(json_str: &str) -> Result<JsValue, JsValue> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(response) => {
            if let Some(links) = response.get("CRT Restatement")
                .and_then(|r| r.get("Links"))
                .and_then(|l| l.as_array()) {
                
                let added_links: Vec<_> = links.iter()
                    .filter(|link| link.get("added").and_then(|a| a.as_bool()).unwrap_or(false))
                    .collect();
                
                Ok(serde_wasm_bindgen::to_value(&added_links)?)
            } else {
                Ok(serde_wasm_bindgen::to_value(&Vec::<serde_json::Value>::new())?)
            }
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_content(content: &str) -> Result<JsValue, JsValue> {
    use crate::parser::{parse_crt, Expr};
    use std::collections::BTreeSet;
    use js_sys::{Array, Object, Reflect};

    let crt = parse_crt(content).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let nodes_array = Array::new();
    let start_node = Object::new();
    Reflect::set(
        &start_node,
        &JsValue::from_str("id"),
        &JsValue::from_str("IF"),
    )?;
    Reflect::set(
        &start_node,
        &JsValue::from_str("text"),
        &JsValue::from_str("IF"),
    )?;
    Reflect::set(
        &start_node,
        &JsValue::from_str("type"),
        &JsValue::from_str("start"),
    )?;
    nodes_array.push(&start_node);

    for entity in crt.entities.values() {
        let node_obj = Object::new();
        Reflect::set(
            &node_obj,
            &JsValue::from_str("id"),
            &JsValue::from_f64(entity.id as f64),
        )?;
        Reflect::set(
            &node_obj,
            &JsValue::from_str("text"),
            &JsValue::from_str(&entity.text),
        )?;
        Reflect::set(
            &node_obj,
            &JsValue::from_str("type"),
            &JsValue::from_str("normal"),
        )?;
        nodes_array.push(&node_obj);
    }

    let links_array = Array::new();
    let mut source_terms: BTreeSet<Leaf> = BTreeSet::new();
    let mut target_terms: BTreeSet<Leaf> = BTreeSet::new();

    for link in crt.links.values() {
        if link.segments.len() < 2 {
            continue;
        }

        for window in link.segments.windows(2) {
            let source_expr = &window[0];
            let target_expr = &window[1];

            let mut from_terms = flatten_expr(source_expr);
            let mut to_terms = flatten_expr(target_expr);

            from_terms.sort();
            from_terms.dedup();
            to_terms.sort();
            to_terms.dedup();

            from_terms.iter().for_each(|leaf| {
                source_terms.insert(leaf.clone());
            });
            to_terms.iter().for_each(|leaf| {
                target_terms.insert(leaf.clone());
            });

            let relation_type = if matches!(source_expr, Expr::And(_)) {
                "AND"
            } else {
                "THEN"
            };

            for source in &from_terms {
                let source_id = JsValue::from_f64(source.id as f64);
                for target in &to_terms {
                    let target_id = JsValue::from_f64(target.id as f64);
                    push_link(
                        &links_array,
                        source_id.clone(),
                        target_id,
                        relation_type,
                        source.negated,
                        target.negated,
                    )?;
                }
            }
        }
    }

    for leaf in source_terms.iter() {
        if target_terms.contains(leaf) {
            continue;
        }
        push_link(
            &links_array,
            JsValue::from_str("IF"),
            JsValue::from_f64(leaf.id as f64),
            "IF",
            false,
            leaf.negated,
        )?;
    }

    let result = Object::new();
    Reflect::set(&result, &JsValue::from_str("nodes"), &nodes_array)?;
    Reflect::set(&result, &JsValue::from_str("links"), &links_array)?;

    Ok(result.into())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Leaf {
    id: u32,
    negated: bool,
}

fn flatten_expr(expr: &crate::parser::Expr) -> Vec<Leaf> {
    let mut leaves = Vec::new();
    flatten_expr_inner(expr, false, &mut leaves);
    leaves
}

fn flatten_expr_inner(expr: &crate::parser::Expr, negated: bool, leaves: &mut Vec<Leaf>) {
    match expr {
        crate::parser::Expr::EntityRef(id) => {
            leaves.push(Leaf {
                id: *id,
                negated,
            });
        }
        crate::parser::Expr::Not(inner) => {
            flatten_expr_inner(inner, !negated, leaves);
        }
        crate::parser::Expr::And(items) => {
            for item in items {
                flatten_expr_inner(item, negated, leaves);
            }
        }
    }
}

fn push_link(
    links_array: &Array,
    source: JsValue,
    target: JsValue,
    relation_type: &str,
    source_negated: bool,
    target_negated: bool,
) -> Result<(), JsValue> {
    let link_obj = Object::new();
    Reflect::set(&link_obj, &JsValue::from_str("source"), &source)?;
    Reflect::set(&link_obj, &JsValue::from_str("target"), &target)?;
    Reflect::set(
        &link_obj,
        &JsValue::from_str("type"),
        &JsValue::from_str(relation_type),
    )?;
    Reflect::set(
        &link_obj,
        &JsValue::from_str("source_negated"),
        &JsValue::from_bool(source_negated),
    )?;
    Reflect::set(
        &link_obj,
        &JsValue::from_str("target_negated"),
        &JsValue::from_bool(target_negated),
    )?;
    links_array.push(&link_obj);
    Ok(())
}

// Utility function to get node count
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_node_count(content: &str) -> usize {
    use crate::parser::parse_crt;
    parse_crt(content)
        .map(|crt| crt.entities.len())
        .unwrap_or(0)
}

// Utility function to get relationship count
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_relationship_count(content: &str) -> usize {
    use crate::parser::parse_crt;
    parse_crt(content).map(|crt| crt.links.len()).unwrap_or(0)
}
