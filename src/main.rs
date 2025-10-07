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
                [Sexp::Atom(S(op)), e] if op == "add1"                         => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e] if op == "sub1"                         => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e))),
                
                [Sexp::Atom(S(op)), e1, e2] if op == "+"                       => Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), e1, e2] if op == "-"                       => Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
                [Sexp::Atom(S(op)), e1, e2] if op == "*"                       => Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),

                [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                    let mut bs = Vec::new();
                    for b in bindings {
                        match b {
                            Sexp::List(vec![Sexp::Atom(S(name)), e])           => {
                                let pair = (name.clone(), parse_expr(e));
                                bs.push(pair);
                            }
                            _                                                  => panic!("Invalid")
                        }
                    }

                    Expr::Let(bs, Box::new(parse_expr(body)))
                }
                _                                                              => panic!("Invalid"),
            }
        },
        _                                                                      => panic!("Invalid"),
    }
}


// TODO ask a question about this mutable/immutable map because I honestly do not understand.
fn compile_expr(e: &Expr, si: i32, env: &mut HashMap<String, i32>) -> String {
    match e {
        Expr::Number(n)         => format!("mov rax, {}", *n),
        Expr::Id(s)             => format!("mov rax, {}", ),
        Expr::Let(v, e)         => format!("let"),
        Expr::UnOp(op, e)       => {
            let instr = compile_expr(e, si);
            match op {
                Op1::Add1       => format!("\nadd rax, 1"),
                Op1::Sub1       => format!("\nsub rax, 1"),
            }
        }
        Expr::BinOp(op, e1, e2) => {
            let e1_instr = compile_expr(e1, si);
            let e2_instr = compile_expr(e2, si + 1);
            let stack_offset = si * 8;
            match op {
                Op2::Plus       => format!("{e1_instr}\nmov [rsp - {stack_offset}], rax \n{e2_instr}\nadd rax, [rsp - {stack_offset}]"),
                Op2::Minus      => format!("{e2_instr}\nmov [rsp - {stack_offset}], rax \n{e1_instr}\nsub rax, [rsp - {stack_offset}]"),
                Op2::Times      => format!("{e1_instr}\nmov [rsp - {stack_offset}], rax \n{e2_instr}\nimul rax, [rsp - {stack_offset}]"),
            }
        }
    }
}

// fn compile_ops(e : &Expr, ops : &mut dynasmrt::x64::Assembler) {
//     match e {
//         Expr::Num(n) => { dynasm!(ops ; .arch x64 ; mov rax, *n); }
//         Expr::Add1(subexpr) => {
//             compile_ops(&subexpr, ops);
//             dynasm!(ops ; .arch x64 ; add rax, 1);
//         }
//         Expr::Sub1(subexpr) => {
//             compile_ops(&subexpr, ops);
//             dynasm!(ops ; .arch x64 ; sub rax, 1);
//         }
//         Expr::Negate(subexpr) => {
//             compile_ops(&subexpr, ops);
//             dynasm!(ops ; .arch x64; neg rax);
//         }
//     }
// }


fn main() -> std::io::Result<()> {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();
    let start = ops.offset();

    let args: Vec<String> = env::args().collect();

    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let mut env = HashMap::new();

    let expr = parse_expr(&parse(&in_contents).unwrap());
    let result = compile_expr(&expr, 2, env);
    let asm_program = format!("
section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
", result);

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    // compile_ops(&expr, &mut ops);

    // dynasm!(ops ; .arch x64 ; ret);
    // let buf = ops.finalize().unwrap();
    // let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
    // let result = jitted_fn();
    // println!("{}", result);

    Ok(())
}