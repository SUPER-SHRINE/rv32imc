use crate::cpu::Cpu;
use crate::cpu::privilege_mode::PrivilegeMode;
use crate::cpu::StepResult;

impl Cpu {
    #[inline(always)]
    pub(crate) fn lui(&mut self, rd: u8, imm: u32) -> StepResult {
        let rd: usize = rd as usize;
        self.regs[rd] = imm;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn auipc(&mut self, rd: u8, imm: u32) -> StepResult {
        let rd: usize = rd as usize;
        self.regs[rd] = self.pc.wrapping_add(imm);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn jal(&mut self, rd: u8, imm: u32) -> StepResult {
        let rd: usize = rd as usize;
        self.regs[rd] = self.pc.wrapping_add(4);
        self.pc = self.pc.wrapping_add(imm);
        StepResult::Jumped
    }

    #[inline(always)]
    pub(crate) fn jalr(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let t = self.pc.wrapping_add(4);
        let target = self.regs[rs1].wrapping_add(imm) & !1;
        self.regs[rd] = t;
        self.pc = target;
        StepResult::Jumped
    }

    #[inline(always)]
    pub(crate) fn beq(&mut self, rs1: u8, rs2: u8, imm: u16) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        if self.regs[rs1] == self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(4)
        }
    }

    #[inline(always)]
    pub(crate) fn bne(&mut self, rs1: u8, rs2: u8, imm: u16) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        if self.regs[rs1] != self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(4)
        }
    }

    #[inline(always)]
    pub(crate) fn blt(&mut self, rs1: u8, rs2: u8, imm: u16) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(4)
        }
    }

    #[inline(always)]
    pub(crate) fn bge(&mut self, rs1: u8, rs2: u8, imm: u16) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(4)
        }
    }

    #[inline(always)]
    pub(crate) fn bltu(&mut self, rs1: u8, rs2: u8, imm: u16) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        if self.regs[rs1] < self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(4)
        }
    }

    #[inline(always)]
    pub(crate) fn bgeu(&mut self, rs1: u8, rs2: u8, imm: u16) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        if self.regs[rs1] >= self.regs[rs2] {
            self.pc = self.pc.wrapping_add(imm);
            StepResult::Jumped
        } else {
            StepResult::Ok(4)
        }
    }

    #[inline(always)]
    pub(crate) fn lb<B: crate::bus::Bus>(&mut self, rd: u8, rs1: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as i8 as i32 as u32;
        self.regs[rd] = val;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn lh<B: crate::bus::Bus>(&mut self, rd: u8, rs1: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as i16 as i32 as u32;
        self.regs[rd] = val;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn lw<B: crate::bus::Bus>(&mut self, rd: u8, rs1: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read32(addr);
        self.regs[rd] = val;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn lbu<B: crate::bus::Bus>(&mut self, rd: u8, rs1: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read8(addr) as u32;
        self.regs[rd] = val;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn lhu<B: crate::bus::Bus>(&mut self, rd: u8, rs1: u8, imm: u16, bus: &mut B) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = bus.read16(addr) as u32;
        self.regs[rd] = val;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sb<B: crate::bus::Bus>(&mut self, rs1: u8, rs2: u8, imm: u16, bus: &mut B) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xff) as u8;
        bus.write8(addr, val);
        self.flush_cache_line(addr);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sh<B: crate::bus::Bus>(&mut self, rs1: u8, rs2: u8, imm: u16, bus: &mut B) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = (self.regs[rs2] & 0xffff) as u16;
        bus.write16(addr, val);
        self.flush_cache_line(addr);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sw<B: crate::bus::Bus>(&mut self, rs1: u8, rs2: u8, imm: u16, bus: &mut B) -> StepResult {
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let imm: u32   = (imm as i16 as i32) as u32;
        let addr = self.regs[rs1].wrapping_add(imm);
        let val = self.regs[rs2];
        bus.write32(addr, val);
        self.flush_cache_line(addr);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn addi(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32; // Sign extension
        self.regs[rd] = self.regs[rs1].wrapping_add(imm);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn slti(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: i32   = imm as i16 as i32; // Sign extension
        self.regs[rd] = if (self.regs[rs1] as i32) < imm {
            1
        } else {
            0
        };
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sltiu(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32; // Sign extension
        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn xori(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32; // Sign extension
        self.regs[rd] = self.regs[rs1] ^ imm;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn ori(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32; // Sign extension
        self.regs[rd] = self.regs[rs1] | imm;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn andi(&mut self, rd: u8, rs1: u8, imm: u16) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let imm: u32   = (imm as i16 as i32) as u32; // Sign extension
        self.regs[rd] = self.regs[rs1] & imm;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn slli(&mut self, rd: u8, rs1: u8, shamt: u8) -> StepResult {
        let rd:    usize = rd    as usize;
        let rs1:   usize = rs1   as usize;
        let shamt: u32   = shamt as u32;
        self.regs[rd] = self.regs[rs1] << shamt;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn srli(&mut self, rd: u8, rs1: u8, shamt: u8) -> StepResult {
        let rd:    usize = rd    as usize;
        let rs1:   usize = rs1   as usize;
        let shamt: u32   = shamt as u32;
        self.regs[rd] = self.regs[rs1] >> shamt;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn srai(&mut self, rd: u8, rs1: u8, shamt: u8) -> StepResult {
        let rd:    usize = rd    as usize;
        let rs1:   usize = rs1   as usize;
        let shamt: u32   = shamt as u32;
        self.regs[rd] = (self.regs[rs1] as i32 >> shamt) as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn add(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sub(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sll(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let shamt = self.regs[rs2] & 0x1f;
        self.regs[rd] = self.regs[rs1] << shamt;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn slt(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
            1
        } else {
            0
        };
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sltu(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn xor(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn srl(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let shamt = self.regs[rs2] & 0x1f;
        self.regs[rd] = self.regs[rs1] >> shamt;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn sra(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let shamt = self.regs[rs2] & 0x1f;
        self.regs[rd] = (self.regs[rs1] as i32 >> shamt) as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn or(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn and(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn fence(&mut self) -> StepResult {
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn fence_i(&mut self) -> StepResult {
        for page in self.pages.iter_mut() {
            *page = None;
        }
        self.current_page_num = 0xffffffff;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn ecall(&mut self) -> StepResult {
        let code = match self.mode {
            PrivilegeMode::User => 8,
            PrivilegeMode::Supervisor => 9,
            PrivilegeMode::Machine => 11,
        };
        StepResult::Trap(code)
    }

    #[inline(always)]
    pub(crate) fn ebreak(&mut self) -> StepResult {
        StepResult::Trap(3) // Breakpoint exception code is 3
    }

    #[inline(always)]
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
