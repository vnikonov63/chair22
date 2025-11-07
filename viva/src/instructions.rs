use dynasmrt::{dynasm, DynasmApi};
use dynasmrt::DynasmLabelApi;
use std::collections::HashMap;
use std::mem;

use crate::runtime::{snek_error, snek_print};

#[derive(Debug, Clone)]
pub enum Reg {
    Rax, /* Rcx, */ Rdx, Rbx,
    Rsp, /* Rbp, */ /* Rsi, */ Rdi,
    R8, /* R9, */ R10, R11, 
    R12, /* R13, */ /* R14, */ /* R15, */
}

pub fn reg_to_number(reg: &Reg) -> u8 {
    match *reg {
        Reg::Rax => 0,  /* Reg::Rcx => 1, */  Reg::Rdx => 2,  Reg::Rbx => 3,
        Reg::Rsp => 4,  /* Reg::Rbp => 5, */  /* Reg::Rsi => 6,  */ Reg::Rdi => 7,
        Reg::R8  => 8,  /* Reg::R9  => 9, */  Reg::R10 => 10, Reg::R11 => 11,
        Reg::R12 => 12, /* Reg::R13 => 13, */ /* Reg::R14 => 14, */ /* Reg::R15 => 15, */
    }
}

pub fn reg_to_string(reg: &Reg) -> &str {
    match reg {
        Reg::Rax => "rax", /* Reg::Rcx => "rcx", */ Reg::Rdx => "rdx", Reg::Rbx => "rbx",
        Reg::Rsp => "rsp", /* Reg::Rbp => "rbp", */ /* Reg::Rsi => "rsi", */ Reg::Rdi => "rdi",
        Reg::R8  => "r8", /* Reg::R9  => "r9", */ Reg::R10 => "r10", Reg::R11 => "r11",
        Reg::R12 => "r12", /* Reg::R13 => "r13", */ /* Reg::R14 => "r14", */ /* Reg::R15 => "r15", */
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
    /* And(Reg, Reg), */
    Or(Reg, Reg),
    Xor(Reg, Reg),
    AddRaxMemFromStack(i32),
    SubRaxMemFromStack(i32),
    MulRaxMemFromStack(i32),
    MovToStack(Reg, i32),
    MovFromStack(Reg, i32),
    MovLabel(String, i32),
    Label(String),
    Compare(Reg),
    CompareWithMemory(Reg, i32),
    /* CompareRegs(Reg, Reg), */
    Test(Reg, i32),
    Jmp(String),
    JmpReg(Reg),
    Je(String),
    Jne(String),
    /* Jl(String), */
    /* Jle(String), */
    /* Jg(String), */
    /* Jge(String), */
    Jno(String),
    Cmove(Reg, Reg),
    Cmovne(Reg, Reg),
    Cmovl(Reg, Reg),
    Cmovle(Reg, Reg),
    Cmovg(Reg, Reg),
    Cmovge(Reg, Reg),
    ShiftArithmeticRight(Reg, i8),
    CallRustError(i8),
    CallRustPrint(Reg),
    Comment(String),
    /* Ret, */
}

pub fn instr_to_string(instr: &Instr) -> String {
    match instr {
        Instr::Mov(reg, val) => format!("\tmov {}, {}", reg_to_string(reg), val),
        Instr::MovFromReg(regd, regs) => format!("\tmov {}, {}", reg_to_string(regd), reg_to_string(regs)),
        Instr::MovRaxFromRaxPtr => "\tmov rax, [rax]".to_string(),
        Instr::MovToPtrFromReg(ptr, src) => format!("\tmov [{}], {}", reg_to_string(ptr), reg_to_string(src)),
        Instr::Add(reg, val) => format!("\tadd {}, {}", reg_to_string(reg), val),
        Instr::Sub(reg, val) => format!("\tsub {}, {}", reg_to_string(reg), val),
        /* Instr::And(dst, src) => format!("\tand {}, {}", reg_to_string(dst), reg_to_string(src)), */ 
        Instr::Or(dst, src) => format!("\tor {}, {}", reg_to_string(dst), reg_to_string(src)),
        Instr::Xor(dst, src) => format!("\txor {}, {}", reg_to_string(dst), reg_to_string(src)),
        Instr::AddRaxMemFromStack(offset) => format!("\tadd rax, [rsp - {}]", offset),
        Instr::SubRaxMemFromStack(offset) => format!("\tsub rax, [rsp - {}]", offset),
        Instr::MulRaxMemFromStack(offset) => format!("\timul rax, [rsp - {}]", offset),
        Instr::MovToStack(reg, offset) => format!("\tmov [rsp - {}], {}", offset, reg_to_string(reg)),
        Instr::MovFromStack(reg, offset) => format!("\tmov {}, [rsp - {}]", reg_to_string(reg), offset),
        /* Instr::MovLabel(label, offset) => format!("\tlea r8, [rel {}]\n\tmov [rsp - {}], r8", label, offset), */
        Instr::MovLabel(label, offset) => format!("\tlea rax, [rel {}]\n\tmov QWORD [rsp - {}], rax", label, offset),
        Instr::Label(label) => format!("{}:", label),
        Instr::Compare(reg) => format!("\tcmp {}, 3", reg_to_string(reg)),
        Instr::CompareWithMemory(reg, offset) => format!("\tcmp {}, [rsp - {}]", reg_to_string(reg), offset),
        /* Instr::CompareRegs(r1, r2) => format!("\tcmp {}, {}", reg_to_string(r1), reg_to_string(r2)), */
        Instr::Test(reg, val) => format!("\ttest {}, {}", reg_to_string(reg), val),
        Instr::Jmp(label) => format!("\tjmp {}", label),
        Instr::JmpReg(reg) => format!("\tjmp QWORD [{}]", reg_to_string(reg)),
        Instr::Je(label) => format!("\tje {}", label),
        Instr::Jne(label) => format!("\tjne {}", label),
        /* Instr::Jl(label) => format!("\tjl {}", label), */
        /* Instr::Jle(label) => format!("\tjle {}", label), */
        /* Instr::Jg(label) => format!("\tjg {}", label), */
        /* Instr::Jge(label) => format!("\tjge {}", label), */
        Instr::Jno(label) => format!("\tjno {}", label),
        Instr::Cmove(reg1, reg2) => format!("\tcmove {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovne(reg1, reg2) => format!("\tcmovne {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovl(reg1, reg2) => format!("\tcmovl {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovle(reg1, reg2) => format!("\tcmovle {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovg(reg1, reg2) => format!("\tcmovg {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::Cmovge(reg1, reg2) => format!("\tcmovge {}, {}", reg_to_string(reg1), reg_to_string(reg2)),
        Instr::ShiftArithmeticRight(reg, val) => format!("\tsar {}, {}", reg_to_string(reg), val),
        Instr::CallRustError(err_code) => format!("\tmov rdi, {}\n\tcall snek_error", err_code),
        Instr::CallRustPrint(reg) => format!("\tsub rsp, 8\n\tmov rdi, {}\n\tcall snek_print\n\tadd rsp, 8", reg_to_string(reg)),
        Instr::Comment(s) => format!("; {}", s),
        /* Instr::Ret => format!("\tret"), */
    }
}

pub fn instrs_to_string(instrs: &Vec<Instr>) -> std::io::Result<String> {
    Ok(instrs
        .iter()
        .map(instr_to_string)
        .collect::<Vec<String>>()
        .join("\n"))
}

pub fn instr_to_dynasm(
    ops: &mut dynasmrt::x64::Assembler,
    instrs: &Vec<Instr>,
    labels: &mut HashMap<String, dynasmrt::DynamicLabel>,
) -> std::io::Result<()> {
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
            /* Instr::And(dst, src) => { dynasm!(ops; .arch x64; and Rq(reg_to_number(dst)), Rq(reg_to_number(src))); } */
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
            /* Instr::MovLabel(label, offset) => { dynasm!(ops; .arch x64; lea r8, [=>labels[label]]; mov [rsp - *offset], r8); } */
            Instr::MovLabel(label, offset) => { dynasm!(ops; .arch x64; lea rax, [=>labels[label]]; mov QWORD [rsp - *offset], rax); }
            Instr::Label(label) => { dynasm!(ops; .arch x64; =>labels[label].clone()); }
            Instr::Compare(reg) => { dynasm!(ops; .arch x64; cmp Rq(reg_to_number(reg)), 3); }
            Instr::CompareWithMemory(reg, offset) => { dynasm!(ops; .arch x64; cmp Rq(reg_to_number(reg)), [rsp - *offset])}
            /* Instr::CompareRegs(reg1, reg2) => { dynasm!(ops; .arch x64; cmp Rq(reg_to_number(reg1)), Rq(reg_to_number(reg2))); } */
            Instr::Test(reg, val) => { dynasm!(ops; .arch x64; test Rq(reg_to_number(reg)), *val); },
            Instr::Jmp(label) => { dynasm!(ops; .arch x64; jmp =>labels[label].clone()); },
            Instr::JmpReg(reg) => { dynasm!(ops; .arch x64; jmp QWORD [Rq(reg_to_number(reg))]) },
            Instr::Je(label) => {dynasm!(ops; .arch x64; je =>labels[label].clone()); },
            Instr::Jne(label) => { dynasm!(ops; .arch x64; jne =>labels[label].clone()); },
            /* Instr::Jl(label) => { dynasm!(ops; .arch x64; jl =>labels[label].clone()); }, */
            /* Instr::Jle(label) => { dynasm!(ops; .arch x64; jle =>labels[label].clone()); }, */
            /* Instr::Jg(label) => { dynasm!(ops; .arch x64; jg =>labels[label].clone()); }, */
            /* Instr::Jge(label) => { dynasm!(ops; .arch x64; jge =>labels[label].clone()); }, */
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
                dynasm!(ops; .arch x64; sub rsp, 8);
                dynasm!(ops; .arch x64; mov rdi, *err_code as i32);
                dynasm!(ops; .arch x64; mov rax, QWORD snek_error_addr as _);
                dynasm!(ops; .arch x64; call rax);
                dynasm!(ops; .arch x64; add rsp, 8);
            },
            Instr::CallRustPrint(reg) => {
                let snek_print_ptr = snek_print as *const ();
                let snek_print_addr = unsafe { mem::transmute::<* const (), fn() -> i32>(snek_print_ptr) } as i64;
                dynasm!(ops; .arch x64; sub rsp, 8);
                dynasm!(ops; .arch x64; mov rdi, Rq(reg_to_number(reg)));
                dynasm!(ops; .arch x64; mov rax, QWORD snek_print_addr as _);
                dynasm!(ops; .arch x64; call rax);
                dynasm!(ops; .arch x64; add rsp, 8);
            }
            Instr::Comment(_) => {},
            /* Instr::Ret => { dynasm!(ops; .arch x64; ret); } */
        }
    }

    Ok(())
}
