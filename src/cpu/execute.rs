use super::Cpu;

impl Cpu {
    pub(super) fn lui(&mut self, inst_bin: u32) {
        let (rd, imm) = self.decode_u_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = imm;
        }
    }

    pub(super) fn auipc(&mut self, inst_bin: u32) {
        let (rd, imm) = self.decode_u_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(imm);
        }
    }

    pub(super) fn jal(&mut self, inst_bin: u32) {
        let (rd, imm) = self.decode_j_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(4);
        }
        self.pc = self.pc.wrapping_add(imm);
    }

    pub(super) fn jalr(&mut self, inst_bin: u32) {
        let (rd, rs1, _funct3, imm) = self.decode_i_type(inst_bin);
        let t = self.pc.wrapping_add(4);
        let target = self.regs[rs1].wrapping_add(imm) & !1;
        if rd != 0 {
            self.regs[rd] = t;
        }
        self.pc = target;
    }

    pub(super) fn beq(&mut self, inst_bin: u32) {
        let (rs1, rs2, _funct3, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] == self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    pub(super) fn bne(&mut self, inst_bin: u32) {
        let (rs1, rs2, _funct3, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] != self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    pub(super) fn blt(&mut self, inst_bin: u32) {
        let (rs1, rs2, _funct3, imm) = self.decode_b_type(inst_bin);
        if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    pub(super) fn bge(&mut self, inst_bin: u32) {
        let (rs1, rs2, _funct3, imm) = self.decode_b_type(inst_bin);
        if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    pub(super) fn bltu(&mut self, inst_bin: u32) {
        let (rs1, rs2, _funct3, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] < self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    pub(super) fn bgeu(&mut self, inst_bin: u32) {
        let (rs1, rs2, _funct3, imm) = self.decode_b_type(inst_bin);
        if self.regs[rs1] >= self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }
    }

    pub(super) fn lb<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rd, rs1, _funct3, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as i8 as i32 as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
    }

    pub(super) fn lh<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rd, rs1, _funct3, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as i16 as i32 as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
    }

    pub(super) fn lw<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rd, rs1, _funct3, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read32(addr);
        if rd != 0 {
            self.regs[rd] = val;
        }
    }

    pub(super) fn lbu<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rd, rs1, _funct3, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
    }

    pub(super) fn lhu<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rd, rs1, _funct3, imm) = self.decode_i_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
    }

    pub(super) fn sb<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rs1, rs2, _funct3, imm) = self.decode_s_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xff) as u8;
        bus.write8(addr, val);
    }

    pub(super) fn sh<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rs1, rs2, _funct3, imm) = self.decode_s_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xffff) as u16;
        bus.write16(addr, val);
    }

    pub(super) fn sw<B: crate::bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let (rs1, rs2, _funct3, imm) = self.decode_s_type(inst_bin);
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = self.regs[rs2];
        bus.write32(addr, val);
    }
}
