use std::collections::HashMap;
use std::io;
use std::io::prelude::*;

use sexp::*;

use crate::parse::{parse_repl_expr};
use crate::compile_jit::{compile_repl_to_instr, compile_jit_and_persist};
use crate::expressions::{ReplExpr};

fn format_viva_value(val: i64) -> String {
    if val == 3 {
        "true".to_string()
    } else if val == 1 {
        "false".to_string()
    } else if val % 2 == 0 {
        format!("{}", val >> 1)
    } else {
        format!("Unknown value: {}", val)
    }
}

pub fn cli_mode() -> std::io::Result<()> {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    let mut reader = io::stdin().lock();
    println!("Press ^D, exit or quit to exit the REPL interative mode.");

    let mut define_env: HashMap<String, i64> = HashMap::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("\nThanks for you business with us!");
                break Ok(());
            },
            Ok(_) => {
                let input = buffer.trim();

                if input.is_empty() {
                    continue;
                }

                let command = input.to_lowercase();
                if command == "exit" || command == "quit" {
                    println!("Thanks for you business with us!");
                    break Ok(());
                }

                let sexp = match parse(&command) {
                    Ok(s) => s,
                    Err(_) => {
                        println!("Invalid: parse error");
                        continue;
                    }
                };

                let expr = match parse_repl_expr(&sexp) {
                    Ok(e) => e,
                    Err(_) => {
                        println!("Invalid: parse error");
                        continue;
                    }
                };

                let instrs = match compile_repl_to_instr(&expr, 2, &mut define_env, &mut ops) {
                    Ok(i) => i,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
                // compile_repl_to_instr in compile_jit handles JIT and printing for expressions,
                // and updates define_env for defines. It returns an empty instr list by design.
                let _ = instrs; // keep signature-consistent with minimal changes
            },
            Err(e) => {
                eprintln!("Error reading input {}", e);
                break Ok(());

            }
        }
    }
}

pub struct Repl {
    ops: dynasmrt::x64::Assembler,
    define_env: HashMap<String, i64>,
}

impl Repl {
    pub fn new() -> Self {
        Repl { ops: dynasmrt::x64::Assembler::new().unwrap(), define_env: HashMap::new() }
    }

    pub fn feed(&mut self, raw: &str) -> std::io::Result<Option<String>> {
        let input = raw.trim();

        if input.is_empty() {
            return Ok(None);
        }

        let command = input.to_lowercase();
        if command == "exit" || command == "quit" {
            return Ok(Some("Thanks for you business with us!".to_string()));
        }

        let sexp = match parse(&command) {
            Ok(s) => s,
            Err(_) => {
                return Ok(Some("Invalid: parse error".to_string()));
            }
        };

        let expr = match parse_repl_expr(&sexp) {
            Ok(e) => e,
            Err(_) => {
                return Ok(Some("Invalid: parse error".to_string()));
            }
        };

        match expr {
            ReplExpr::Define(name, inner) => {
                if self.define_env.contains_key(&name) {
                    return Ok(Some("Duplicate binding".to_string()));
                }
                let result = compile_jit_and_persist(&inner, 2, &mut self.define_env, &mut self.ops, false)?;
                self.define_env.insert(name, result as i64);
                Ok(None)
            }
            ReplExpr::Expr(inner) => {
                let result = compile_jit_and_persist(&inner, 2, &mut self.define_env, &mut self.ops, false)?;
                Ok(Some(format_viva_value(result)))
            }
        }
    }
}