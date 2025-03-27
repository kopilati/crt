mod parser;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use parser::{parse_neo_content, Relationship}; 