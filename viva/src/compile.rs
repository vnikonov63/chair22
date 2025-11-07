use std::collections::HashMap;
use std::collections::HashSet;

use crate::compile_helpers::{
    overflow_handler,
    at_least_one_bool_handler,
    unary_not_bool_handler,
    gen_compare,
    gen_istype,
    recursively_collet_depth,
    CmpOp,
    TypeOp,
};
use crate::expressions::{Expr, Op1, Op2, Defenition};
use crate::instructions::{Reg, Instr};
use crate::counter::{next_id};
use crate::context::{Context};

pub fn compile_expr_to_instr(e: &Expr, ctx: &mut Context<'_>) -> std::io::Result<Vec<Instr>> {
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
            match ctx.env.get(s) {
                Some(offset) => Ok(vec![Instr::MovFromStack(Reg::Rax, offset * 8)]),
                None => match ctx.define_ptrs.get(s) {
                    Some(ptr) => Ok(vec![
                        Instr::Mov(Reg::Rax, *ptr),
                        Instr::MovRaxFromRaxPtr,
                    ]),
                    None => match ctx.define_env.get(s) {
                        Some(value) => Ok(vec![Instr::Mov(Reg::Rax, *value)]),
                        None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
                    }
                }
            }
        },
        Expr::Let(bindings, body) => {
            let mut result_instr : Vec<Instr> = Vec::new();
            let mut curr_si = ctx.si;
            let mut curr_env = ctx.env.clone();

            let mut level = HashSet::new();
            for (v, e) in bindings {
                if level.contains(v) {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
                }
                let e_ctx =  &mut Context { si: curr_si, env: curr_env.clone(), ..*ctx };
                let e_instr = compile_expr_to_instr(e, e_ctx)?;
                result_instr.extend(e_instr);
                result_instr.push(Instr::MovToStack(Reg::Rax, curr_si * 8));

                level.insert(v.clone());
                curr_env.insert(v.clone(), curr_si);
                curr_si += 1;
            }

            let body_ctx = &mut Context { si: curr_si, env: curr_env.clone(), ..*ctx };
            let body_instr = compile_expr_to_instr(body, body_ctx)?;
            result_instr.extend(body_instr);

            Ok(result_instr)
        },
        Expr::UnOp(op, e) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let e_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
            let e_instr = compile_expr_to_instr(e, e_ctx)?;
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
                },
                Op1::Print => {
                    result_instr.push(Instr::MovFromReg(Reg::Rbx, Reg::Rax));
                    result_instr.push(Instr::MovFromReg(Reg::R12, Reg::Rdi));
                    result_instr.push(Instr::CallRustPrint(Reg::Rax));
                    result_instr.push(Instr::MovFromReg(Reg::Rdi, Reg::R12));
                    result_instr.push(Instr::MovFromReg(Reg::Rax, Reg::Rbx));
                }
            }

            Ok(result_instr)
        },
        Expr::BinOp(op, e1, e2) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let stack_offset = ctx.si * 8;
            let e1_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
            let e1_instr = compile_expr_to_instr(e1, e1_ctx)?;
            let e2_ctx = &mut Context { si: ctx.si + 1, env: ctx.env.clone(), ..*ctx };
            let e2_instr = compile_expr_to_instr(e2, e2_ctx)?;

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
                    result_instr.extend(compile_expr_to_instr(e2, &mut Context { env: ctx.env.clone(), ..*ctx })?);
                    result_instr.push(Instr::MovToStack(Reg::Rax, stack_offset));
                    result_instr.extend(compile_expr_to_instr(e1, &mut Context { si: ctx.si + 1, env: ctx.env.clone(), ..*ctx })?);

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

            let cond_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
            let cond_instr = compile_expr_to_instr(cond, cond_ctx)?;
            let ifbr_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
            let ifbr_instr = compile_expr_to_instr(ifbr, ifbr_ctx)?;
            let elsebr_ctx = &mut Context { si: ctx.si + 1, env: ctx.env.clone(), ..*ctx };
            let elsebr_instr = compile_expr_to_instr(elbr, elsebr_ctx)?;

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

            let inner_ctx = &mut Context { curr_break: id, env: ctx.env.clone(), ..*ctx };
            let inner_instr = compile_expr_to_instr(e, inner_ctx)?;

            result_instr.push(Instr::Label(start_label.to_string()));
            result_instr.extend(inner_instr);
            result_instr.push(Instr::Jmp(start_label.to_string()));
            result_instr.push(Instr::Label(end_label.to_string()));

            Ok(result_instr)
        },
        Expr::Break(e) => {
            if ctx.curr_break == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "break outside of a loop"));
            }
            let mut result_instr: Vec<Instr> = Vec::new();

            let label = format!("loop_end{}", ctx.curr_break);

            let inner_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
            let inner_instr = compile_expr_to_instr(e, inner_ctx)?;

            result_instr.extend(inner_instr);
            result_instr.push(Instr::Jmp(label.to_string()));

            Ok(result_instr)
        }, 
        Expr::Set(s, e) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let inner_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
            let inner_instr = compile_expr_to_instr(e, inner_ctx)?;

            result_instr.extend(inner_instr);

            match ctx.env.get(s) {
                Some(offset) => {
                    result_instr.push(Instr::MovToStack(Reg::Rax, offset * 8));
                    Ok(result_instr)
                }
                None => match ctx.define_ptrs.get(s) {
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
                let expr_ctx = &mut Context { env: ctx.env.clone(), ..*ctx };
                let expr_instr = compile_expr_to_instr(expr, expr_ctx)?;
                result_instr.extend(expr_instr);
            }

            Ok(result_instr)
        },
        Expr::Call(name, args) => {
            let mut result_instr: Vec<Instr> = Vec::new();
            let id = next_id();
            let aftercall_label = format!("after_call_{}_{}", name, id);
            let function_label = format!("function_{}_call_label", name);

            result_instr.push(Instr::Comment(format!("START of call to {} [{} arg{}]", name, args.len(), if args.len() == 1 { "" } else { "s" })));

            result_instr.push(Instr::MovLabel(aftercall_label.clone(), ctx.si * 8));

            let safe_si = 2 + ctx.si + args.len() as i32 + args.iter().map(|a| recursively_collet_depth(a)).max().unwrap_or(0);
            for (index, arg) in args.iter().enumerate() {
                let arc_ctx = &mut Context { si: safe_si, env: ctx.env.clone(), ..*ctx };
                let arg_instr = compile_expr_to_instr(arg, arc_ctx)?;
                result_instr.extend(arg_instr);
                let offset = (ctx.si + 1 + index as i32) * 8;
                result_instr.push(Instr::MovToStack(Reg::Rax, offset));
            }

            result_instr.push(Instr::Sub(Reg::Rsp, ctx.si * 8));
            result_instr.push(Instr::Jmp(function_label.clone()));
            result_instr.push(Instr::Label(aftercall_label));
            result_instr.push(Instr::Add(Reg::Rsp, ctx.si * 8));
            result_instr.push(Instr::Comment(format!("END of call to {}", name)));
            Ok(result_instr)
        }
    }
}

pub fn compile_defs_to_instr(defs: &Vec<Defenition>, ctx: &mut Context<'_>) -> std::io::Result<Vec<Instr>> {
    let mut result_instr: Vec<Instr> = Vec::new();
    for def in defs {
        match def {
            Defenition::Fun(name, params, body) => {
                result_instr.push(Instr::Comment(format!("START of function {}({})", name, params.join(", "))));

                let label = format!("function_{}_call_label", name.clone());
                result_instr.push(Instr::Label(label));

                let mut func_env: HashMap<String, i32> = HashMap::new();
                for (index, param) in params.iter().enumerate() {
                    func_env.insert(param.clone(), 1 + (index as i32));
                }

                let body_ctx = &mut Context { si: 1 + (params.len() as i32), env: func_env, curr_break: 0, ..*ctx };
                let body_instr = compile_expr_to_instr(body, body_ctx)?;
                result_instr.extend(body_instr);

                result_instr.push(Instr::JmpReg(Reg::Rsp));
                result_instr.push(Instr::Comment(format!("END of function {} definition", name)));
            }
        }
    }
    Ok(result_instr)
}