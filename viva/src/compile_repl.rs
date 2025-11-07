use std::mem;
use std::collections::HashMap;

use dynasmrt::{dynasm, DynasmApi};

use crate::compile::{compile_expr_to_instr, compile_defs_to_instr};
use crate::context::Context;
use crate::compile_helpers::{allocate_define_ptrs_for_set_targets};
use crate::expressions::{Expr, ReplExpr, Defenition};
use crate::instructions::{Instr, instr_to_dynasm};
use crate::runtime::snek_print;

pub fn compile_repl_to_instr(
    e: &ReplExpr, si: i32,
    define_env: &mut HashMap<String, i64>,
    ops: &mut dynasmrt::x64::Assembler,
    labels: &mut HashMap<String, dynasmrt::DynamicLabel>,
) -> std::io::Result<Vec<Instr>> {
    match e {
        ReplExpr::Define(v, e) => {
            if define_env.contains_key(v) {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
            }
            let result = compile_repl_and_persist(e, si, define_env, ops, labels, false)?;
            define_env.insert(v.clone(), result as i64);
            Ok(vec![])
        },
        ReplExpr::Fun(name, params, body) => {
            let empty_ptrs: HashMap<String, i64> = HashMap::new();
            let mut ctx = Context::new(&*define_env, &empty_ptrs).with_si(si);
            let defs = vec![Defenition::Fun(name.clone(), params.clone(), body.clone())];
            let def_instrs = compile_defs_to_instr(&defs, &mut ctx)?;

            instr_to_dynasm(ops, &def_instrs, labels)?;
            ops.commit().unwrap();
            Ok(vec![])
        },
        ReplExpr::Expr(e) => {
            let _ = compile_repl_and_persist(e, si, define_env, ops, labels, true)?;
            Ok(vec![])
        }
    }
}

pub fn compile_repl_and_persist(
    e: &Expr,
    si: i32,
    define_env: &mut HashMap<String, i64>,
    ops: &mut dynasmrt::x64::Assembler,
    labels: &mut HashMap<String, dynasmrt::DynamicLabel>,
    print_result: bool,
) -> std::io::Result<i64> {
    let define_ptrs = allocate_define_ptrs_for_set_targets(e, define_env);
    let mut ctx = Context::new(&*define_env, &define_ptrs).with_si(si);
    let e_instr = compile_expr_to_instr(e, &mut ctx)?;

    let start = ops.offset();
    dynasm!(ops ; .arch x64 ; push rbx ; push r12);
    instr_to_dynasm(ops, &e_instr, labels)?;
    if print_result {
        let snek_print_ptr = snek_print as *const ();
        let snek_print_addr = unsafe { std::mem::transmute::<* const (), fn() -> i32>(snek_print_ptr) } as i64;
        dynasm!(ops ; .arch x64 ; sub rsp, 8 ; mov rdi, rax ; mov rax, QWORD snek_print_addr ; call rax ; add rsp, 8 );
        dynasm!(ops ; .arch x64 ; pop r12 ; pop rbx ; ret);
    } else {
        dynasm!(ops ; .arch x64 ; pop r12 ; pop rbx ; ret);
    }
    ops.commit().unwrap();
    let reader = ops.reader();
    let buf = reader.lock();
    let jitted_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(buf.ptr(start)) };
    let result = jitted_fn();

    unsafe {
        for (name, ptr) in define_ptrs.into_iter() {
            let boxed = Box::from_raw(ptr as *mut i64);
            let val = *boxed;
            define_env.insert(name, val);
        }
    }

    Ok(result)
}