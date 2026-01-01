use crate::cpu::{Cpu, StepResult};

impl Cpu {
    pub(crate) fn c_addi4spn(&mut self, rd: usize, imm: u32) -> StepResult {
        if imm == 0 {
            return StepResult::Trap(2); // Reserved
        }
        self.regs[rd] = self.regs[2].wrapping_add(imm);
        StepResult::Ok
    }

    pub(crate) fn c_lw(&mut self, rd: usize, rs1: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read32(addr);
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn c_lwsp(&mut self, rd: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        if rd == 0 {
            return StepResult::Trap(2); // Reserved
        }
        let addr = self.regs[2].wrapping_add(imm);
        self.regs[rd] = bus.read32(addr);
        StepResult::Ok
    }

    pub(crate) fn c_swsp(&mut self, rs2: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[2].wrapping_add(imm);
        bus.write32(addr, self.regs[rs2]);
        StepResult::Ok
    }

    pub(crate) fn c_jr(&mut self, rs1: usize) -> StepResult {
        if rs1 == 0 {
            return StepResult::Trap(2); // C.JR: rs1 != 0
        }
        self.pc = self.regs[rs1] & !1;
        StepResult::Jumped
    }

    pub(crate) fn c_mv(&mut self, rd: usize, rs2: usize) -> StepResult {
        if rd == 0 || rs2 == 0 {
            return StepResult::Trap(2); // C.MV: rd != 0, rs2 != 0
        }
        self.regs[rd] = self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_sw(&mut self, rs1: usize, rs2: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        bus.write32(addr, self.regs[rs2]);
        StepResult::Ok
    }

    pub(crate) fn c_addi(&mut self, rd: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rd].wrapping_add(imm);
        }
        StepResult::Ok
    }

    pub(crate) fn c_jal(&mut self, imm: u32) -> StepResult {
        self.regs[1] = self.pc.wrapping_add(2);
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    pub(crate) fn c_li(&mut self, rd: usize, imm: u32) -> StepResult {
        if rd == 0 {
            return StepResult::Trap(2); // Reserved for HINTs
        }
        self.regs[rd] = imm;
        StepResult::Ok
    }

    pub(crate) fn c_addi16sp(&mut self, imm: u32) -> StepResult {
        if imm == 0 {
            return StepResult::Trap(2); // Reserved
        }
        self.regs[2] = self.regs[2].wrapping_add(imm);
        StepResult::Ok
    }

    pub(crate) fn c_lui(&mut self, rd: usize, imm: u32) -> StepResult {
        if rd == 0 || rd == 2 {
            return StepResult::Trap(2); // rd=0 is reserved, rd=2 is C.ADDI16SP
        }
        let imm_u32 = imm << 12;
        self.regs[rd] = imm_u32;
        StepResult::Ok
    }

    pub(crate) fn c_srli(&mut self, rd: usize, shamt: u32) -> StepResult {
        if shamt == 0 {
            // shamt=0 is reserved for HINTs
            return StepResult::Ok;
        }
        self.regs[rd] >>= shamt;
        StepResult::Ok
    }

    pub(crate) fn c_srai(&mut self, rd: usize, shamt: u32) -> StepResult {
        if shamt == 0 {
            // shamt=0 is reserved for HINTs
            return StepResult::Ok;
        }
        self.regs[rd] = ((self.regs[rd] as i32) >> shamt) as u32;
        StepResult::Ok
    }

    pub(crate) fn c_andi(&mut self, rd: usize, imm: u32) -> StepResult {
        self.regs[rd] &= imm;
        StepResult::Ok
    }

    pub(crate) fn c_sub(&mut self, rd: usize, rs2: usize) -> StepResult {
        self.regs[rd] = self.regs[rd].wrapping_sub(self.regs[rs2]);
        StepResult::Ok
    }

    pub(crate) fn c_xor(&mut self, rd: usize, rs2: usize) -> StepResult {
        self.regs[rd] ^= self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_or(&mut self, rd: usize, rs2: usize) -> StepResult {
        self.regs[rd] |= self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_and(&mut self, rd: usize, rs2: usize) -> StepResult {
        self.regs[rd] &= self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_j(&mut self, imm: u32) -> StepResult {
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    pub(crate) fn c_beqz(&mut self, rs1: usize, imm: u32) -> StepResult {
        if self.regs[rs1] == 0 {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn c_bnez(&mut self, rs1: usize, imm: u32) -> StepResult {
        if self.regs[rs1] != 0 {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn c_slli(&mut self, rd: usize, shamt: u32) -> StepResult {
        if rd == 0 {
            return StepResult::Trap(2); // Reserved
        }
        if shamt == 0 {
            // shamt=0 is reserved for HINTs
            return StepResult::Ok;
        }
        self.regs[rd] <<= shamt;
        StepResult::Ok
    }

    pub(crate) fn c_jalr(&mut self, rs1: usize) -> StepResult {
        if rs1 == 0 {
            return StepResult::Trap(2);
        }
        let next_pc = self.pc + 2;
        self.pc = self.regs[rs1] & !1;
        self.regs[1] = next_pc;
        StepResult::Jumped
    }

    pub(crate) fn c_add(&mut self, rd: usize, rs2: usize) -> StepResult {
        if rd == 0 || rs2 == 0 {
             return StepResult::Trap(2);
        }
        self.regs[rd] = self.regs[rd].wrapping_add(self.regs[rs2]);
        StepResult::Ok
    }
}
