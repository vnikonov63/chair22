use dynasmrt::{dynasm, DynasmApi};
use dynasmrt::DynasmLabelApi;
use std::collections::HashMap;
use std::mem;

use crate::runtime::{snek_error};

#[derive(Debug, Clone)]
pub enum Reg {
    Rax, Rcx, Rdx, Rbx,
    Rsp, Rbp, Rsi, Rdi,
    R8, R9, R10, R11, 
    R12, R13, R14, R15,
}

pub fn reg_to_number(reg: &Reg) -> u8 {
    match *reg {
        Reg::Rax => 0,  Reg::Rcx => 1,  Reg::Rdx => 2,  Reg::Rbx => 3,
        Reg::Rsp => 4,  Reg::Rbp => 5,  Reg::Rsi => 6,  Reg::Rdi => 7,
        Reg::R8  => 8,  Reg::R9  => 9,  Reg::R10 => 10, Reg::R11 => 11,
        Reg::R12 => 12, Reg::R13 => 13, Reg::R14 => 14, Reg::R15 => 15,
    }
}

pub fn reg_to_string(reg: &Reg) -> &str {
    match reg {
        Reg::Rax => "rax", Reg::Rcx => "rcx", Reg::Rdx => "rdx", Reg::Rbx => "rbx",
        Reg::Rsp => "rsp", Reg::Rbp => "rbp", Reg::Rsi => "rsi", Reg::Rdi => "rdi",
        Reg::R8  => "r8", Reg::R9  => "r9", Reg::R10 => "r10", Reg::R11 => "r11",
        Reg::R12 => "r12", Reg::R13 => "r13", Reg::R14 => "r14", Reg::R15 => "r15",
    }
}

#[derive(Debug, Clone)]
pub enum Instr {
    Mov(Reg, i64),
    MovFromReg(Reg, Reg),
    MovRaxFromRaxPtr,
    MovToPtrFromReg(Reg, Reg),
    Add(Reg, i32),
    Sub(Reg, i32),
    And(Reg, Reg),
    Or(Reg, Reg),
    Xor(Reg, Reg),
    AddRaxMemFromStack(i32),
    SubRaxMemFromStack(i32),
    MulRaxMemFromStack(i32),
    MovToStack(Reg, i32),
    MovFromStack(Reg, i32),
    Label(String),
    Compare(Reg),
    CompareWithMemory(Reg, i32),
    CompareRegs(Reg, Reg),
    Test(Reg, i32),
    Jmp(String),
    Je(String),
    Jne(String),
    Jl(String),
    Jle(String),
    Jg(String),
    Jge(String),
    Jno(String),
    Cmove(Reg, Reg),
    Cmovne(Reg, Reg),
    Cmovl(Reg, Reg),
    Cmovle(Reg, Reg),
    Cmovg(Reg, Reg),
    Cmovge(Reg, Reg),
    ShiftArithmeticRight(Reg, i8),
    CallRustError(i8),
    Comment(String)
}

