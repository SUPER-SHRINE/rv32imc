mod csr;
mod execute;
mod decode;
mod privilege_mode;

#[cfg(test)]
mod tests;

use super::bus;
use csr::Csr;
use privilege_mode::PrivilegeMode;

/// CPU の内部状態
pub struct Cpu {
    /// 32本の汎用レジスタ (x0-x31)
    pub regs: [u32; 32],

    /// プログラムカウンタ
    pub pc: u32,

    /// 制御ステータスレジスタ (CSR)
    pub csr: Csr,

    /// 特権モード
    pub mode: PrivilegeMode,
}

impl Cpu {
    pub fn new(pc: u32) -> Self {
        Self {
            regs: [0; 32],
            pc,
            csr: Csr::default(),
            mode: PrivilegeMode::Machine,
        }
    }

    /// 1ステップ実行
    pub fn step<B: bus::Bus>(&mut self, bus: &mut B) {
        let inst_bin = self.fetch(bus);
        self.execute(inst_bin, bus);
    }

    /// レジスタの状態をダンプ
    pub fn dump_registers(&self) {
        for (i, reg) in self.regs.iter().enumerate() {
            println!("x{:02}: 0x{:08x}", i, reg);
        }
        println!("pc : 0x{:08x}", self.pc);
    }

    fn handle_trap(&mut self, exception_code: u32) {
        // 1. mepc に現在の PC を保存
        self.csr.mepc = self.pc;

        // 2. mcause に例外コードを設定
        self.csr.mcause = exception_code;

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
        self.pc = self.csr.mtvec;
    }

    fn mret(&mut self) {
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
    }

    fn fetch<B: bus::Bus>(&mut self, bus: &mut B) -> u32 {
        bus.read32(self.pc)
    }

    fn execute<B: bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) {
        let opcode = inst_bin & 0x7f;
        match opcode {
            0b0110111 => {
                self.lui(inst_bin);
                self.pc += 4;
            }
            0b0010111 => {
                self.auipc(inst_bin);
                self.pc += 4;
            }
            0b1101111 => self.jal(inst_bin),
            0b1100111 => self.jalr(inst_bin),
            0b0010011 => {
                let funct3 = (inst_bin >> 12) & 0x7;
                match funct3 {
                    0b000 => self.addi(inst_bin),
                    0b001 => self.slli(inst_bin),
                    0b010 => self.slti(inst_bin),
                    0b011 => self.sltiu(inst_bin),
                    0b100 => self.xori(inst_bin),
                    0b101 => {
                        let imm11_5 = (inst_bin >> 25) & 0x7f;
                        match imm11_5 {
                            0b0000000 => self.srli(inst_bin),
                            0b0100000 => self.srai(inst_bin),
                            _ => {}
                        }
                    }
                    0b110 => self.ori(inst_bin),
                    0b111 => self.andi(inst_bin),
                    _ => {}
                }
                self.pc += 4;
            }
            0b1100011 => {
                let funct3 = (inst_bin >> 12) & 0x7;
                match funct3 {
                    0b000 => self.beq(inst_bin),
                    0b001 => self.bne(inst_bin),
                    0b100 => self.blt(inst_bin),
                    0b101 => self.bge(inst_bin),
                    0b110 => self.bltu(inst_bin),
                    0b111 => self.bgeu(inst_bin),
                    _ => self.pc += 4,
                }
            }
            0b0000011 => {
                let funct3 = (inst_bin >> 12) & 0x7;
                match funct3 {
                    0b000 => self.lb(inst_bin, bus),
                    0b001 => self.lh(inst_bin, bus),
                    0b010 => self.lw(inst_bin, bus),
                    0b100 => self.lbu(inst_bin, bus),
                    0b101 => self.lhu(inst_bin, bus),
                    _ => {}
                }
                self.pc += 4;
            }
            0b0100011 => {
                let funct3 = (inst_bin >> 12) & 0x7;
                match funct3 {
                    0b000 => self.sb(inst_bin, bus),
                    0b001 => self.sh(inst_bin, bus),
                    0b010 => self.sw(inst_bin, bus),
                    _ => {}
                }
                self.pc += 4;
            }
            0b0110011 => {
                let funct3 = (inst_bin >> 12) & 0x7;
                let funct7 = (inst_bin >> 25) & 0x7f;
                match (funct3, funct7) {
                    (0b000, 0b0000000) => self.add(inst_bin),
                    (0b000, 0b0100000) => self.sub(inst_bin),
                    (0b001, 0b0000000) => self.sll(inst_bin),
                    (0b010, 0b0000000) => self.slt(inst_bin),
                    (0b011, 0b0000000) => self.sltu(inst_bin),
                    (0b100, 0b0000000) => self.xor(inst_bin),
                    (0b101, 0b0000000) => self.srl(inst_bin),
                    (0b101, 0b0100000) => self.sra(inst_bin),
                    (0b110, 0b0000000) => self.or(inst_bin),
                    (0b111, 0b0000000) => self.and(inst_bin),
                    _ => {}
                }
                self.pc += 4;
            }
            0b0001111 => {
                // FENCE, FENCE.I
                // 現在の実装では NOP 扱い
                self.pc += 4;
            }
            0b1110011 => {
                // SYSTEM (ECALL, EBREAK, CSR instructions)
                let funct3 = (inst_bin >> 12) & 0x7;
                let imm11_0 = (inst_bin >> 20) & 0xfff;

                match (funct3, imm11_0) {
                    (0b000, 0b000000000000) => {
                        // ECALL
                        let code = match self.mode {
                            PrivilegeMode::User => 8,
                            PrivilegeMode::Supervisor => 9,
                            PrivilegeMode::Machine => 11,
                        };
                        self.handle_trap(code);
                    }
                    (0b000, 0b000000000001) => {
                        // EBREAK
                        self.handle_trap(3); // Breakpoint exception code is 3
                    }
                    (0b000, 0b001100000010) => {
                        // MRET
                        self.mret();
                    }
                    _ => {
                        // CSR instructions etc (not implemented yet)
                        self.pc += 4;
                    }
                }
            }
            _ => {
                self.pc += 4;
            }
        }
    }
}
