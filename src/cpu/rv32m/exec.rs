use crate::cpu::Cpu;
use crate::cpu::StepResult;

impl Cpu {
    #[inline(always)]
    pub(crate) fn mul(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]);
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn mulh(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let src1 = self.regs[rs1] as i32 as i64;
        let src2 = self.regs[rs2] as i32 as i64;
        let result = src1 * src2;
        self.regs[rd] = (result >> 32) as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn mulhsu(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let src1 = self.regs[rs1] as i32 as i128;
        let src2 = self.regs[rs2] as u64 as i128;
        let result = src1 * src2;
        self.regs[rd] = (result >> 32) as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn mulhu(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let src1 = self.regs[rs1] as u64;
        let src2 = self.regs[rs2] as u64;
        let result = src1 * src2;
        self.regs[rd] = (result >> 32) as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn div(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let val1 = self.regs[rs1] as i32;
        let val2 = self.regs[rs2] as i32;

        let result = if val2 == 0 {
            -1
        } else if val1 == i32::MIN && val2 == -1 {
            i32::MIN
        } else {
            val1 / val2
        };

        self.regs[rd] = result as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn divu(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let val1 = self.regs[rs1];
        let val2 = self.regs[rs2];

        let result = if val2 == 0 {
            u32::MAX
        } else {
            val1 / val2
        };

        self.regs[rd] = result;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn rem(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let val1 = self.regs[rs1] as i32;
        let val2 = self.regs[rs2] as i32;

        let result = if val2 == 0 {
            val1
        } else if val1 == i32::MIN && val2 == -1 {
            0
        } else {
            val1 % val2
        };

        self.regs[rd] = result as u32;
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn remu(&mut self, rd: u8, rs1: u8, rs2: u8) -> StepResult {
        let rd:  usize = rd  as usize;
        let rs1: usize = rs1 as usize;
        let rs2: usize = rs2 as usize;
        let val1 = self.regs[rs1];
        let val2 = self.regs[rs2];

        let result = if val2 == 0 {
            val1
        } else {
            val1 % val2
        };

        self.regs[rd] = result;
        StepResult::Ok(4)
    }
}