pub fn instr_to_string(instr: &Instr) -> String {
    match instr {
        Instr::Mov(reg, val) => format!("mov {}, {}", reg_to_string(reg), val),
        Instr::MovFromReg(regd, regs) => format!("mov {}, {}", reg_to_string(regd), reg_to_string(regs)),
        Instr::MovRaxFromRaxPtr => "mov rax, [rax]".to_string(),
        Instr::MovToPtrFromReg(ptr, src) => format!("mov [{}], {}", reg_to_string(ptr), reg_to_string(src)),
        Instr::Add(reg, val) => format!("add {}, {}", reg_to_string(reg), val),
        Instr::Sub(reg, val) => format!("sub {}, {}", reg_to_string(reg), val),
        Instr::And(dst, src) => format!("and {}, {}", reg_to_string(dst), reg_to_string(src)), 
        Instr::Or(dst, src) => format!("or {}, {}", reg_to_string(dst), reg_to_string(src)),
        Instr::Xor(dst, src) => format!("xor {}, {}", reg_to_string(dst), reg_to_string(src)),
        Instr::AddRaxMemFromStack(offset) => format!("add rax, [rsp - {}]", offset),
        Instr::SubRaxMemFromStack(offset) => format!("sub rax, [rsp - {}]", offset),
        Instr::MulRaxMemFromStack(offset) => format!("imul rax, [rsp - {}]", offset),
        Instr::MovToStack(reg, offset) => format!("mov [rsp - {}], {}", offset, reg_to_string(reg)),
        Instr::MovFromStack(reg, offset) => format!("mov {}, [rsp - {}]", reg_to_string(reg), offset),
        Instr::Label(label) => format!("{}:", label),
        Instr::Compare(reg) => format!("cmp {}, 3", reg_to_string(reg)),
        Instr::CompareWithMemory(reg, offset) => format!("cmp {}, [rsp - {}]", reg_to_string(reg), offset),
        Instr::CompareRegs(r1, r2) => format!("cmp {}, {}", reg_to_string(r1), reg_to_string(r2)),
        Instr::Test(reg, val) => format!("test {}, {}", reg_to_string(reg), val),
        Instr::Jmp(label) => format!("jmp {}", label),
        Instr::Je(label) => format!("je {}", label),
        Instr::Jne(label) => format!("jne {}", label),
        Instr::Jl(label) => format!("jl {}", label),
        Instr::Jle(label) => format!("jle {}", label),
        Instr::Jg(label) => format!("jg {}", label),
        Instr::Jge(label) => format!("jge {}", label),
        Instr::Jno(label) => format!("jno {}", label),
        Instr::Cmove(reg1, reg2) => format!("cmove {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovne(reg1, reg2) => format!("cmovne {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovl(reg1, reg2) => format!("cmovl {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovle(reg1, reg2) => format!("cmovle {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovg(reg1, reg2) => format!("cmovg {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovge(reg1, reg2) => format!("cmovge {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::ShiftArithmeticRight(reg, val) => format!("sar {}, {}", reg_to_string(reg), val),
        Instr::CallRustError(err_code) => format!("mov rdi, {}\n  call snek_error", err_code),
        Instr::Comment(s) => format!("; {}", s)
    }
}

pub fn instrs_to_string(instrs: &Vec<Instr>) -> std::io::Result<String> {
    Ok(instrs
        .iter()
        .map(instr_to_string)
        .collect::<Vec<String>>()
        .join("\n"))
}

pub fn instr_to_dynasm(ops: &mut dynasmrt::x64::Assembler, instrs: &Vec<Instr>) -> std::io::Result<()> {
    let mut labels: HashMap<String, dynasmrt::DynamicLabel> = HashMap::new();

    for instr in instrs.iter() {
        if let Instr::Label(label) = instr {
            if !labels.contains_key(label) {
                let dl = ops.new_dynamic_label();
                labels.insert(label.clone(), dl);
            }
        }
    }

    for instr in instrs.iter() {
        match instr { 
            Instr::Mov(reg, val) => { dynasm!(ops; .arch x64; mov Rq(reg_to_number(reg)), QWORD *val); }
            Instr::MovFromReg(dest, src) => { dynasm!(ops; .arch x64; mov Rq(reg_to_number(dest)), Rq(reg_to_number(src))); }
            /* Think aout this, it is not good no hardcode the stuff */
            Instr::MovRaxFromRaxPtr => { dynasm!(ops; .arch x64; mov rax, [rax]); }
            Instr::MovToPtrFromReg(ptr, src) => { dynasm!(ops; .arch x64; mov [Rq(reg_to_number(ptr))], Rq(reg_to_number(src))); }
            Instr::Add(reg, val) => { dynasm!(ops; .arch x64; add Rq(reg_to_number(reg)), *val); }
            Instr::Sub(reg, val) => { dynasm!(ops; .arch x64; sub Rq(reg_to_number(reg)), *val); }
            Instr::And(dst, src) => { dynasm!(ops; .arch x64; and Rq(reg_to_number(dst)), Rq(reg_to_number(src))); }
            Instr::Or(dst, src) => { dynasm!(ops; .arch x64; or Rq(reg_to_number(dst)), Rq(reg_to_number(src))); }
            Instr::Xor(dst, src) => { dynasm!(ops; .arch x64; xor Rq(reg_to_number(dst)), Rq(reg_to_number(src))); }

            /* There is no need to have separate commands to this stuff below, but this will require changing the way I 
               call stuff, so the edit would be happening the next time. 
             */
            Instr::AddRaxMemFromStack(offset) => { dynasm!(ops; .arch x64; add rax, [rsp - *offset]); }
            Instr::SubRaxMemFromStack(offset) => { dynasm!(ops; .arch x64; sub rax, [rsp - *offset]); }
            Instr::MulRaxMemFromStack(offset) => { dynasm!(ops; .arch x64; imul rax, [rsp - *offset]); }


            Instr::MovToStack(reg, offset) => { dynasm!(ops; .arch x64; mov QWORD [rsp - *offset], Rq(reg_to_number(reg))); }
            Instr::MovFromStack(reg, offset) => { dynasm!(ops; .arch x64; mov Rq(reg_to_number(reg)), [rsp - *offset]); }
            Instr::Label(label) => { dynasm!(ops; .arch x64; =>labels[label].clone()); }
            Instr::Compare(reg) => { dynasm!(ops; .arch x64; cmp Rq(reg_to_number(reg)), 3); }
            Instr::CompareWithMemory(reg, offset) => { dynasm!(ops; .arch x64; cmp Rq(reg_to_number(reg)), [rsp - *offset])}
            Instr::CompareRegs(reg1, reg2) => { dynasm!(ops; .arch x64; cmp Rq(reg_to_number(reg1)), Rq(reg_to_number(reg2))); }
            Instr::Test(reg, val) => { dynasm!(ops; .arch x64; test Rq(reg_to_number(reg)), *val); },
            Instr::Jmp(label) => { dynasm!(ops; .arch x64; jmp =>labels[label].clone()); },
            Instr::Je(label) => {dynasm!(ops; .arch x64; je => labels[label].clone()); },
            Instr::Jne(label) => { dynasm!(ops; .arch x64; jne =>labels[label].clone()); },
            Instr::Jl(label) => { dynasm!(ops; .arch x64; jl =>labels[label].clone()); },
            Instr::Jle(label) => { dynasm!(ops; .arch x64; jle =>labels[label].clone()); },
            Instr::Jg(label) => { dynasm!(ops; .arch x64; jg =>labels[label].clone()); },
            Instr::Jge(label) => { dynasm!(ops; .arch x64; jge =>labels[label].clone()); },
            Instr::Jno(label) => { dynasm!(ops; .arch x64; jno =>labels[label].clone()); },
            Instr::Cmove(dest, src) => { dynasm!(ops; .arch x64; cmove Rq(reg_to_number(dest)), Rq(reg_to_number(src))); },
            Instr::Cmovne(dest, src) => { dynasm!(ops; .arch x64; cmovne Rq(reg_to_number(dest)), Rq(reg_to_number(src))); },
            Instr::Cmovl(dest, src) => { dynasm!(ops; .arch x64; cmovl Rq(reg_to_number(dest)), Rq(reg_to_number(src))); },
            Instr::Cmovle(dest, src) => { dynasm!(ops; .arch x64; cmovle Rq(reg_to_number(dest)), Rq(reg_to_number(src))); },
            Instr::Cmovg(dest, src) => { dynasm!(ops; .arch x64; cmovg Rq(reg_to_number(dest)), Rq(reg_to_number(src))); },
            Instr::Cmovge(dest, src) => { dynasm!(ops; .arch x64; cmovge Rq(reg_to_number(dest)), Rq(reg_to_number(src))); },
            Instr::ShiftArithmeticRight(reg, val) => { dynasm!(ops; .arch x64; sar Rq(reg_to_number(reg)), *val as i8); }
            Instr::CallRustError(err_code) => {
                let snek_error_ptr = snek_error as *const ();
                let snek_error_addr = unsafe { mem::transmute::<* const (), fn() -> i32>(snek_error_ptr) } as i64;
            
                dynasm!(ops; .arch x64; mov rdi, *err_code as i32);
                dynasm!(ops; .arch x64; mov rax, QWORD snek_error_addr as _);
                dynasm!(ops; .arch x64; call rax);
            },
            Instr::Comment(str) => {}
        }
    }

    Ok(())
}
