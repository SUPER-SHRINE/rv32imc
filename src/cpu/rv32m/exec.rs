use crate::cpu::Cpu;
use crate::cpu::StepResult;

impl Cpu {
    pub(crate) fn mul(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]);
        }
        StepResult::Ok
    }

    pub(crate) fn mulh(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            let src1 = self.regs[rs1] as i32 as i64;
            let src2 = self.regs[rs2] as i32 as i64;
            let result = src1 * src2;
            self.regs[rd] = (result >> 32) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn mulhsu(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            let src1 = self.regs[rs1] as i32 as i128;
            let src2 = self.regs[rs2] as u64 as i128;
            let result = src1 * src2;
            self.regs[rd] = (result >> 32) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn mulhu(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            let src1 = self.regs[rs1] as u64;
            let src2 = self.regs[rs2] as u64;
            let result = src1 * src2;
            self.regs[rd] = (result >> 32) as u32;
        }
        StepResult::Ok
    }

    pub(crate) fn div(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
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
        }
        StepResult::Ok
    }

    pub(crate) fn divu(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            let val1 = self.regs[rs1];
            let val2 = self.regs[rs2];

            let result = if val2 == 0 {
                u32::MAX
            } else {
                val1 / val2
            };

            self.regs[rd] = result;
        }
        StepResult::Ok
    }

    pub(crate) fn rem(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
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
        }
        StepResult::Ok
    }

    pub(crate) fn remu(&mut self, rd: usize, rs1: usize, rs2: usize) -> StepResult {
        if rd != 0 {
            let val1 = self.regs[rs1];
            let val2 = self.regs[rs2];

            let result = if val2 == 0 {
                val1
            } else {
                val1 % val2
            };

            self.regs[rd] = result;
        }
        StepResult::Ok
    }
}
