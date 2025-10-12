use std::env;
use std::mem;
use std::fs::File;
// TODO: ask whether I should import the prelude thingy here, if I already imported the io.
use std::io;
use std::io::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::result;

use sexp::*;
use sexp::Atom::*;

use dynasmrt::{dynasm, DynasmApi};

/* ----------------------------------- */
/* Instructions Logic */

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
/* ----------------------------------- */

/* ----------------------------------- */
/* Expressions Logic */
#[derive(Debug)]
enum Op1 {
    Add1,
    Sub1
}

#[derive(Debug)]
enum Op2 {
    Plus,
    Minus,
    Times
}

#[derive(Debug)]
enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
enum ReplExpr {
    Define(String, Box<Expr>),
    Expr(Box<Expr>),
}
/* ----------------------------------- */

/* ----------------------------------- */
/* PARSING LOGIC */
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

fn parse_repl_expr(s: &Sexp) -> std::io::Result<ReplExpr> {
    match s {
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), Sexp::Atom(S(v)), e] if op == "define" => Ok(ReplExpr::Define(v.clone(), Box::new(parse_expr(e)?))),

                _ => Ok(ReplExpr::Expr(Box::new(parse_expr(s)?))),
            }
        }
        _ => Ok(ReplExpr::Expr(Box::new(parse_expr(s)?))),
    }
}
/* ----------------------------------- */

/* ----------------------------------- */
/* Compiling Logic */
fn compile_to_instr(e: &Expr, si: i32, env: HashMap<String, i32>) -> std::io::Result<Vec<Instr>> {
    match e {
        Expr::Number(n) => Ok(vec![Instr::Mov(Reg::Rax, *n)]),
        Expr::Id(s) => {
            match env.get(s) {
                // the multiplication coerces/dereferences the &i32 here, but in the case
                // of compile_reple_to_instr I need to dereference it!
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
        }
    }
}

// I understand this is not really the compile thing, but in my head this is on the same level as compile_to_instr
fn compile_repl_to_instr(
    e: &ReplExpr, si: i32, 
    env: HashMap<String, i32>, 
    define_env: &mut HashMap<String, i32>, 
    ops: &mut dynasmrt::x64::Assembler
) -> std::io::Result<Vec<Instr>> {
    match e {
        ReplExpr::Define(v, e) => {
            let e_instr = compile_to_instr(e, si, env)?;

            /* the running logic */
            let start = ops.offset();
            instr_to_dynasm(ops, &e_instr)?;
            dynasm!(ops ; .arch x64 ; ret);
            ops.commit().unwrap();
            let reader = ops.reader();
            let buf = reader.lock();
            let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
            let result = jitted_fn();
            define_env.insert(v.clone(), result as i32);

            Ok(vec![])
        }
        ReplExpr::Expr(e) => {
            // e initially here is &Box<Expr>
            // *e dereferences to Box<Expr>
            // **e dereferences to Expr
            // &**e makes it &Expr, so we can use all of the previous non repl stuff 
            match &**e {
                Expr::Id(s) => {
                    // we can only acess this on the very very top level
                    // as this is the only time we are calling for the compile_repl... thingy
                    // so we automatically check two boxes
                    // 1. define can only be on the uppermost level
                    // 2. we can overshadow the variables "defined" within the let statements, 
                    // as it is the compile_to_instr business now BINGO.
                    match define_env.get(s) {
                        Some(val) => Ok(vec![Instr::Mov(Reg::Rax, *val)]),
                        None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
                    }
                }
                _ => compile_to_instr(e, si, env.clone()),
            }
        }
    }
}
/* ----------------------------------- */

/* ----------------------------------- */
/* Different Modes Logic */
fn file_to_expr(in_name: &str) -> std::io::Result<Expr> {
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;
    // TODO: Ask a question about this bit. It was generated by copilot for error propagation or smth
    // in class wed we did a similar thing, and I did not catch it. What is the meaning of this "thunk" it
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

fn repl_mode() -> std::io::Result<()> {
    // We can reuse this ops later, so we do not compile it every time as new
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let start = ops.offset();

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

                let sexp = parse(&command).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Sexp parse error: {}", e)))?;
                let expr = parse_repl_expr(&sexp)?;

                let env = HashMap::new();
                let instrs = compile_repl_to_instr(&expr, 2, env.clone(), &mut define_env, &mut ops)?;

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