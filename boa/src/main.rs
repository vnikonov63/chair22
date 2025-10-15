use std::env;
use std::fs::File;
use std::io::prelude::*;

mod instructions;
mod expressions;
mod parse;
mod compile;
mod modes;

use crate::modes::{generate_string_mode, eval_mode, repl_mode};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -c <input.snek> <output.s>");
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -e <input.snek>");
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -g <input.snek> <output.s>");
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -i");
    }

    let flag = &args[1];

    match flag.as_str() {
        "-c" => {
            if args.len() < 4 {
                eprintln!("Usage: cargo run [CARGO_FLAGS] -- -c <input.snek> <output.s>");
            }

            let in_name = &args[2];
            let out_name = &args[3];

            let asm_program = generate_string_mode(in_name)?;

            let mut out_file = File::create(out_name)?;
            out_file.write_all(asm_program.as_bytes())?;
        },
        "-e" => {
            if args.len() < 3 {
                eprintln!("Usage: cargo run [CARGO_FLAGS] -- -e <input.snek>");
            }

            let in_name = &args[2];

            eval_mode(in_name)?;
        },
        "-g" => {
            if args.len() < 4 {
                eprintln!("Usage: cargo run [CARGO_FLAGS] -- -g <input.snek> <output.s>");
            }

            let in_name = &args[2];
            let out_name = &args[3];

            let asm_program = generate_string_mode(in_name)?;
            eval_mode(in_name)?;

            let mut out_file = File::create(out_name)?;
            out_file.write_all(asm_program.as_bytes())?;
        },
        "-i" => {
            repl_mode()?;
        },
        _    => {
            eprintln!("Unknown flag: {}", flag);
            eprintln!("Allowed options: -c, -e, -d, -i");
        }
    }
    Ok(())
}