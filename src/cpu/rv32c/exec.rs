use crate::cpu::{Cpu, StepResult};

impl Cpu {
    pub(crate) fn c_addi4spn(&mut self, inst_bin: u16) -> StepResult {
        let (rd, imm) = self.decode_ciw_type(inst_bin);
        if imm == 0 {
            return self.handle_trap(2); // Reserved
        }
        self.regs[rd] = self.regs[2].wrapping_add(imm);
        StepResult::Ok
    }

    pub(crate) fn c_lw<B: crate::bus::Bus>(&mut self, inst_bin: u16, bus: &mut B) -> StepResult {
        let (rd, rs1, imm) = self.decode_cl_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read32(addr);
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn c_lwsp<B: crate::bus::Bus>(&mut self, inst_bin: u16, bus: &mut B) -> StepResult {
        let (rd, imm) = self.decode_c_lwsp_type(inst_bin);
        if rd == 0 {
            return self.handle_trap(2); // Reserved
        }
        let addr = self.regs[2].wrapping_add(imm);
        self.regs[rd] = bus.read32(addr);
        StepResult::Ok
    }

    pub(crate) fn c_swsp<B: crate::bus::Bus>(&mut self, inst_bin: u16, bus: &mut B) -> StepResult {
        let (rs2, imm) = self.decode_c_swsp_type(inst_bin);
        let addr = self.regs[2].wrapping_add(imm);
        bus.write32(addr, self.regs[rs2]);
        StepResult::Ok
    }

    pub(crate) fn c_jr(&mut self, inst_bin: u16) -> StepResult {
        let (rs1, rs2) = self.decode_cr_type(inst_bin);
        if rs1 == 0 || rs2 != 0 {
            return self.handle_trap(2); // C.JR: rs1 != 0, rs2 == 0
        }
        self.pc = self.regs[rs1] & !1;
        StepResult::Jumped
    }

    pub(crate) fn c_mv(&mut self, inst_bin: u16) -> StepResult {
        let (rd, rs2) = self.decode_cr_type(inst_bin);
        if rd == 0 || rs2 == 0 {
            return self.handle_trap(2); // C.MV: rd != 0, rs2 != 0
        }
        self.regs[rd] = self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_sw<B: crate::bus::Bus>(&mut self, inst_bin: u16, bus: &mut B) -> StepResult {
        let (rs1, rs2, imm) = self.decode_cs_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        bus.write32(addr, self.regs[rs2]);
        StepResult::Ok
    }

    pub(crate) fn c_addi(&mut self, inst_bin: u16) -> StepResult {
        let (rd, imm) = self.decode_ci_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rd].wrapping_add(imm);
        }
        StepResult::Ok
    }

