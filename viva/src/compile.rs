use std::collections::HashMap;
use std::collections::HashSet;

use crate::compile_helpers::{
    overflow_handler,
    at_least_one_bool_handler,
    unary_not_bool_handler,
    gen_compare,
    gen_istype,
    CmpOp,
    TypeOp,
};
use crate::expressions::{Expr, Op1, Op2};
use crate::instructions::{Reg, Instr};
use crate::counter::{next_id};


pub fn compile_to_instr(
    e: &Expr, si: i32,
    env: HashMap<String, i32>,
    define_env: &HashMap<String, i64>,
    define_ptrs: &HashMap<String, i64>,
    curr_break: u64,
) -> std::io::Result<Vec<Instr>> {
    match e {
        Expr::Number(n) => Ok(vec![Instr::Mov(Reg::Rax, *n << 1)]),
        Expr::Boolean(b) => {
            match b {
                true => Ok(vec![Instr::Mov(Reg::Rax, 3)]),
                false => Ok(vec![Instr::Mov(Reg::Rax, 1)]),
            }
        }
        Expr::Id(s) => {
            if s == "input" {
                return Ok(vec![Instr::MovFromReg(Reg::Rax, Reg::Rdi)]);
            }
            match env.get(s) {
                Some(offset) => Ok(vec![Instr::MovFromStack(Reg::Rax, offset * 8)]),
                None => match define_ptrs.get(s) {
                    Some(ptr) => Ok(vec![
                        Instr::Mov(Reg::Rax, *ptr),
                        Instr::MovRaxFromRaxPtr,
                    ]),
                    None => match define_env.get(s) {
                        Some(value) => Ok(vec![Instr::Mov(Reg::Rax, *value)]),
                        None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
                    }
                }
            }
        },
        Expr::Let(bs, body) => {
            let mut result_instr : Vec<Instr> = Vec::new();
            let mut curr_si = si;
            let mut curr_env = env.clone();

            let mut level = HashSet::new();
            for (v, e) in bs {
                if level.contains(v) {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
                }
                let e_instr = compile_to_instr(e, curr_si, curr_env.clone(), define_env, define_ptrs, curr_break)?;
                result_instr.extend(e_instr);
                result_instr.push(Instr::MovToStack(Reg::Rax, curr_si * 8));

                level.insert(v.clone());
                curr_env.insert(v.clone(), curr_si);
                curr_si += 1;
            }

            let b_instr = compile_to_instr(body, curr_si, curr_env, define_env, define_ptrs, curr_break)?;
            result_instr.extend(b_instr);

            Ok(result_instr)
        },
        Expr::UnOp(op, e) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let e_instr = compile_to_instr(e, si, env.clone(), define_env, define_ptrs, curr_break)?;
            result_instr.extend(e_instr);

            match op {
                Op1::Add1 => {
                    result_instr.extend(unary_not_bool_handler());

                    result_instr.push(Instr::Add(Reg::Rax, 2));

                    result_instr.extend(overflow_handler());
                },
                Op1::Sub1 => { 
                    result_instr.extend(unary_not_bool_handler());

                    result_instr.push(Instr::Sub(Reg::Rax, 2));

                    result_instr.extend(overflow_handler());
                },
                Op1::IsNum => {
                    result_instr.extend(gen_istype(TypeOp::Num));
                },
                Op1::IsBool => {
                    result_instr.extend(gen_istype(TypeOp::Bool));
                }
            }

            Ok(result_instr)
        },
        Expr::BinOp(op, e1, e2) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let stack_offset = si * 8;
            let e1_instr = compile_to_instr(e1, si, env.clone(), define_env, define_ptrs, curr_break)?;
            let e2_instr = compile_to_instr(e2, si + 1, env.clone(), define_env, define_ptrs, curr_break)?;

            match op {
                Op2::Plus => {
                    result_instr.extend(e1_instr);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(e2_instr);

                    result_instr.extend(at_least_one_bool_handler(stack_offset));

                    result_instr.push(Instr::AddRaxMemFromStack(stack_offset));
                    result_instr.extend(overflow_handler());
                }
                Op2::Minus => {
                    result_instr.extend(compile_to_instr(e2, si, env.clone(), define_env, define_ptrs, curr_break)?);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(compile_to_instr(e1, si + 1, env.clone(), define_env, define_ptrs, curr_break)?);

                    result_instr.extend(at_least_one_bool_handler(stack_offset));

                    result_instr.push(Instr::SubRaxMemFromStack(stack_offset));

                    result_instr.extend(overflow_handler());
                }
                Op2::Times => {
                    result_instr.extend(e1_instr);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(e2_instr);
                    result_instr.extend(at_least_one_bool_handler(stack_offset));

                    result_instr.push(Instr::MulRaxMemFromStack(stack_offset));

                    result_instr.extend(overflow_handler());
                    result_instr.push(Instr::ShiftArithmeticRight(Reg::Rax, 1));

                },
                Op2::Equal => {
                    result_instr.extend(gen_compare(e1_instr, e2_instr, stack_offset, CmpOp::Equal));
                },
                Op2::Greater => {
                    result_instr.extend(gen_compare(e1_instr, e2_instr, stack_offset, CmpOp::Greater));
                },
                Op2::GreaterEqual => {
                    result_instr.extend(gen_compare(e1_instr, e2_instr, stack_offset, CmpOp::GreaterEqual));
                },
                Op2::Less => {
                    result_instr.extend(gen_compare(e1_instr, e2_instr, stack_offset, CmpOp::Less));
                },
                Op2::LessEqual => {
                    result_instr.extend(gen_compare(e1_instr, e2_instr, stack_offset, CmpOp::LessEqual));
                }
            }
            Ok(result_instr)
        },
        Expr::If(cond, ifbr, elbr) => {
            let mut result_instr : Vec<Instr> = Vec::new();

            let id = next_id();
            let else_label = format!("else_branch{}", id);
            let end_label = format!("if_statement_end{}", id);

            let cond_instr = compile_to_instr(cond, si, env.clone(), define_env, define_ptrs, curr_break)?;
            let ifbr_instr = compile_to_instr(ifbr, si, env.clone(), define_env, define_ptrs, curr_break)?;
            let elsebr_instr = compile_to_instr(elbr, si + 1, env.clone(), define_env, define_ptrs, curr_break)?;

            result_instr.extend(cond_instr);
            result_instr.push(Instr::Compare(Reg::Rax));
            result_instr.push(Instr::Jne(else_label.to_string()));
            result_instr.extend(ifbr_instr);
            result_instr.push(Instr::Jmp(end_label.to_string()));
            result_instr.push(Instr::Label(else_label.to_string()));
            result_instr.extend(elsebr_instr);
            result_instr.push(Instr::Label(end_label.to_string()));

            Ok(result_instr)
        },
        Expr::Loop(e) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let id = next_id();
            let start_label = format!("loop_start{}", id);
            let end_label = format!("loop_end{}", id);

            let inner_instr = compile_to_instr(e, si, env, define_env, define_ptrs, id)?;

            result_instr.push(Instr::Label(start_label.to_string()));
            result_instr.extend(inner_instr);
            result_instr.push(Instr::Jmp(start_label.to_string()));
            result_instr.push(Instr::Label(end_label.to_string()));

            Ok(result_instr)
        },
        Expr::Break(e) => {
            if curr_break == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "break outside of a loop"));
            }
            let mut result_instr: Vec<Instr> = Vec::new();

            let label = format!("loop_end{}", curr_break);

            let inner_instr = compile_to_instr(e, si, env.clone(), define_env, define_ptrs, curr_break)?;

            result_instr.extend(inner_instr);
            result_instr.push(Instr::Jmp(label.to_string()));

            Ok(result_instr)
        }, 

        Expr::Set(s, e) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let inner_instr = compile_to_instr(e, si, env.clone(), define_env, define_ptrs, curr_break)?;

            result_instr.extend(inner_instr);

            match env.get(s) {
                Some(offset) => {
                    result_instr.push(Instr::MovToStack(Reg::Rax, offset * 8));
                    Ok(result_instr)
                }
                None => match define_ptrs.get(s) {
                    Some(ptr) => {
                        result_instr.push(Instr::Mov(Reg::Rdx, *ptr));
                        result_instr.push(Instr::MovToPtrFromReg(Reg::Rdx, Reg::Rax));
                        Ok(result_instr)
                    }
                    None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
                }
            }
        },

        Expr::Block(bs) => {
            let mut result_instr: Vec<Instr> = Vec::new();
            for expr in bs {
                let expr_instr = compile_to_instr(expr, si, env.clone(), define_env, define_ptrs, curr_break)?;
                result_instr.extend(expr_instr);
            }

            Ok(result_instr)
        }
    }
}