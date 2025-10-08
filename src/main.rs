use std::env;
use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use sexp::*;
use sexp::Atom::*;

use dynasmrt::{dynasm, DynasmApi};

enum Op1 {
    Add1,
    Sub1
}

enum Op2 {
    Plus,
    Minus,
    Times
}

enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>)
}

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n))                                                       => Expr::Number(i32::try_from(*n).unwrap()),
        Sexp::Atom(S(s))                                                       => Expr::Id(s.clone()),
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                    let mut bs = Vec::new();
                    for b in bindings {
                        match b {
                            Sexp::List(pair) => {
                                match &pair[..] {
                                    [Sexp::Atom(S(name)), e] => {
                                        let pair = (name.clone(), parse_expr(e));
                                        bs.push(pair);
                                    }
                                    _ => panic!("Parse Error in Let Binding"),
                                }
                            }
                            _ => panic!("Parse Error in Let Binding"),
                        }
                    }
                    Expr::Let(bs, Box::new(parse_expr(body)))
                }
                [Sexp::Atom(S(op)), e] if op == "add1"                         => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e] if op == "sub1"                         => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e))),
                
                [Sexp::Atom(S(op)), e1, e2] if op == "+"                       => Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-"                       => Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*"                       => Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                _                                                              => panic!("Parse Error"),
            }
        },
        _                                                                      => panic!("Parse Error"),
    }
}

fn compile_expr(e: &Expr, si: i32, env: HashMap<String, i32>) -> std::io::Result<String> {
    match e {
        Expr::Number(n)         => Ok(format!("mov rax, {}", *n)),
        Expr::Id(s)             => {
            match env.get(s) {
                Some(offset) => Ok(format!("mov rax, [rsp - {}]", offset * 8)),
                None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
            }
        }
        Expr::Let(bs, body)     => {
            let mut result_instr = String::new();
            let mut curr_si = si;
            let mut curr_env = env.clone();

            for (v, e) in bs {
                if curr_env.contains_key(v) {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
                }
                let e_instr = compile_expr(e, si, env.clone())?;
                let store_curr_value_instr = format!("mov [rsp - {}], rax", curr_si * 8);

                curr_env.insert(v.clone(), curr_si);
                if !result_instr.is_empty() {
                    result_instr.push_str("\n");
                }
                result_instr.push_str(&e_instr);
                result_instr.push_str("\n");
                result_instr.push_str(&store_curr_value_instr);
                curr_si += 1;
            }

            if !result_instr.is_empty() {
                result_instr.push_str("\n");
            }
            let b_instr = compile_expr(body, curr_si, curr_env)?;

            result_instr.push_str(&b_instr);

            Ok(result_instr)
        },
        Expr::UnOp(op, e)       => {
            let instr = compile_expr(e, si, env.clone())?;
            match op {
                Op1::Add1       => Ok(format!("{instr}\nadd rax, 1")),
                Op1::Sub1       => Ok(format!("{instr}\nsub rax, 1")),
            }
        },
        Expr::BinOp(op, e1, e2) => {
            let e1_instr = compile_expr(e1, si, env.clone())?;
            let e2_instr = compile_expr(e2, si + 1, env.clone())?;
            let stack_offset = si * 8;
            match op {
                Op2::Plus       => Ok(format!("{e1_instr}\nmov [rsp - {stack_offset}], rax \n{e2_instr}\nadd rax, [rsp - {stack_offset}]")),
                Op2::Minus      => Ok(format!("{e2_instr}\nmov [rsp - {stack_offset}], rax \n{e1_instr}\nsub rax, [rsp - {stack_offset}]")),
                Op2::Times      => Ok(format!("{e1_instr}\nmov [rsp - {stack_offset}], rax \n{e2_instr}\nimul rax, [rsp - {stack_offset}]")),
            }
        },
    }
}

