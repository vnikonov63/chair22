use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::mem;

use sexp::*;

use dynasmrt::{dynasm, DynasmApi};

use crate::expressions::{Expr};
use crate::parse::{parse_expr, parse_repl_expr};
use crate::instructions::{instrs_to_string, instr_to_dynasm};
use crate::compile::{compile_to_instr, compile_repl_to_instr};

pub fn file_to_expr(in_name: &str) -> std::io::Result<Expr> {
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;
    let sexp = parse(&in_contents).map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "Invalid: parse error".to_string()))?;
    parse_expr(&sexp)
}

pub fn generate_string_mode(in_name: &str) -> std::io::Result<String> {
    let expr = file_to_expr(in_name)?;

    let env = HashMap::new();
    let empty_define_env = HashMap::new();
    let instrs = compile_to_instr(&expr, 2, env.clone(), &empty_define_env)?;
    let result = instrs_to_string(&instrs)?;

    Ok(format!("\nsection .text\nglobal our_code_starts_here\nour_code_starts_here:\n  {}\n  ret\n", result))
}

pub fn eval_mode(in_name: &str) -> std::io::Result<()> {
    let expr = file_to_expr(in_name)?;

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let start = ops.offset();

    let env = HashMap::new();
    let empty_define_env = HashMap::new();
    let instrs = compile_to_instr(&expr, 2, env.clone(), &empty_define_env)?;
    instr_to_dynasm(&mut ops, &instrs)?;
    dynasm!(ops ; .arch x64 ; ret);
    
    let buf = ops.finalize().unwrap();
    let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
    let result = jitted_fn();
    println!("{}", result);
    Ok(())
}

pub fn repl_mode() -> std::io::Result<()> {
    // We can reuse this ops later, so we do not compile it every time as new
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    let mut reader = io::stdin().lock();
    println!("Press ^D, exit or quit to exit the REPL interative mode.");

    // We are going to use this in mutiple loop iterations, so it shouldn't 
    // be destructed
    let mut define_env = HashMap::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // The Ctr-D (^D) wouldn;t create a new line, because the escape character is not 
                // printed, so it would appear as if my farewell message is with the > delimeter
                // without the newline character at the begining of the message below.
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

                let start = ops.offset();
                let instrs = match compile_repl_to_instr(&expr, 2, &mut define_env, &mut ops) {
                    Ok(i) => i,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

                // We do not want to print for define
                if instrs.is_empty() {
                    continue;
                }

                instr_to_dynasm(&mut ops, &instrs)?;
                dynasm!(ops ; .arch x64 ; ret);

                ops.commit().unwrap();
                let reader = ops.reader();
                let buf = reader.lock();
                let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
                let result = jitted_fn();
                
                println!("{}", result);
            },
            Err(e) => {
                eprintln!("Error reading input {}", e);
                break Ok(());

            }
        }
    }
}