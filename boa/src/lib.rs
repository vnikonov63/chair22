pub mod instructions;
pub mod expressions;
pub mod parse;
pub mod compile;
pub mod modes;

pub use crate::modes::{generate_string_mode, eval_mode, repl_mode, file_to_expr};