fn compile_ops(e : &Expr, ops : &mut dynasmrt::x64::Assembler, si : i32, env: HashMap<String, i32>) -> std::io::Result<()> {
    match e {
        Expr::Number(n)        => { dynasm!(ops ; .arch x64 ; mov rax, *n); Ok(()) }
        Expr::Id(s)            => {
            match env.get(s) {
                Some(offset) => { dynasm!(ops ; .arch x64 ; mov rax, [rsp - offset * 8]); Ok(())},
                None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier{}", s))),
            }
        }
        Expr::Let(bs, body)    => {
            let mut curr_si = si;
            let mut curr_env = env.clone();

            for (v, e) in bs {
                if curr_env.contains_key(v) {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
                }
                compile_ops(&e, ops, si, env.clone())?;
                let stack_offset = curr_si * 8;
                dynasm!(ops ; .arch x64 ; mov [rsp - stack_offset], rax);
                curr_env.insert(v.clone(), curr_si);
                curr_si += 1;
            }
            compile_ops(&body, ops, curr_si, curr_env.clone())?;
            Ok(())
        }
        Expr::UnOp(t, e)       => {
            match t {
                Op1::Add1      => { 
                    compile_ops(&e, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; add rax, 1); 
                    Ok(())
                }
                Op1::Sub1      => {
                    compile_ops(&e, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; sub rax, 1); 
                    Ok(())
                }
            }
        } 
        Expr::BinOp(t, e1, e2) => {
            match t {
                Op2::Plus      => {
                    let stack_offset = si * 8;
                    compile_ops(&e1, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; mov [rsp - stack_offset], rax);
                    compile_ops(&e2, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; add rax, [rsp - stack_offset]);
                    Ok(())
                }
                Op2::Minus     => {
                    let stack_offset = si * 8;
                    compile_ops(&e2, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; mov [rsp - stack_offset], rax);
                    compile_ops(&e1, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; sub rax, [rsp - stack_offset]);
                    Ok(())
                }
                Op2::Times     => {
                    let stack_offset = si * 8;
                    compile_ops(&e1, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; mov [rsp - stack_offset], rax);
                    compile_ops(&e2, ops, si, env.clone())?;
                    dynasm!(ops ; .arch x64 ; imul rax, [rsp - stack_offset]);
                    Ok(())
                }
            }
        }
    }
}

fn file_to_expr(in_name: &str) -> std::io::Result<Expr> {
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;
    Ok(parse_expr(&parse(&in_contents).unwrap()))
}

fn generate_string_mode(in_name: &str) -> std::io::Result<String> {
    let expr = file_to_expr(in_name)?;

    let env = HashMap::new();
    let result = compile_expr(&expr, 2, env.clone())?;
    Ok(format!("
section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
", result))
}

fn eval_mode(in_name: &str) -> std::io::Result<()> {
    let expr = file_to_expr(in_name)?;

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let start = ops.offset();

    let env = HashMap::new();
    let _ = compile_ops(&expr, &mut ops, 2, env.clone())?;

    dynasm!(ops ; .arch x64 ; ret);
    let buf = ops.finalize().unwrap();
    let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
    let result = jitted_fn();
    println!("{}", result);
    Ok(())
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -c <input.snek> <output.s>");
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -e <input.snek>");
        eprintln!("Usage: cargo run [CARGO_FLAGS] -- -g <input.snek> <output.s>");
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
                eprintln!("Usage: cargo run [CARGO_FLAGS] -- -g <input.snek>");
            }

            let in_name = &args[2];
            let out_name = &args[3];

            let asm_program = generate_string_mode(in_name)?;
            eval_mode(in_name)?;

            let mut out_file = File::create(out_name)?;
            out_file.write_all(asm_program.as_bytes())?;
        },
        _    => {
            eprintln!("Unknown flag: {}", flag);
            eprintln!("Allowed options: -c, -e, -d");
        }
    }
    Ok(())
}