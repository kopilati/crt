use std::collections::BTreeSet;

use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::parser::{parse_crt, Expr};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Leaf {
    id: u32,
    negated: bool,
}

fn flatten_expr(expr: &Expr) -> Vec<Leaf> {
    let mut leaves = Vec::new();
    flatten_expr_inner(expr, false, &mut leaves);
    leaves
}

fn flatten_expr_inner(expr: &Expr, mut negated: bool, out: &mut Vec<Leaf>) {
    match expr {
        Expr::EntityRef(id) => out.push(Leaf {
            id: *id,
            negated,
        }),
        Expr::Not(inner) => {
            negated = !negated;
            flatten_expr_inner(inner, negated, out);
        }
        Expr::And(items) => {
            for item in items {
                flatten_expr_inner(item, negated, out);
            }
        }
    }
}

fn push_link(
    links_array: &Array, 
    source: JsValue,
    target: JsValue,
    rel_type: &str,
    source_negated: bool,
    target_negated: bool,
) -> Result<(), JsValue> {
    let link_obj = Object::new();

    let class_type = JsValue::from_str(rel_type);
    let label = if target_negated {
        format!("{rel_type} NOT")
    } else {
        rel_type.to_string()
    };
    let negated = source_negated || target_negated;

    Reflect::set(&link_obj, &JsValue::from_str("source"), &source)?;
    Reflect::set(&link_obj, &JsValue::from_str("target"), &target)?;
    Reflect::set(&link_obj, &JsValue::from_str("type"), &class_type)?;
    Reflect::set(&link_obj, &JsValue::from_str("label"), &JsValue::from_str(&label))?;
    Reflect::set(&link_obj, &JsValue::from_str("negated"), &JsValue::from_bool(negated))?;
    Reflect::set(
        &link_obj,
        &JsValue::from_str("sourceNegated"),
        &JsValue::from_bool(source_negated),
    )?;
    Reflect::set(
        &link_obj,
        &JsValue::from_str("targetNegated"),
        &JsValue::from_bool(target_negated),
    )?;

    links_array.push(&link_obj);
    Ok(())
}

// WebAssembly entry point for parsing content
#[wasm_bindgen]
pub fn parse_content(content: &str) -> Result<JsValue, JsValue> {
    let crt = parse_crt(content).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let nodes_array = Array::new();
    let start_node = Object::new();
    Reflect::set(&start_node, &JsValue::from_str("id"), &JsValue::from_str("IF"))?;
    Reflect::set(&start_node, &JsValue::from_str("text"), &JsValue::from_str("IF"))?;
    Reflect::set(&start_node, &JsValue::from_str("type"), &JsValue::from_str("start"))?;
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
    let mut seen_if_terms: BTreeSet<Leaf> = BTreeSet::new();

    for link in crt.links.values() {
        let mut from_terms = flatten_expr(&link.from);
        let mut to_terms = flatten_expr(&link.to);

        from_terms.sort();
        from_terms.dedup();
        to_terms.sort();
        to_terms.dedup();

        for leaf in &from_terms {
            seen_if_terms.insert(leaf.clone());
        }

        let relation_type = if matches!(link.from, Expr::And(_)) {
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

    for leaf in seen_if_terms {
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

// Utility function to get node count
#[wasm_bindgen]
pub fn get_node_count(content: &str) -> usize {
    parse_crt(content)
        .map(|crt| crt.entities.len())
        .unwrap_or(0)
}

// Utility function to get relationship count
#[wasm_bindgen]
pub fn get_relationship_count(content: &str) -> usize {
    parse_crt(content)
        .map(|crt| crt.links.len())
        .unwrap_or(0)
}

