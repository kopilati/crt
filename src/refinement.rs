use log::warn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRefinement {
    #[serde(rename = "CRT Restatement")]
    pub crt_restatement: CrtRestatement,
    #[serde(rename = "Leap Analysis", default)]
    pub leap_analysis: Vec<LeapAnalysisEntry>,
    #[serde(rename = "Suggested Edits", default)]
    pub suggested_edits: Vec<String>,
    #[serde(rename = "Quick Consistency Checks", default)]
    pub quick_consistency_checks: Vec<String>,
    #[serde(default, rename = "run_id")]
    pub run_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtRestatement {
    #[serde(rename = "Entities", default)]
    pub entities: Vec<CrtEntity>,
    #[serde(rename = "Links", default)]
    pub links: Vec<CrtLink>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtEntity {
    pub id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub added: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrtLink {
    pub id: String,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub added: bool,
    #[serde(default)]
    pub entities: Vec<String>,
    #[serde(default)]
    pub source_entities: Vec<String>,
    #[serde(default)]
    pub target_entities: Vec<String>,
    #[serde(default)]
    pub expressions: Vec<String>,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeapAnalysisEntry {
    #[serde(rename = "Link")]
    pub link: Option<String>,
    #[serde(rename = "CLR Finding")]
    pub clr_finding: Option<String>,
    #[serde(rename = "Why it’s a leap")]
    pub why_its_a_leap: Option<String>,
    #[serde(rename = "Bridging proposal", default)]
    pub bridging_proposal: Vec<String>,
    #[serde(rename = "Rewritten micro-chain")]
    pub rewritten_micro_chain: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl AgentRefinement {
    pub fn sanitize(
        &mut self,
        existing_entity_ids: &HashSet<String>,
        existing_link_ids: &HashSet<String>,
    ) {
        sanitize_entities_and_links(
            &mut self.crt_restatement,
            existing_entity_ids,
            existing_link_ids,
        );
    }
}

fn sanitize_entities_and_links(
    restatement: &mut CrtRestatement,
    existing_entity_ids: &HashSet<String>,
    existing_link_ids: &HashSet<String>,
) {
    let valid_entity_regex = Regex::new(r"^E\d+$").expect("valid regex");
    let mut used_ids: HashSet<String> = existing_entity_ids.clone();
    let mut next_index = used_ids
        .iter()
        .filter_map(|id| id.trim_start_matches('E').parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;

    let mut id_mapping: HashMap<String, String> = HashMap::new();

    for entity in restatement.entities.iter_mut() {
        let mut needs_new = false;
        if !valid_entity_regex.is_match(&entity.id) {
            needs_new = true;
        } else if existing_entity_ids.contains(&entity.id) {
            if entity.added {
                needs_new = true;
            }
        } else if used_ids.contains(&entity.id) {
            needs_new = true;
        }

        if needs_new {
            let old_id = entity.id.clone();
            let new_id = loop {
                let candidate = format!("E{}", next_index);
                next_index += 1;
                if !used_ids.contains(&candidate) {
                    used_ids.insert(candidate.clone());
                    break candidate;
                }
            };
            entity.id = new_id.clone();
            id_mapping.insert(old_id, new_id);
        } else {
            used_ids.insert(entity.id.clone());
        }
    }

    let known_ids = used_ids.clone();

    let mut used_link_ids: HashSet<String> = existing_link_ids.clone();
    let mut next_link_index = used_link_ids
        .iter()
        .filter_map(|id| id.trim_start_matches('L').parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let valid_link_regex = Regex::new(r"^L\d+$").expect("valid regex");

    for link in restatement.links.iter_mut() {
        if link.added {
            let original = link.id.clone();
            let new_link_id = loop {
                let candidate = format!("L{}", next_link_index);
                next_link_index += 1;
                if !used_link_ids.contains(&candidate) {
                    used_link_ids.insert(candidate.clone());
                    break candidate;
                }
            };
            if link.id != new_link_id {
                warn!(
                    "Assigning new link identifier for added link: {} -> {}",
                    original, new_link_id
                );
            }
            link.id = new_link_id;
        } else {
            if !valid_link_regex.is_match(&link.id) {
                let original = link.id.clone();
                let new_link_id = loop {
                    let candidate = format!("L{}", next_link_index);
                    next_link_index += 1;
                    if !used_link_ids.contains(&candidate) {
                        used_link_ids.insert(candidate.clone());
                        break candidate;
                    }
                };
                warn!(
                    "Renaming invalid link identifier from agent: {} -> {}",
                    original, new_link_id
                );
                link.id = new_link_id;
            }
            used_link_ids.insert(link.id.clone());
        }

        let update_refs = |value: &mut Vec<String>| {
            let mut replacements = Vec::new();
            for id_str in value.drain(..) {
                let mapped = id_mapping
                    .get(&id_str)
                    .cloned()
                    .unwrap_or_else(|| id_str.clone());
                if valid_entity_regex.is_match(&mapped) && known_ids.contains(&mapped) {
                    replacements.push(mapped);
                } else {
                    warn!(
                        "Skipping entity reference {} in agent link {}",
                        mapped, link.id
                    );
                }
            }
            value.extend(replacements);
        };

        update_refs(&mut link.entities);
        update_refs(&mut link.source_entities);
        update_refs(&mut link.target_entities);

        if let Some(from) = &mut link.from {
            if let Some(mapped) = id_mapping.get(from) {
                *from = mapped.clone();
            }
            if let Some(ref val) = link.from {
                if valid_entity_regex.is_match(val) && known_ids.contains(val) {
                    if !link.source_entities.contains(val) {
                        link.source_entities.push(val.clone());
                    }
                    if !link.entities.contains(val) {
                        link.entities.push(val.clone());
                    }
                } else {
                    warn!(
                        "Removing invalid 'from' reference {} in link {}",
                        val, link.id
                    );
                    link.from = None;
                }
            }
        }

        if let Some(to_ref) = &mut link.to {
            if let Some(mapped) = id_mapping.get(to_ref) {
                *to_ref = mapped.clone();
            }
            if let Some(ref val) = link.to {
                if valid_entity_regex.is_match(val) && known_ids.contains(val) {
                    if !link.target_entities.contains(val) {
                        link.target_entities.push(val.clone());
                    }
                    if !link.entities.contains(val) {
                        link.entities.push(val.clone());
                    }
                } else {
                    warn!(
                        "Removing invalid 'to' reference {} in link {}",
                        val, link.id
                    );
                    link.to = None;
                }
            }
        }

        dedup(&mut link.entities);
        dedup(&mut link.source_entities);
        dedup(&mut link.target_entities);

        let mut text_for_line: Option<String> = None;
        if let Some(Value::String(text)) = link.extra.get_mut("text") {
            let updated = apply_mapping(text, &id_mapping);
            *text = updated.clone();
            text_for_line = Some(updated);
        }

        if let Some(line) = link.line.as_mut() {
            *line = apply_mapping(line, &id_mapping);
        }

        if let Some(text) = text_for_line {
            link.line = Some(format!("{}. {}", link.id, text));
        } else if let Some(existing_line) = link.line.clone() {
            let suffix = existing_line
                .splitn(2, '.')
                .nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| existing_line.trim().to_string());
            link.line = Some(format!("{}. {}", link.id, suffix));
        }

        if !link.expressions.is_empty() {
            for expr in link.expressions.iter_mut() {
                *expr = apply_mapping(expr, &id_mapping);
            }
        }
    }
}

fn apply_mapping(text: &str, mapping: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (old, new) in mapping {
        let regex = Regex::new(&format!(r"\\b{}\\b", regex::escape(old))).expect("valid regex");
        result = regex.replace_all(&result, new.as_str()).to_string();
    }
    result
}

fn dedup(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|id| seen.insert(id.clone()));
}
