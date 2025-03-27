use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::JsValue;
use crate::parser::{parse_neo_content, Relationship};

// WebAssembly entry point for parsing content
#[wasm_bindgen]
pub fn parse_content(content: &str) -> Result<JsValue, JsValue> {
    // Parse content using the regular Rust function
    let (nodes, relationships) = parse_neo_content(content);
    
    // Convert nodes to a JavaScript array
    let nodes_array = Array::new();
    for (i, node) in nodes.iter().enumerate() {
        let node_obj = Object::new();
        let node_type = if node == "IF" { "start" } else { "normal" };
        
        Reflect::set(&node_obj, &JsValue::from_str("id"), &JsValue::from_f64(i as f64))?;
        Reflect::set(&node_obj, &JsValue::from_str("text"), &JsValue::from_str(node))?;
        Reflect::set(&node_obj, &JsValue::from_str("type"), &JsValue::from_str(node_type))?;
        
        nodes_array.push(&node_obj);
    }
    
    // Create a map from node text to index for creating links
    let node_indices: std::collections::HashMap<&String, usize> = nodes.iter()
        .enumerate()
        .map(|(i, node)| (node, i))
        .collect();
    
    // Convert relationships to a JavaScript array of links
    let links_array = Array::new();
    for rel in relationships.iter() {
        if let (Some(&source), Some(&target)) = (node_indices.get(&rel.from()), node_indices.get(&rel.to())) {
            let link_obj = Object::new();
            
            Reflect::set(&link_obj, &JsValue::from_str("source"), &JsValue::from_f64(source as f64))?;
            Reflect::set(&link_obj, &JsValue::from_str("target"), &JsValue::from_f64(target as f64))?;
            Reflect::set(&link_obj, &JsValue::from_str("type"), &JsValue::from_str(&rel.rel_type()))?;
            
            links_array.push(&link_obj);
        }
    }
    
    // Create a result object with nodes and links
    let result = Object::new();
    Reflect::set(&result, &JsValue::from_str("nodes"), &nodes_array)?;
    Reflect::set(&result, &JsValue::from_str("links"), &links_array)?;
    
    Ok(result.into())
}

// Utility function to get node count
#[wasm_bindgen]
pub fn get_node_count(content: &str) -> usize {
    let (nodes, _) = parse_neo_content(content);
    nodes.len()
}

// Utility function to get relationship count
#[wasm_bindgen]
pub fn get_relationship_count(content: &str) -> usize {
    let (_, relationships) = parse_neo_content(content);
    relationships.len()
} 