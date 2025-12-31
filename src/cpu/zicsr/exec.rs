use crate::cpu::{Cpu, StepResult};

impl Cpu {
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
