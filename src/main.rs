use std::env;
use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

use sexp::*;
use sexp::Atom::*;

use dynasmrt::{dynasm, DynasmApi};

#[derive(Debug, Clone)]
enum Reg {
    Rax,
}

fn reg_to_string(reg: &Reg) -> &str {
  match reg {
    Reg::Rax => "rax",
  }
}

#[derive(Debug, Clone)]
enum Instr {
    Mov(Reg, i32),         
    Add(Reg, i32),         
    Sub(Reg, i32),         
    AddRaxMemFromStack(i32),    
    SubRaxMemFromStack(i32),
    MulRaxMemFromStack(i32),
    MovToStack(Reg, i32),  
    MovFromStack(Reg, i32),
}

fn instr_to_string(instr: &Instr) -> String {
  match instr {
    Instr::Mov(reg, val) => format!("mov {}, {}", reg_to_string(reg), val),
    Instr::Add(reg, val) => format!("add {}, {}", reg_to_string(reg), val),
    Instr::Sub(reg, val) => format!("sub {}, {}", reg_to_string(reg), val),
    Instr::AddRaxMemFromStack(offset) => format!("add rax, [rsp - {}]", offset),
    Instr::SubRaxMemFromStack(offset) => format!("sub rax, [rsp - {}]", offset),
    Instr::MulRaxMemFromStack(offset) =>format!("imul rax, [rsp - {}]", offset),
    Instr::MovToStack(reg, offset) => format!("mov [rsp - {}], {}", offset, reg_to_string(reg)),
    Instr::MovFromStack(reg, offset) => format!("mov {}, [rsp - {}]", reg_to_string(reg), offset),
  }
}

fn instrs_to_string(instrs: &Vec<Instr>) -> std::io::Result<String> {
  Ok(instrs.iter()
    .map(instr_to_string)
    .collect::<Vec<String>>()
    .join("\n"))
}

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

fn parse_expr(s: &Sexp) -> std::io::Result<Expr> {
    match s {
        Sexp::Atom(I(n)) => Ok(Expr::Number(i32::try_from(*n).unwrap())),
        Sexp::Atom(S(s)) => Ok(Expr::Id(s.clone())),
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                    let mut bs = Vec::new();
                    for b in bindings {
                        match b {
                            Sexp::List(pair) => {
                                match &pair[..] {
                                    [Sexp::Atom(S(name)), e] => {
                                        let parsed = parse_expr(e)?;
                                        let pair = (name.clone(), parsed);
                                        bs.push(pair);
                                    }
                                    _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse Error in Let Binding")),
                                }
                            }
                            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse Error in Let Binding")),
                        }
                    }
                    Ok(Expr::Let(bs, Box::new(parse_expr(body)?)))
                }
                [Sexp::Atom(S(op)), e] if op == "add1" => Ok(Expr::UnOp(Op1::Add1, Box::new(parse_expr(e)?))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => Ok(Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e)?))),

                [Sexp::Atom(S(op)), e1, e2] if op == "+" => Ok(Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-" => Ok(Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*" => Ok(Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)?), Box::new(parse_expr(e2)?))),
                _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse Error")),
            }
        },
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse Error")),
    }
}

// TODO: Ask what is happening here in case in the future I decide to add more registers, dynsasm
// does not just take the strings, and we might need to use something different, right?
fn instr_to_dynasm(ops : &mut dynasmrt::x64::Assembler, instrs: &Vec<Instr>) -> std::io::Result<()> {
    for instr in instrs {
        match instr {
            Instr::Mov(Reg::Rax, val) => { dynasm!(ops ; .arch x64 ; mov rax, *val); }
            Instr::Add(Reg::Rax, val) => { dynasm!(ops ; .arch x64 ; add rax, *val); }
            Instr::Sub(Reg::Rax, val) => { dynasm!(ops ; .arch x64 ; sub rax, *val); }
            Instr::AddRaxMemFromStack(offset) => { dynasm!(ops ; .arch x64 ; add rax, [rsp - *offset]); }
            Instr::SubRaxMemFromStack(offset) => { dynasm!(ops ; .arch x64 ; sub rax, [rsp - *offset]); }
            Instr::MulRaxMemFromStack(offset) => { dynasm!(ops ; .arch x64 ; imul rax, [rsp - *offset]); }
            Instr::MovToStack(Reg::Rax, offset) => { dynasm!(ops ; .arch x64 ; mov [rsp - *offset], rax); }
            Instr::MovFromStack(Reg::Rax, offset) => { dynasm!(ops ; .arch x64 ; mov rax, [rsp - *offset]); }
        }
    }
    Ok(())
}

