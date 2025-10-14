mod parser;
pub mod refinement;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use parser::{parse_crt, Expr, Link, Relationship, CRT};
