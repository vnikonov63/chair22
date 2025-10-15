use std::env;
use std::fs::File;
use std::io::prelude::*;

use boa::{generate_string_mode, eval_mode, repl_mode};

fn usage() {
    eprintln!("Usage: cargo run -p cli -- -c <input.snek> <output.s>");
    eprintln!("       cargo run -p cli -- -e <input.snek>");
    eprintln!("       cargo run -p cli -- -g <input.snek> <output.s>");
    eprintln!("       cargo run -p cli -- -i");
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "-c" => {
            if args.len() < 4 { usage(); std::process::exit(1); }
            let in_name = &args[2];
            let out_name = &args[3];
            let asm_program = generate_string_mode(in_name)?;
            File::create(out_name)?.write_all(asm_program.as_bytes())?;
        }
        "-e" => {
            if args.len() < 3 { usage(); std::process::exit(1); }
            let in_name = &args[2];
            eval_mode(in_name)?;
        }
        "-g" => {
            if args.len() < 4 { usage(); std::process::exit(1); }
            let in_name = &args[2];
            let out_name = &args[3];
            let asm_program = generate_string_mode(in_name)?;
            eval_mode(in_name)?;
            File::create(out_name)?.write_all(asm_program.as_bytes())?;
        }
        "-i" => {
            repl_mode()?;
        }
        _ => {
            eprintln!("Unknown flag: {}", args[1]);
            usage();
            std::process::exit(1);
        }
    }
    Ok(())
}