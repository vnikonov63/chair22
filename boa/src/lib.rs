pub mod instructions;
pub mod expressions;
pub mod parse;
pub mod compile;
pub mod modes;

pub use modes::{generate_string_mode, eval_mode, repl_mode};