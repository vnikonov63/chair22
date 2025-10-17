use dynasmrt::{dynasm, DynasmApi};

#[derive(Debug, Clone)]
pub enum Reg {
    Rax,
}

pub fn reg_to_string(reg: &Reg) -> &str {
  match reg {
    Reg::Rax => "rax",
  }
}

#[derive(Debug, Clone)]
pub enum Instr {
    Mov(Reg, i32),         
    Add(Reg, i32),         
    Sub(Reg, i32),         
    AddRaxMemFromStack(i32),    
    SubRaxMemFromStack(i32),
    MulRaxMemFromStack(i32),
    MovToStack(Reg, i32),  
    MovFromStack(Reg, i32),
}

pub fn instr_to_string(instr: &Instr) -> String {
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

pub fn instrs_to_string(instrs: &Vec<Instr>) -> std::io::Result<String> {
  Ok(instrs.iter()
    .map(instr_to_string)
    .collect::<Vec<String>>()
    .join("\n"))
}

pub fn instr_to_dynasm(ops : &mut dynasmrt::x64::Assembler, instrs: &Vec<Instr>) -> std::io::Result<()> {
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