    pub(crate) fn c_jal(&mut self, inst_bin: u16) -> StepResult {
        let imm = self.decode_cj_type(inst_bin);
        self.regs[1] = self.pc.wrapping_add(2);
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    pub(crate) fn c_li(&mut self, inst_bin: u16) -> StepResult {
        let (rd, imm) = self.decode_ci_type(inst_bin);
        if rd == 0 {
            return self.handle_trap(2); // Reserved for HINTs
        }
        self.regs[rd] = imm;
        StepResult::Ok
    }

    pub(crate) fn c_addi16sp(&mut self, inst_bin: u16) -> StepResult {
        let imm = self.decode_c_addi16sp_imm(inst_bin);
        if imm == 0 {
            return self.handle_trap(2); // Reserved
        }
        self.regs[2] = self.regs[2].wrapping_add(imm);
        StepResult::Ok
    }

    pub(crate) fn c_lui(&mut self, inst_bin: u16) -> StepResult {
        let (rd, imm) = self.decode_ci_type(inst_bin);
        if rd == 0 || rd == 2 {
            return self.handle_trap(2); // rd=0 is reserved, rd=2 is C.ADDI16SP
        }
        // C.LUI loads the 6-bit immediate into bits [17:12], then sign-extends.
        // But wait, the spec says "loads the non-zero 6-bit immediate into bits 17-12, then sign-extends".
        // Our decode_ci_type returns a sign-extended 32-bit value from bits [12|6:2] of inst.
        // For C.LUI, the 6 bits are imm[17:12].

        let imm_u32 = imm << 12;
        self.regs[rd] = imm_u32;
        StepResult::Ok
    }

    pub(crate) fn c_srli(&mut self, inst_bin: u16) -> StepResult {
        let (rd, shamt) = self.decode_cb_shamt_type(inst_bin);
        // shamt[5] must be 0 for RV32C
        if (inst_bin >> 12) & 0x1 != 0 {
            return self.handle_trap(2);
        }
        if shamt == 0 {
            // shamt=0 is reserved for HINTs
            return StepResult::Ok;
        }
        self.regs[rd] >>= shamt;
        StepResult::Ok
    }

    pub(crate) fn c_srai(&mut self, inst_bin: u16) -> StepResult {
        let (rd, shamt) = self.decode_cb_shamt_type(inst_bin);
        // shamt[5] must be 0 for RV32C
        if (inst_bin >> 12) & 0x1 != 0 {
            return self.handle_trap(2);
        }
        if shamt == 0 {
            // shamt=0 is reserved for HINTs
            return StepResult::Ok;
        }
        self.regs[rd] = ((self.regs[rd] as i32) >> shamt) as u32;
        StepResult::Ok
    }

    pub(crate) fn c_andi(&mut self, inst_bin: u16) -> StepResult {
        let (rd, imm) = self.decode_cb_andi_type(inst_bin);
        self.regs[rd] &= imm;
        StepResult::Ok
    }

    pub(crate) fn c_sub(&mut self, inst_bin: u16) -> StepResult {
        let (rd, rs2) = self.decode_ca_type(inst_bin);
        self.regs[rd] = self.regs[rd].wrapping_sub(self.regs[rs2]);
        StepResult::Ok
    }

    pub(crate) fn c_xor(&mut self, inst_bin: u16) -> StepResult {
        let (rd, rs2) = self.decode_ca_type(inst_bin);
        self.regs[rd] ^= self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_or(&mut self, inst_bin: u16) -> StepResult {
        let (rd, rs2) = self.decode_ca_type(inst_bin);
        self.regs[rd] |= self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_and(&mut self, inst_bin: u16) -> StepResult {
        let (rd, rs2) = self.decode_ca_type(inst_bin);
        self.regs[rd] &= self.regs[rs2];
        StepResult::Ok
    }

    pub(crate) fn c_j(&mut self, inst_bin: u16) -> StepResult {
        let imm = self.decode_cj_type(inst_bin);
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    pub(crate) fn c_beqz(&mut self, inst_bin: u16) -> StepResult {
        let (rs1, imm) = self.decode_cb_branch_type(inst_bin);
        if self.regs[rs1] == 0 {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn c_bnez(&mut self, inst_bin: u16) -> StepResult {
        let (rs1, imm) = self.decode_cb_branch_type(inst_bin);
        if self.regs[rs1] != 0 {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn c_slli(&mut self, inst_bin: u16) -> StepResult {
        let (rd, shamt) = self.decode_ci_shamt_type(inst_bin);
        if rd == 0 {
            return self.handle_trap(2); // Reserved
        }
        // shamt[5] must be 0 for RV32C
        if (inst_bin >> 12) & 0x1 != 0 {
            return self.handle_trap(2);
        }
        if shamt == 0 {
            // shamt=0 is reserved for HINTs
            return StepResult::Ok;
        }
        self.regs[rd] <<= shamt;
        StepResult::Ok
    }

    pub(crate) fn c_jalr(&mut self, inst_bin: u16) -> StepResult {
        let rs1 = ((inst_bin >> 7) & 0x1f) as usize;
        let next_pc = self.pc + 2;
        self.pc = self.regs[rs1] & !1;
        self.regs[1] = next_pc;
        StepResult::Jumped
    }

    pub(crate) fn c_add(&mut self, inst_bin: u16) -> StepResult {
        let rd = ((inst_bin >> 7) & 0x1f) as usize;
        let rs2 = ((inst_bin >> 2) & 0x1f) as usize;
        self.regs[rd] = self.regs[rd].wrapping_add(self.regs[rs2]);
        StepResult::Ok
    }
}
