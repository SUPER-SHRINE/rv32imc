use crate::cpu::{Cpu, StepResult};
use crate::cpu::privilege_mode::PrivilegeMode;

impl Cpu {
    pub(super) fn handle_trap(&mut self, exception_code: u32, mtval: u32) -> StepResult {
        // 1. mepc に現在の PC を保存
        self.csr.mepc = self.pc;

        // 2. mcause に例外コードを設定
        self.csr.mcause = exception_code;

        // 3. mtval の設定
        self.csr.mtval = mtval;

        // 3. mstatus の更新 (MPP, MPIE, MIE)
        // mstatus bit fields:
        // MIE:  bit 3
        // MPIE: bit 7
        // MPP:  bits 11-12
        let mie = (self.csr.mstatus >> 3) & 1;
        self.csr.mstatus &= !(1 << 7); // MPIE = 0
        self.csr.mstatus |= mie << 7;  // MPIE = MIE
        self.csr.mstatus &= !(1 << 3); // MIE = 0

        let mpp = self.mode as u32;
        self.csr.mstatus &= !(0b11 << 11); // MPP = 0
        self.csr.mstatus |= mpp << 11;     // MPP = mode

        // 4. 特権モードを Machine に遷移
        self.mode = PrivilegeMode::Machine;

        // 5. mtvec のアドレスへジャンプ
        let is_interrupt = (exception_code >> 31) & 1;
        let mtvec_mode = self.csr.mtvec & 0b11;
        let mtvec_base = self.csr.mtvec & !0b11;

        if is_interrupt == 1 && mtvec_mode == 1 {
            // Vectored mode
            let code = exception_code & 0x7fff_ffff;
            self.pc = mtvec_base + 4 * code;
        } else {
            // Direct mode or exception
            self.pc = mtvec_base;
        }

        StepResult::Trap(exception_code)
    }
}
