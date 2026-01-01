use crate::cpu::{Cpu, StepResult};
use crate::bus::Bus;

pub(super) trait Instruction {
    fn execute(&self, cpu: &mut Cpu, bus: &mut dyn Bus) -> StepResult;
    fn get_inst_bin(&self) -> u32;
}

pub(super) struct RType {
    pub inst_bin: u32,
    pub rd:  usize,
    pub rs1: usize,
    pub rs2: usize,
    pub executor: fn(cpu: &mut Cpu, rd: usize, rs1: usize, rs2: usize) -> StepResult,
}

impl Instruction for RType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.rs1, self.rs2)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct IType {
    pub inst_bin: u32,
    pub rd:  usize,
    pub rs1: usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, rs1: usize, imm: u32) -> StepResult,
}

impl Instruction for IType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.rs1, self.imm)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct ITypeLoad {
    pub inst_bin: u32,
    pub rd:  usize,
    pub rs1: usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, rs1: usize, imm: u32, bus: &mut dyn Bus) -> StepResult,
}

impl Instruction for ITypeLoad {
    fn execute(&self, cpu: &mut Cpu, bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.rs1, self.imm, bus)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct SType {
    pub inst_bin: u32,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rs1: usize, rs2: usize, imm: u32, bus: &mut dyn Bus) -> StepResult,
}

impl Instruction for SType {
    fn execute(&self, cpu: &mut Cpu, bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rs1, self.rs2, self.imm, bus)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct BType {
    pub inst_bin: u32,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rs1: usize, rs2: usize, imm: u32) -> StepResult,
}

impl Instruction for BType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rs1, self.rs2, self.imm)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct UType {
    pub inst_bin: u32,
    pub rd:  usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, imm: u32) -> StepResult,
}

impl Instruction for UType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.imm)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct JType {
    pub inst_bin: u32,
    pub rd:  usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, imm: u32) -> StepResult,
}

impl Instruction for JType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.imm)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

pub(super) struct CType {
    pub inst_bin: u16,
    pub rd:  usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, imm: u32) -> StepResult,
}

impl Instruction for CType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.imm)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin as u32 }
}

pub(super) struct CTypeLoad {
    pub inst_bin: u16,
    pub rd:  usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, imm: u32, bus: &mut dyn Bus) -> StepResult,
}

impl Instruction for CTypeLoad {
    fn execute(&self, cpu: &mut Cpu, bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.imm, bus)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin as u32 }
}

pub(super) struct CSRType {
    pub inst_bin: u32,
    pub rd:       usize,
    pub rs1_uimm: usize, // can be rs1 or uimm
    pub csr_addr: u32,
    pub executor: fn(cpu: &mut Cpu, rd: usize, rs1_uimm: usize, csr_addr: u32) -> StepResult,
}

impl Instruction for CSRType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.rd, self.rs1_uimm, self.csr_addr)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

// Special cases like mret, ecall, ebreak, fence
pub(super) struct NoArgsType {
    pub inst_bin: u32,
    pub executor: fn(cpu: &mut Cpu) -> StepResult,
}

impl Instruction for NoArgsType {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin }
}

// Special case for c.jr and c.jalr (one register)
pub(super) struct CTypeReg {
    pub inst_bin: u16,
    pub reg: usize,
    pub executor: fn(cpu: &mut Cpu, reg: usize) -> StepResult,
}

impl Instruction for CTypeReg {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.reg)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin as u32 }
}

// Special case for c.sw and c.lw (rs1, rs2/rd, imm)
pub(super) struct CTypeThree {
    pub inst_bin: u16,
    pub r1:  usize,
    pub r2:  usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, r1: usize, r2: usize, imm: u32) -> StepResult,
}

impl Instruction for CTypeThree {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.r1, self.r2, self.imm)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin as u32 }
}

pub(super) struct CTypeThreeBus {
    pub inst_bin: u16,
    pub r1:  usize,
    pub r2:  usize,
    pub imm: u32,
    pub executor: fn(cpu: &mut Cpu, r1: usize, r2: usize, imm: u32, bus: &mut dyn Bus) -> StepResult,
}

impl Instruction for CTypeThreeBus {
    fn execute(&self, cpu: &mut Cpu, bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.r1, self.r2, self.imm, bus)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin as u32 }
}

pub(super) struct CTypeTwo {
    pub inst_bin: u16,
    pub r1:  usize,
    pub r2:  usize,
    pub executor: fn(cpu: &mut Cpu, r1: usize, r2: usize) -> StepResult,
}

impl Instruction for CTypeTwo {
    fn execute(&self, cpu: &mut Cpu, _bus: &mut dyn Bus) -> StepResult {
        (self.executor)(cpu, self.r1, self.r2)
    }
    fn get_inst_bin(&self) -> u32 { self.inst_bin as u32 }
}
