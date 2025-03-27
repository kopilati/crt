use std::collections::HashSet;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Make the Relationship struct wasm compatible with proper getters and setters
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Relationship {
    from: String,
    to: String,
    rel_type: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Relationship {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(from: String, to: String, rel_type: String) -> Self {
        Relationship { from, to, rel_type }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn from(&self) -> String {
        self.from.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn to(&self) -> String {
        self.to.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn rel_type(&self) -> String {
        self.rel_type.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_to(&mut self, to: String) {
        self.to = to;
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_rel_type(&mut self, rel_type: String) {
        self.rel_type = rel_type;
    }
}

// Regular Rust function for parsing content, not directly exposed to WebAssembly
pub fn parse_neo_content(content: &str) -> (HashSet<String>, Vec<Relationship>) {
    let mut nodes = HashSet::new();
    let mut relationships = Vec::new();
    
    // Add the special START node
    let start_node = "IF".to_string();
    nodes.insert(start_node.clone());
    
    let mut previous_node: Option<String> = None;
    
    // Parse the content line by line
    for line in content.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            previous_node = None;
            continue;
        }
        
        if !trimmed_line.starts_with("-") {
            // If the line doesn't start with -, treat it as a standalone node (like -if->)
            let node_text = trimmed_line.to_string();
            nodes.insert(node_text.clone());
            
            // Create a relationship from START to this node
            relationships.push(Relationship::new(
                start_node.clone(),
                node_text.clone(),
                "IF".to_string(),
            ));
            
            previous_node = Some(node_text);
        } else if line.starts_with("-if->") {
            let node_text = line[5..].trim().to_string();
            nodes.insert(node_text.clone());
            
            // Create a relationship from START to this -if-> node
            relationships.push(Relationship::new(
                start_node.clone(),
                node_text.clone(),
                "IF".to_string(), // Using a specific relationship type for START connections
            ));
            
            previous_node = Some(node_text);
        } else if line.starts_with("-then->") || line.starts_with("-and->") {
            let node_text = line[7..].trim().to_string();
            nodes.insert(node_text.clone());
            
            let rel_type = if line.starts_with("-then->") {
                "THEN"
            } else {
                "AND"
            };
            
            if let Some(prev) = &previous_node {
                relationships.push(Relationship::new(
                    prev.clone(),
                    node_text.clone(),
                    rel_type.to_string(),
                ));
            }
            
            previous_node = Some(node_text);
        }
    }
    
    (nodes, relationships)
} 