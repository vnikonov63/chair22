use std::collections::{HashMap, HashSet};
use std::io;
use std::io::prelude::*;

use sexp::*;
// no explicit dynasm usage here; compilation happens in helpers

use crate::parse::parse_repl_expr;
use crate::compile_repl::{compile_repl_and_persist, compile_repl_to_instr};
use crate::expressions::ReplExpr;

pub fn cli_mode() -> std::io::Result<()> {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let mut labels: HashMap<String, dynasmrt::DynamicLabel> = HashMap::new();
    let mut define_env: HashMap<String, i64> = HashMap::new();
    let mut func_names: HashSet<String> = HashSet::new();

    let mut reader = io::stdin().lock();
    println!("Press ^D, exit or quit to exit the REPL interative mode.");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("\nThanks for you business with us!");
                break Ok(());
            }
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

                let expr = match parse_repl_expr(&sexp, &func_names) {
                    Ok(e) => e,
                    Err(_) => {
                        println!("Invalid: parse error");
                        continue;
                    }
                };

                match &expr {
                    ReplExpr::Fun(name, _, _) => {
                        // Register the function name first to prevent future duplicates
                        func_names.insert(name.clone());
                        if let Err(err) = compile_repl_to_instr(&expr, 2, &mut define_env, &mut ops, &mut labels) {
                            println!("{}", err);
                        }
                    }
                    ReplExpr::Define(name, inner) => {
                        if define_env.contains_key(name.as_str()) {
                            println!("Duplicate binding");
                            continue;
                        }
                        match compile_repl_and_persist(inner.as_ref(), 2, &mut define_env, &mut ops, &mut labels, false) {
                            Ok(result) => {
                                define_env.insert(name.clone(), result as i64);
                            }
                            Err(err) => println!("{}", err),
                        }
                    }
                    ReplExpr::Expr(inner) => {
                        // Print the result using runtime printer
                        match compile_repl_and_persist(inner.as_ref(), 2, &mut define_env, &mut ops, &mut labels, true) {
                            Ok(_) => {}
                            Err(err) => println!("{}", err),
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input {}", e);
                break Ok(());
            }
        }
    }
}

fn format_viva_value(result: i64) -> String {
    if result == 3 {
        "true".to_string()
    } else if result == 1 {
        "false".to_string()
    } else {
        format!("{}", result >> 1)
    }
}

pub struct Repl {
    ops: dynasmrt::x64::Assembler,
    labels: HashMap<String, dynasmrt::DynamicLabel>,
    define_env: HashMap<String, i64>,
    func_names: HashSet<String>,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            ops: dynasmrt::x64::Assembler::new().unwrap(),
            labels: HashMap::new(),
            define_env: HashMap::new(),
            func_names: HashSet::new(),
        }
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

        let expr = match parse_repl_expr(&sexp, &self.func_names) {
            Ok(e) => e,
            Err(_) => {
                return Ok(Some("Invalid: parse error".to_string()));
            }
        };

        match &expr {
            ReplExpr::Fun(name, _, _) => {
                self.func_names.insert(name.clone());
                if let Err(err) = compile_repl_to_instr(&expr, 2, &mut self.define_env, &mut self.ops, &mut self.labels) {
                    return Ok(Some(format!("{}", err)));
                }
                Ok(None)
            }
            ReplExpr::Define(name, inner) => {
                if self.define_env.contains_key(name.as_str()) {
                    return Ok(Some("Duplicate binding".to_string()));
                }
                let result = compile_repl_and_persist(inner.as_ref(), 2, &mut self.define_env, &mut self.ops, &mut self.labels, false)?;
                self.define_env.insert(name.clone(), result as i64);
                Ok(None)
            }
            ReplExpr::Expr(inner) => {
                let result = compile_repl_and_persist(inner.as_ref(), 2, &mut self.define_env, &mut self.ops, &mut self.labels, false)?;
                Ok(Some(format_viva_value(result)))
            }
        }
    }
}