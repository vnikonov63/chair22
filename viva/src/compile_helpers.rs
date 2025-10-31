use std::collections::{HashMap, HashSet};

use crate::instructions::{Instr, Reg};
use crate::counter::next_id;
use crate::expressions::{Expr};

#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

pub enum TypeOp {
    Bool,
    Num
}


pub fn gen_compare(e1_instr: Vec<Instr>, e2_instr: Vec<Instr>, stack_offset: i32, op: CmpOp) -> Vec<Instr> {
    let mut result: Vec<Instr> = Vec::new();

    result.extend(e2_instr);
    result.push(Instr::MovToStack(Reg::Rax, stack_offset + 8));

    result.extend(e1_instr);

    match op {
        CmpOp::Equal => {
            result.extend(equal_type_handler(stack_offset + 8));
        }
        _ => {
            result.extend(at_least_one_bool_handler(stack_offset + 8));
        }
    }

    result.push(Instr::CompareWithMemory(Reg::Rax, stack_offset + 8));
    result.push(Instr::Mov(Reg::Rax, 1));
    result.push(Instr::Mov(Reg::R10, 3));

    match op {
        CmpOp::Equal => result.push(Instr::Cmove(Reg::Rax, Reg::R10)),
        CmpOp::Greater => result.push(Instr::Cmovg(Reg::Rax, Reg::R10)),
        CmpOp::GreaterEqual => result.push(Instr::Cmovge(Reg::Rax, Reg::R10)),
        CmpOp::Less => result.push(Instr::Cmovl(Reg::Rax, Reg::R10)),
        CmpOp::LessEqual => result.push(Instr::Cmovle(Reg::Rax, Reg::R10)),
    };

    result
}

pub fn gen_istype(op: TypeOp) -> Vec<Instr> {
    let mut result: Vec<Instr> = Vec::new();

    result.push(Instr::Test(Reg::Rax, 1));

    result.push(Instr::Mov(Reg::Rax, 1));
    result.push(Instr::Mov(Reg::R10, 3));

    match op {
        TypeOp::Bool => result.push(Instr::Cmovne(Reg::Rax, Reg::R10)), 
        TypeOp::Num => result.push(Instr::Cmove(Reg::Rax, Reg::R10)),
    }

    result
}

pub fn at_least_one_bool_handler(offset: i32) -> Vec<Instr> {
    let mut result: Vec<Instr> = Vec::new();

    let id = next_id();
    let ok_label = format!("bool_ok{}", id);

    result.push(Instr::MovFromReg(Reg::R11, Reg::Rax));
    result.push(Instr::MovFromStack(Reg::R8, offset));
    result.push(Instr::Or(Reg::Rax, Reg::R8));
    result.push(Instr::Test(Reg::Rax, 1));
    result.push(Instr::Je(ok_label.clone()));
    result.push(Instr::CallRustError(2));
    result.push(Instr::Label(ok_label));
    result.push(Instr::MovFromReg(Reg::Rax, Reg::R11));

    result
}

pub fn equal_type_handler(offset: i32) -> Vec<Instr> {
    let mut result: Vec<Instr> = Vec::new();

    let id = next_id();
    let ok_label = format!("equal_ok{}", id);

    result.push(Instr::MovFromReg(Reg::R11, Reg::Rax));
    result.push(Instr::MovFromStack(Reg::R8, offset));
    result.push(Instr::Xor(Reg::Rax, Reg::R8));
    result.push(Instr::Test(Reg::Rax, 1));
    result.push(Instr::Je(ok_label.clone()));
    result.push(Instr::CallRustError(2));
    result.push(Instr::Label(ok_label));
    result.push(Instr::MovFromReg(Reg::Rax, Reg::R11));

    result
}

pub fn unary_not_bool_handler() -> Vec<Instr> {
    let mut result: Vec<Instr> = Vec::new();
    let id = next_id();
    let ok_label = format!("unop_ok{}", id);

    result.push(Instr::Test(Reg::Rax, 1));
    result.push(Instr::Je(ok_label.clone()));
    result.push(Instr::CallRustError(2));
    result.push(Instr::Label(ok_label.clone()));

    result
}

pub fn overflow_handler() -> Vec<Instr> {
    let mut result: Vec<Instr> = Vec::new();
    let id = next_id();
    let no_overflow_label = format!("no_overflow{}", id);

    result.push(Instr::Jno(no_overflow_label.to_string()));
    result.push(Instr::CallRustError(1));
    result.push(Instr::Label(no_overflow_label.clone()));

    result
}

fn recursively_collet_set_identifiers(curre: &Expr, result: &mut HashSet<String>) {
    match curre {
        Expr::Number(_) => {},
        Expr::Boolean(_) => {},
        Expr::Id(_) => {},
        Expr::UnOp(_, e) => {
            recursively_collet_set_identifiers(e, result);
        },
        Expr::Loop(e) => {
            recursively_collet_set_identifiers(e, result);
        },
        Expr::Break(e) => {
            recursively_collet_set_identifiers(e, result);
        },
        Expr::BinOp(_, e1, e2) => {
            recursively_collet_set_identifiers(e1, result);
            recursively_collet_set_identifiers(e2, result);
        },
        Expr::If(e1, e2, e3) => {
            recursively_collet_set_identifiers(e1, result);
            recursively_collet_set_identifiers(e2, result);
            recursively_collet_set_identifiers(e3, result);
        },
        Expr::Let(bs, body) => {
            for (_, b) in bs {
                recursively_collet_set_identifiers(b, result);
            }
            recursively_collet_set_identifiers(body, result);
        },
        Expr::Block(bs) => {
            for expr in bs {
                recursively_collet_set_identifiers(expr, result);
            }
        },
        Expr::Set(name, e) => {
            result.insert(name.clone());
            recursively_collet_set_identifiers(e, result);
        }
    }
}

pub fn allocate_define_ptrs_for_set_targets(
    expr: &Expr,
    define_env: &HashMap<String, i64>,
) -> HashMap<String, i64> {
    let mut targets = HashSet::new();
    recursively_collet_set_identifiers(expr, &mut targets);

    let mut result: HashMap<String, i64> = HashMap::new();
    for name in targets {
        match define_env.get(&name) {
            Some(&val) => {
                let boxed = Box::new(val);
                let ptr = Box::into_raw(boxed) as i64;
                result.insert(name, ptr);
            }
            None => {}
        }
    }
    result
}