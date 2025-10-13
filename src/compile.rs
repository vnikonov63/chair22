use std::mem;
use std::collections::HashMap;
use std::collections::HashSet;

use dynasmrt::{dynasm, DynasmApi};

use crate::expressions::{Expr, ReplExpr, Op1, Op2};
use crate::instructions::{Reg, Instr, instr_to_dynasm};

pub fn compile_to_instr(e: &Expr, si: i32, env: HashMap<String, i32>, define_env: HashMap<String, i32>) -> std::io::Result<Vec<Instr>> {
    match e {
        Expr::Number(n) => Ok(vec![Instr::Mov(Reg::Rax, *n)]),
        Expr::Id(s) => {
            match env.get(s) {
                // the multiplication coerces/dereferences the &i32 here, but in the case
                // of compile_reple_to_instr I need to dereference it!
                Some(offset) => Ok(vec![Instr::MovFromStack(Reg::Rax, offset * 8)]),
                None => match define_env.get(s) {
                    Some(value) => Ok(vec![Instr::Mov(Reg::Rax, *value)]),
                    None => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Unbound variable identifier {}", s))),
                }
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
                let e_instr = compile_to_instr(e, curr_si, curr_env.clone(), define_env.clone())?;
                result_instr.extend(e_instr);
                result_instr.push(Instr::MovToStack(Reg::Rax, curr_si * 8));

                level.insert(v.clone());
                curr_env.insert(v.clone(), curr_si);
                curr_si += 1;
            }

            let b_instr = compile_to_instr(body, curr_si, curr_env, define_env.clone())?;
            result_instr.extend(b_instr);

            Ok(result_instr)
        },
        Expr::UnOp(op, e) => {
            let mut instr = compile_to_instr(e, si, env.clone(), define_env.clone())?;
            match op {
                Op1::Add1 => instr.push(Instr::Add(Reg::Rax, 1)),
                Op1::Sub1 => instr.push(Instr::Sub(Reg::Rax, 1)),
            }
            Ok(instr)
        },
        Expr::BinOp(op, e1, e2) => {
            let mut result_instr: Vec<Instr> = Vec::new();

            let stack_offset = si * 8;
            let e1_instr = compile_to_instr(e1, si, env.clone(), define_env.clone())?;
            let e2_instr = compile_to_instr(e2, si + 1, env.clone(), define_env.clone())?;

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
pub fn compile_repl_to_instr(
    e: &ReplExpr, si: i32, 
    define_env: &mut HashMap<String, i32>, 
    ops: &mut dynasmrt::x64::Assembler
) -> std::io::Result<Vec<Instr>> {
    match e {
        ReplExpr::Define(v, e) => {
            if define_env.contains_key(v) {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
            }
            
            let env = HashMap::new();
            let e_instr = compile_to_instr(e, si, env, define_env.clone())?;

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
                _ => {
                    let env = HashMap::new();
                    compile_to_instr(e, si, env, define_env.clone())
                }
            }
        }
    }
}