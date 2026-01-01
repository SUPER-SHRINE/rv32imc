use crate::cpu::{Cpu, StepResult};

impl Cpu {
    fn check_csr_privilege(&self, csr_addr: u32, is_write: bool) -> bool {
        let min_priv = (csr_addr >> 8) & 0b11;
        let mode = self.mode as u32;

        if mode < min_priv {
            return false;
        }

        // 読み取り専用 CSR への書き込みチェック
        if is_write && ((csr_addr >> 10) & 0b11 == 0b11) {
            return false;
        }

        // カウンタ CSR (0xc00-0xc1f, 0xc80-0xc9f) のチェック
        if mode != 3 { // Machine 以外のモード
            let counter_idx = match csr_addr {
                0xc00..=0xc1f => Some(csr_addr - 0xc00),
                0xc80..=0xc9f => Some(csr_addr - 0xc80),
                _ => None,
            };
            if let Some(idx) = counter_idx {
                if (self.csr.mcounteren & (1 << idx)) == 0 {
                    return false;
                }
            }
        }

        true
    }

    pub(crate) fn csrrw(&mut self, rd: usize, rs1: usize, csr_addr: u32) -> StepResult {
        if !self.check_csr_privilege(csr_addr, true) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        let new_val = self.regs[rs1];

        if rd != 0 {
            self.regs[rd] = old_val;
        }
        if let Err(_) = self.csr.write(csr_addr, new_val) {
            return StepResult::Trap(2);
        }
        StepResult::Ok
    }

    pub(crate) fn csrrs(&mut self, rd: usize, rs1: usize, csr_addr: u32) -> StepResult {
        let is_write = rs1 != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        let set_mask = self.regs[rs1];

        if rd != 0 {
            self.regs[rd] = old_val;
        }
        if rs1 != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val | set_mask) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok
    }

    pub(crate) fn csrrc(&mut self, rd: usize, rs1: usize, csr_addr: u32) -> StepResult {
        let is_write = rs1 != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        let clear_mask = self.regs[rs1];

        if rd != 0 {
            self.regs[rd] = old_val;
        }
        if rs1 != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val & !clear_mask) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok
    }

    pub(crate) fn csrrwi(&mut self, rd: usize, uimm: usize, csr_addr: u32) -> StepResult {
        if !self.check_csr_privilege(csr_addr, true) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };

        if rd != 0 {
            self.regs[rd] = old_val;
        }
        if let Err(_) = self.csr.write(csr_addr, uimm as u32) {
            return StepResult::Trap(2);
        }
        StepResult::Ok
    }

    pub(crate) fn csrrsi(&mut self, rd: usize, uimm: usize, csr_addr: u32) -> StepResult {
        let is_write = uimm != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        if rd != 0 {
            self.regs[rd] = old_val;
        }
        if uimm != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val | uimm as u32) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok
    }

    pub(crate) fn csrrci(&mut self, rd: usize, uimm: usize, csr_addr: u32) -> StepResult {
        let is_write = uimm != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        if rd != 0 {
            self.regs[rd] = old_val;
        }
        if uimm != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val & !(uimm as u32)) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok
    }
}
