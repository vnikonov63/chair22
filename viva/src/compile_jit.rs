use std::mem;
use std::collections::HashMap;

use dynasmrt::{dynasm, DynasmApi};

use crate::compile::compile_to_instr;
use crate::compile_helpers::allocate_define_ptrs_for_set_targets;
use crate::expressions::{Expr, ReplExpr};
use crate::instructions::{Instr, instr_to_dynasm};
use crate::runtime::snek_print;

pub fn compile_repl_to_instr(
    e: &ReplExpr, si: i32,
    define_env: &mut HashMap<String, i64>,
    ops: &mut dynasmrt::x64::Assembler
) -> std::io::Result<Vec<Instr>> {
    match e {
        ReplExpr::Define(v, e) => {
            if define_env.contains_key(v) {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Duplicate binding"));
            }
            let result = compile_jit_and_persist(e, si, define_env, ops, false)?;
            define_env.insert(v.clone(), result as i64);
            Ok(vec![])
        },
        ReplExpr::Expr(e) => {
            let _ = compile_jit_and_persist(e, si, define_env, ops, true)?;
            Ok(vec![])
        }
    }
}

pub fn compile_jit_and_persist(
    e: &Expr,
    si: i32,
    define_env: &mut HashMap<String, i64>,
    ops: &mut dynasmrt::x64::Assembler,
    print_result: bool,
) -> std::io::Result<i64> {
    let define_ptrs = allocate_define_ptrs_for_set_targets(e, define_env);
    let env = HashMap::new();
    let e_instr = compile_to_instr(e, si, env, define_env, &define_ptrs, 0)?;

    let start = ops.offset();
    instr_to_dynasm(ops, &e_instr)?;
    if print_result {
        let snek_print_ptr = snek_print as *const ();
        let snek_print_addr = unsafe { std::mem::transmute::<* const (), fn() -> i32>(snek_print_ptr) } as i64;
        dynasm!(ops ; .arch x64 ; sub rsp, 16 ; mov rdi, rax ; mov rax, QWORD snek_print_addr ; call rax ; add rsp, 16 ; ret);
    } else {
        dynasm!(ops ; .arch x64 ; ret);
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