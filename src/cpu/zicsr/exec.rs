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

    #[inline(always)]
    pub(crate) fn csrrw(&mut self, csr: u16, rd: u8, rs1: u8) -> StepResult {
        let rd:       usize = rd  as usize;
        let rs1:      usize = rs1 as usize;
        let csr_addr: u32   = csr as u32;

        if !self.check_csr_privilege(csr_addr, true) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        let new_val = self.regs[rs1 as usize];

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if let Err(_) = self.csr.write(csr_addr, new_val) {
            return StepResult::Trap(2);
        }
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn csrrs(&mut self, csr: u16, rd: u8, rs1: u8) -> StepResult {
        let rd:       usize = rd  as usize;
        let rs1:      usize = rs1 as usize;
        let csr_addr: u32   = csr as u32;

        let is_write = rs1 != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        let set_mask = self.regs[rs1 as usize];

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if rs1 != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val | set_mask) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn csrrc(&mut self, csr: u16, rd: u8, rs1: u8) -> StepResult {
        let rd:       usize = rd  as usize;
        let rs1:      usize = rs1 as usize;
        let csr_addr: u32   = csr as u32;

        let is_write = rs1 != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        let clear_mask = self.regs[rs1 as usize];

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if rs1 != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val & !clear_mask) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn csrrwi(&mut self, csr: u16, rd: u8, uimm: u8) -> StepResult {
        let rd:       usize = rd   as usize;
        let uimm:     u32   = uimm as u32;
        let csr_addr: u32   = csr  as u32;

        if !self.check_csr_privilege(csr_addr, true) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };

        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if let Err(_) = self.csr.write(csr_addr, uimm) {
            return StepResult::Trap(2);
        }
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn csrrsi(&mut self, csr: u16, rd: u8, uimm: u8) -> StepResult {
        let rd:       usize = rd   as usize;
        let uimm:     u32   = uimm as u32;
        let csr_addr: u32   = csr  as u32;

        let is_write = uimm != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if uimm != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val | uimm) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok(4)
    }

    #[inline(always)]
    pub(crate) fn csrrci(&mut self, csr: u16, rd: u8, uimm: u8) -> StepResult {
        let rd:       usize = rd   as usize;
        let uimm:     u32   = uimm as u32;
        let csr_addr: u32   = csr  as u32;

        let is_write = uimm != 0;
        if !self.check_csr_privilege(csr_addr, is_write) {
            return StepResult::Trap(2);
        }

        let old_val = match self.csr.read(csr_addr) {
            Ok(v) => v,
            Err(_) => return StepResult::Trap(2),
        };
        if rd != 0 {
            self.regs[rd as usize] = old_val;
        }
        if uimm != 0 {
            if let Err(_) = self.csr.write(csr_addr, old_val & !uimm) {
                return StepResult::Trap(2);
            }
        }
        StepResult::Ok(4)
    }
}
