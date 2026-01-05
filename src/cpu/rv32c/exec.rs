use crate::cpu::{Cpu, StepResult};

impl Cpu {
    #[inline(always)]
    pub(crate) fn c_addi4spn(&mut self, rd: u8, _rs1: u8, imm: u16) -> StepResult {
        let rd = rd as usize;
        let imm = imm as u32;
        if imm == 0 {
            return StepResult::Trap(2); // Reserved
        }
        self.regs[rd] = self.regs[2].wrapping_add(imm);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_lw<B: crate::bus::Bus>(&mut self, rd: u8, rs1: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd = rd as usize;
        let rs1 = rs1 as usize;
        let imm = imm as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        self.regs[rd] = bus.read32(addr);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_lwsp<B: crate::bus::Bus>(&mut self, rd: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd = rd as usize;
        let imm = imm as u32;
        if rd == 0 {
            return StepResult::Trap(2); // Reserved
        }
        let addr = self.regs[2].wrapping_add(imm);
        self.regs[rd] = bus.read32(addr);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_swsp<B: crate::bus::Bus>(&mut self, rs2: u8, imm: u16, bus: &mut B) -> StepResult {
        let rs2 = rs2 as usize;
        let imm = imm as u32;
        let addr = self.regs[2].wrapping_add(imm);
        bus.write32(addr, self.regs[rs2]);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_jr(&mut self, rs1: u8) -> StepResult {
        let rs1 = rs1 as usize;
        if rs1 == 0 {
            return StepResult::Trap(2); // C.JR: rs1 != 0
        }
        self.pc = self.regs[rs1] & !1;
        StepResult::Jumped
    }

    #[inline(always)]
    pub(crate) fn c_mv(&mut self, rd: u8, rs2: u8) -> StepResult {
        let rd = rd as usize;
        let rs2 = rs2 as usize;
        if rd == 0 || rs2 == 0 {
            return StepResult::Trap(2); // C.MV: rd != 0, rs2 != 0
        }
        self.regs[rd] = self.regs[rs2];
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_sw<B: crate::bus::Bus>(&mut self, rs1: u8, rs2: u8, imm: u16, bus: &mut B) -> StepResult {
        let rs1 = rs1 as usize;
        let rs2 = rs2 as usize;
        let imm = imm as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        bus.write32(addr, self.regs[rs2]);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_addi(&mut self, rd: u8, imm: i16) -> StepResult {
        let rd = rd as usize;
        let imm = (imm as i32) as u32;
        self.regs[rd] = self.regs[rd].wrapping_add(imm);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_jal(&mut self, _rd: u8, imm: i16) -> StepResult {
        let imm = (imm as i32) as u32;
        self.regs[1] = self.pc.wrapping_add(2);
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    #[inline(always)]
    pub(crate) fn c_li(&mut self, rd: u8, imm: i16) -> StepResult {
        let rd = rd as usize;
        let imm = (imm as i32) as u32;
        self.regs[rd] = imm;
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_addi16sp(&mut self, _rd: u8, imm: i16) -> StepResult {
        let imm = (imm as i32) as u32;
        if imm == 0 {
            return StepResult::Trap(2); // Reserved
        }
        self.regs[2] = self.regs[2].wrapping_add(imm);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_lui(&mut self, rd: u8, imm: i32) -> StepResult {
        let rd = rd as usize;
        let imm_u32 = (imm << 12) as u32;
        self.regs[rd] = imm_u32;
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_srli(&mut self, rd: u8, shamt: u8) -> StepResult {
        let rd = rd as usize;
        self.regs[rd] = self.regs[rd].wrapping_shr(shamt as u32);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_srai(&mut self, rd: u8, shamt: u8) -> StepResult {
        let rd = rd as usize;
        self.regs[rd] = ((self.regs[rd] as i32).wrapping_shr(shamt as u32)) as u32;
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_andi(&mut self, rd: u8, imm: i16) -> StepResult {
        let rd = rd as usize;
        let imm = (imm as i32) as u32;
        self.regs[rd] &= imm;
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_sub(&mut self, rd: u8, rs2: u8) -> StepResult {
        let rd = rd as usize;
        let rs2 = rs2 as usize;
        self.regs[rd] = self.regs[rd].wrapping_sub(self.regs[rs2]);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_xor(&mut self, rd: u8, rs2: u8) -> StepResult {
        let rd = rd as usize;
        let rs2 = rs2 as usize;
        self.regs[rd] ^= self.regs[rs2];
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_or(&mut self, rd: u8, rs2: u8) -> StepResult {
        let rd = rd as usize;
        let rs2 = rs2 as usize;
        self.regs[rd] |= self.regs[rs2];
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_and(&mut self, rd: u8, rs2: u8) -> StepResult {
        let rd = rd as usize;
        let rs2 = rs2 as usize;
        self.regs[rd] &= self.regs[rs2];
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_j(&mut self, imm: i16) -> StepResult {
        let imm = (imm as i32) as u32;
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    #[inline(always)]
    pub(crate) fn c_beqz(&mut self, rs1: u8, imm: i16) -> StepResult {
        let rs1 = rs1 as usize;
        let imm = (imm as i32) as u32;
        if self.regs[rs1] == 0 {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(2)
        }
    }

    #[inline(always)]
    pub(crate) fn c_bnez(&mut self, rs1: u8, imm: i16) -> StepResult {
        let rs1 = rs1 as usize;
        let imm = (imm as i32) as u32;
        if self.regs[rs1] != 0 {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(2)
        }
    }

    #[inline(always)]
    pub(crate) fn c_slli(&mut self, rd: u8, shamt: u8) -> StepResult {
        let rd = rd as usize;
        self.regs[rd] = self.regs[rd].wrapping_shl(shamt as u32);
        StepResult::Ok(2)
    }

    #[inline(always)]
    pub(crate) fn c_jalr(&mut self, rs1: u8) -> StepResult {
        let rs1 = rs1 as usize;
        if rs1 == 0 {
            return StepResult::Trap(2); // rs1 != 0
        }
        let next_pc = self.pc + 2;
        self.pc = self.regs[rs1] & !1;
        self.regs[1] = next_pc;
        StepResult::Jumped
    }

    #[inline(always)]
    pub(crate) fn c_add(&mut self, rd: u8, rs2: u8) -> StepResult {
        let rd = rd as usize;
        let rs2 = rs2 as usize;
        if rd == 0 {
            // rd != 0 (C.ADD is only for rd != 0, rs2 != 0)
            return StepResult::Trap(2);
        }
        self.regs[rd] = self.regs[rd].wrapping_add(self.regs[rs2]);
        StepResult::Ok(2)
    }
}
