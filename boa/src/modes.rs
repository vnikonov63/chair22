use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::mem;

use sexp::*;

use dynasmrt::{dynasm, DynasmApi};

use crate::parse::{parse_repl_expr};
use crate::instructions::{instr_to_dynasm};
use crate::compile::{compile_repl_to_instr};

pub fn repl_mode() -> std::io::Result<()> {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    let mut reader = io::stdin().lock();
    println!("Press ^D, exit or quit to exit the REPL interative mode.");

    let mut define_env = HashMap::new();

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

                let start = ops.offset();
                let instrs = match compile_repl_to_instr(&expr, 2, &mut define_env, &mut ops) {
                    Ok(i) => i,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };

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