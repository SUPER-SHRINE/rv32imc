use crate::cpu::Cpu;
use crate::cpu::privilege_mode::PrivilegeMode;
use crate::cpu::StepResult;

impl Cpu {
    pub(crate) fn lui(&mut self, rd: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = imm;
        }
        StepResult::Ok
    }

    pub(crate) fn auipc(&mut self, rd: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(imm);
        }
        StepResult::Ok
    }

    pub(crate) fn jal(&mut self, rd: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(4);
        }
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    pub(crate) fn jalr(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        let t = self.pc.wrapping_add(4);
        let target = self.regs[rs1].wrapping_add(imm) & !1;
        if rd != 0 {
            self.regs[rd] = t;
        }
        self.pc = target;
        StepResult::Jumped
    }

    pub(crate) fn beq(&mut self, rs1: usize, rs2: usize, imm: u32) -> StepResult {
        if self.regs[rs1] == self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bne(&mut self, rs1: usize, rs2: usize, imm: u32) -> StepResult {
        if self.regs[rs1] != self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn blt(&mut self, rs1: usize, rs2: usize, imm: u32) -> StepResult {
        if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bge(&mut self, rs1: usize, rs2: usize, imm: u32) -> StepResult {
        if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bltu(&mut self, rs1: usize, rs2: usize, imm: u32) -> StepResult {
        if self.regs[rs1] < self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn bgeu(&mut self, rs1: usize, rs2: usize, imm: u32) -> StepResult {
        if self.regs[rs1] >= self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok
        }
    }

    pub(crate) fn lb(&mut self, rd: usize, rs1: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as i8 as i32 as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lh(&mut self, rd: usize, rs1: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as i16 as i32 as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lw(&mut self, rd: usize, rs1: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read32(addr);
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lbu(&mut self, rd: usize, rs1: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn lhu(&mut self, rd: usize, rs1: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as u32;
        if rd != 0 {
            self.regs[rd] = val;
        }
        StepResult::Ok
    }

    pub(crate) fn sb(&mut self, rs1: usize, rs2: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xff) as u8;
        bus.write8(addr, val);
        StepResult::Ok
    }

    pub(crate) fn sh(&mut self, rs1: usize, rs2: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xffff) as u16;
        bus.write16(addr, val);
        StepResult::Ok
    }

    pub(crate) fn sw(&mut self, rs1: usize, rs2: usize, imm: u32, bus: &mut dyn crate::bus::Bus) -> StepResult {
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = self.regs[rs2];
        bus.write32(addr, val);
        StepResult::Ok
    }

    pub(crate) fn addi(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_add(imm);
        }
        StepResult::Ok
    }

    pub(crate) fn slti(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = if (self.regs[rs1] as i32) < (imm as i32) {
                1
            } else {
                0
            };
        }
        StepResult::Ok
    }

    pub(crate) fn sltiu(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
        }
        StepResult::Ok
    }

    pub(crate) fn xori(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] ^ imm;
        }
        StepResult::Ok
    }

    pub(crate) fn ori(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] | imm;
        }
        StepResult::Ok
    }

    pub(crate) fn andi(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] & imm;
        }
        StepResult::Ok
    }

    pub(crate) fn slli(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if (imm & !0x1f) != 0 {
            return StepResult::Trap(2);
        }
        let shamt = imm & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] << shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn srli(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        if (imm & !0x1f) != 0 {
            return StepResult::Trap(2);
        }
        let shamt = imm & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] >> shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn srai(&mut self, rd: usize, rs1: usize, imm: u32) -> StepResult {
        // SRAI has bit 30 set (0x400 in imm[11:0]), but other bits in imm[11:5] should be 0
        if (imm & !0x1f) != 0x400 {
            return StepResult::Trap(2);
        }
        let shamt = imm & 0x1f;
        if rd != 0 {
            self.regs[rd] = (self.regs[rs1] as i32 >> shamt) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn add(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
        }
        StepResult::Ok
    }

    pub(crate) fn sub(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
        }
        StepResult::Ok
    }

    pub(crate) fn sll(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        let shamt = self.regs[rs2] & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] << shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn slt(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
                1
            } else {
                0
            };
        }
        StepResult::Ok
    }

    pub(crate) fn sltu(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
        }
        StepResult::Ok
    }

    pub(crate) fn xor(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
        }
        StepResult::Ok
    }

    pub(crate) fn srl(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        let shamt = self.regs[rs2] & 0x1f;
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] >> shamt;
        }
        StepResult::Ok
    }

    pub(crate) fn sra(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        let shamt = self.regs[rs2] & 0x1f;
        if rd != 0 {
            self.regs[rd] = (self.regs[rs1] as i32 >> shamt) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn or(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] | self.regs[rs2];
        }
        StepResult::Ok
    }

    pub(crate) fn and(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1] & self.regs[rs2];
        }
        StepResult::Ok
    }

    pub(crate) fn fence(&mut self) -> StepResult {
        StepResult::Ok
    }

    pub(crate) fn fence_i(&mut self) -> StepResult {
        self.cache.clear();
        StepResult::Ok
    }

    pub(crate) fn ecall(&mut self) -> StepResult {
        let code = match self.mode {
            PrivilegeMode::User => 8,
            PrivilegeMode::Supervisor => 9,
            PrivilegeMode::Machine => 11,
        };
        StepResult::Trap(code)
    }

    pub(crate) fn ebreak(&mut self) -> StepResult {
        StepResult::Trap(3) // Breakpoint exception code is 3
    }

    pub(crate) fn mret(&mut self) -> StepResult {
        // mret は Machine モードでのみ実行可能
        // ただし、riscv-tests の中には特権レベルが Machine でないときに mret を実行して
        // 例外が発生することを確認するものがある。
        if self.mode != PrivilegeMode::Machine {
            return StepResult::Trap(2); // Illegal Instruction
        }

        // PC を mepc に復帰
        let next_pc = self.csr.mepc;

        // mstatus の復帰
        let mpie = (self.csr.mstatus >> 7) & 1;
        self.csr.mstatus &= !(1 << 3);  // MIE = 0
        self.csr.mstatus |= mpie << 3;  // MIE = MPIE
        self.csr.mstatus |= 1 << 7;     // MPIE = 1

        let mpp = (self.csr.mstatus >> 11) & 0b11;
        self.mode = match mpp {
            0 => PrivilegeMode::User,
            3 => PrivilegeMode::Machine,
            _ => PrivilegeMode::User, // Sモードはサポートしていないので User に落とす
        };
        // MPP is set to the least-privileged mode supported (User=0)
        self.csr.mstatus &= !(0b11 << 11);

        self.pc = next_pc;
        StepResult::Jumped
    }
}