fn compile_to_instr(e: &Expr, si: i32, env: HashMap<String, i32>) -> std::io::Result<Vec<Instr>> {
    match e {
        Expr::Number(n) => Ok(vec![Instr::Mov(Reg::Rax, *n)]),
        Expr::Id(s) => {
            match env.get(s) {
                Some(offset) => Ok(vec![Instr::MovFromStack(Reg::Rax, offset * 8)]),
                None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
            }
        },
        Expr::Let(bs, body) => {
            let mut result_instr : Vec<Instr> = Vec::new();
            let mut curr_si = si;
            let mut curr_env = env.clone();

            // JUST LIKE IN BFS WE ARE HERE ON THE SAME LEVEL SO WE CAN CHECK UNIQUENESS AT IT WITH A HASH_MAP
            let mut level = HashSet::new();
            for (v, e) in bs {
                if level.contains(v) {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
                }
                let e_instr = compile_to_instr(e, si, curr_env.clone())?;
                result_instr.extend(e_instr);
                result_instr.push(Instr::MovToStack(Reg::Rax, curr_si * 8));

                level.insert(v.clone());
                curr_env.insert(v.clone(), curr_si);
                curr_si += 1;
            }

            let b_instr = compile_to_instr(body, curr_si, curr_env)?;
            result_instr.extend(b_instr);

            Ok(result_instr)
        },
        Expr::UnOp(op, e) => {
            let mut instr = compile_to_instr(e, si, env.clone())?;
            match op {
                Op1::Add1 => instr.push(Instr::Add(Reg::Rax, 1)),
                Op1::Sub1 => instr.push(Instr::Sub(Reg::Rax, 1)),
            }
            Ok(instr)
        },
        Expr::BinOp(op, e1, e2) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let stack_offset = si * 8;
            let e1_instr = compile_to_instr(e1, si, env.clone())?;
            let e2_instr = compile_to_instr(e2, si + 1, env.clone())?;

            match op {
                Op2::Plus => {
                    result_instr.extend(e1_instr);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(e2_instr);
                    result_instr.push(Instr::AddRaxMemFromStack(stack_offset));
                }
                Op2::Minus => {
                    result_instr.extend(e2_instr);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(e1_instr);
                    result_instr.push(Instr::SubRaxMemFromStack(stack_offset));
                }
                Op2::Times => {
                    result_instr.extend(e1_instr);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(e2_instr);
                    result_instr.push(Instr::MulRaxMemFromStack(stack_offset));
                }
            }
            Ok(result_instr)
        },
    }
}

fn file_to_expr(in_name: &str) -> std::io::Result<Expr> {
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;
    // `parse` returns a Result<Sexp, _> from the `sexp` crate; propagate any errors
    let sexp = parse(&in_contents).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Sexp parse error: {}", e)))?;
    parse_expr(&sexp)
}

fn generate_string_mode(in_name: &str) -> std::io::Result<String> {
    let expr = file_to_expr(in_name)?;

    let env = HashMap::new();
    let instrs = compile_to_instr(&expr, 2, env.clone())?;
    let result = instrs_to_string(&instrs)?;

    Ok(format!("\nsection .text\nglobal our_code_starts_here\nour_code_starts_here:\n  {}\n  ret\n", result))
}

fn eval_mode(in_name: &str) -> std::io::Result<()> {
    let expr = file_to_expr(in_name)?;

    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let start = ops.offset();

    let env = HashMap::new();
    let instrs = compile_to_instr(&expr, 2, env.clone())?;
    instr_to_dynasm(&mut ops, &instrs)?;
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