use crate::cpu::Cpu;
use crate::cpu::privilege_mode::PrivilegeMode;
use crate::cpu::StepResult;

impl Cpu {
    pub(crate) fn lui(&mut self, inst_bin: u32) -> StepResult {
        let (rd, imm) = self.decode_u_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = imm;
        }
        StepResult::Ok
    }

    pub(crate) fn auipc(&mut self, inst_bin: u32) -> StepResult {
        let (rd, imm) = self.decode_u_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(imm);
        }
        StepResult::Ok
    }

    pub(crate) fn jal(&mut self, inst_bin: u32) -> StepResult {
        let (rd, imm) = self.decode_j_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(4);
        }
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    pub(crate) fn jalr(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let t = self.pc.wrapping_add(4);
        let target = self.regs[rs1].wrapping_add(imm) & !1;
        if rd != 0 {
            self.regs[rd] = t;
        }
        self.pc = target;
        StepResult::Jumped
    }

    pub(crate) fn beq(&mut self, inst_bin: u32) -> StepResult {
        let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] == self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bne(&mut self, inst_bin: u32) -> StepResult {
        let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] != self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn blt(&mut self, inst_bin: u32) -> StepResult {
        let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
        if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bge(&mut self, inst_bin: u32) -> StepResult {
        let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
        if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bltu(&mut self, inst_bin: u32) -> StepResult {
        let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] < self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bgeu(&mut self, inst_bin: u32) -> StepResult {
        let (rs1, rs2, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] >= self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn lb<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as i8 as i32 as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lh<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as i16 as i32 as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lw<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read32(addr);
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lbu<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lhu<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn sb<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rs1, rs2, imm) = self.decode_s_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xff) as u8;
        bus.write8(addr, val);
        StepResult::Ok
    }

    pub(crate) fn sh<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rs1, rs2, imm) = self.decode_s_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xffff) as u16;
        bus.write16(addr, val);
        StepResult::Ok
    }

    pub(crate) fn sw<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let (rs1, rs2, imm) = self.decode_s_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = self.regs[rs2];
        bus.write32(addr, val);
        StepResult::Ok
    }

    pub(crate) fn addi(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_add(imm);
        }
        StepResult::Ok
    }

    pub(crate) fn slti(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = if (self.regs[rs1] as i32) < (imm as i32) {
                1
            } else {
                0
            };
        }
        StepResult::Ok
    }

    pub(crate) fn sltiu(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
        }
        StepResult::Ok
    }

    pub(crate) fn xori(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] ^ imm;
        }
        StepResult::Ok
    }

    pub(crate) fn ori(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] | imm;
        }
        StepResult::Ok
    }

    pub(crate) fn andi(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] & imm;
        }
        StepResult::Ok
    }

    pub(crate) fn slli(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let shamt = imm & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] << shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn srli(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let shamt = imm & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] >> shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn srai(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, imm) = self.decode_i_type(inst_bin);
        let shamt = imm & 0x1f;
        if rd != 0 {
            self.regs[rd] = (self.regs[rs1] as i32 >> shamt) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn add(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
        }
        StepResult::Ok
    }

    pub(crate) fn sub(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
        }
        StepResult::Ok
    }

    pub(crate) fn sll(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        let shamt = self.regs[rs2] & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] << shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn slt(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
                1
            } else {
                0
            };
        }
        StepResult::Ok
    }

    pub(crate) fn sltu(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
        }
        StepResult::Ok
    }

    pub(crate) fn xor(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
        }
        StepResult::Ok
    }

    pub(crate) fn srl(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        let shamt = self.regs[rs2] & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] >> shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn sra(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        let shamt = self.regs[rs2] & 0x1f;
        if rd != 0 {
            self.regs[rd] = (self.regs[rs1] as i32 >> shamt) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn or(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] | self.regs[rs2];
        }
        StepResult::Ok
    }

    pub(crate) fn and(&mut self, inst_bin: u32) -> StepResult {
        let (rd, rs1, rs2) = self.decode_r_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] & self.regs[rs2];
        }
        StepResult::Ok
    }

    pub(crate) fn fence(&mut self) -> StepResult {
        StepResult::Ok
    }

    pub(crate) fn fence_i(&mut self) -> StepResult {
        StepResult::Ok
    }

    pub(crate) fn ecall(&mut self) -> StepResult {
        let code = match self.mode {
            PrivilegeMode::User => 8,
            PrivilegeMode::Supervisor => 9,
            PrivilegeMode::Machine => 11,
        };
        self.handle_trap(code)
    }

    pub(crate) fn ebreak(&mut self) -> StepResult {
        self.handle_trap(3) // Breakpoint exception code is 3
    }

    pub(crate) fn mret(&mut self) -> StepResult {
        // PC を mepc に復帰
        self.pc = self.csr.mepc;

        // mstatus の復帰
        let mpie = (self.csr.mstatus >> 7) & 1;
        self.csr.mstatus &= !(1 << 3);  // MIE = 0
        self.csr.mstatus |= mpie << 3;  // MIE = MPIE
        self.csr.mstatus |= 1 << 7;     // MPIE = 1 (spec says MPIE is set to 1)

        let mpp = (self.csr.mstatus >> 11) & 0b11;
        self.mode = match mpp {
            0 => PrivilegeMode::User,
            1 => PrivilegeMode::Supervisor,
            3 => PrivilegeMode::Machine,
            _ => PrivilegeMode::Machine, // Should not happen
        };
        // MPP is set to the least-privileged mode supported (User=0)
        self.csr.mstatus &= !(0b11 << 11);
        StepResult::Jumped
    }

    pub(crate) fn csrrw(&mut self, inst_bin: u32) -> StepResult {
        let csr_addr = (inst_bin >> 20) & 0xfff;
        let rs1 = (inst_bin >> 15) & 0x1f;
        let rd = (inst_bin >> 7) & 0x1f;

        let old_val = self.csr.read(csr_addr);
        let new_val = self.regs[rs1 as usize];

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        self.csr.write(csr_addr, new_val);
        StepResult::Ok
    }

    pub(crate) fn csrrs(&mut self, inst_bin: u32) -> StepResult {
        let csr_addr = (inst_bin >> 20) & 0xfff;
        let rs1 = (inst_bin >> 15) & 0x1f;
        let rd = (inst_bin >> 7) & 0x1f;

        let old_val = self.csr.read(csr_addr);
        let set_mask = self.regs[rs1 as usize];

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if rs1 != 0 {
            self.csr.write(csr_addr, old_val | set_mask);
        }
        StepResult::Ok
    }

    pub(crate) fn csrrc(&mut self, inst_bin: u32) -> StepResult {
        let csr_addr = (inst_bin >> 20) & 0xfff;
        let rs1 = (inst_bin >> 15) & 0x1f;
        let rd = (inst_bin >> 7) & 0x1f;

        let old_val = self.csr.read(csr_addr);
        let clear_mask = self.regs[rs1 as usize];

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if rs1 != 0 {
            self.csr.write(csr_addr, old_val & !clear_mask);
        }
        StepResult::Ok
    }

    pub(crate) fn csrrwi(&mut self, inst_bin: u32) -> StepResult {
        let csr_addr = (inst_bin >> 20) & 0xfff;
        let uimm = (inst_bin >> 15) & 0x1f;
        let rd = (inst_bin >> 7) & 0x1f;

        let old_val = self.csr.read(csr_addr);

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        self.csr.write(csr_addr, uimm);
        StepResult::Ok
    }

    pub(crate) fn csrrsi(&mut self, inst_bin: u32) -> StepResult {
        let csr_addr = (inst_bin >> 20) & 0xfff;
        let uimm = (inst_bin >> 15) & 0x1f;
        let rd = (inst_bin >> 7) & 0x1f;

        let old_val = self.csr.read(csr_addr);
        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if uimm != 0 {
            self.csr.write(csr_addr, old_val | uimm);
        }
        StepResult::Ok
    }

    pub(crate) fn csrrci(&mut self, inst_bin: u32) -> StepResult {
        let csr_addr = (inst_bin >> 20) & 0xfff;
        let uimm = (inst_bin >> 15) & 0x1f;
        let rd = (inst_bin >> 7) & 0x1f;

        let old_val = self.csr.read(csr_addr);
        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if uimm != 0 {
            self.csr.write(csr_addr, old_val & !uimm);
        }
        StepResult::Ok
    }
}
