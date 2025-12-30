mod csr;
mod decode;
mod execute;
mod handle_trap;
mod privilege_mode;

#[cfg(test)]
mod tests;

use super::bus;
use csr::Csr;
use privilege_mode::PrivilegeMode;

pub enum StepResult {
    Ok,
    Trap(u32),
}

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
    pub fn step<B: bus::Bus>(&mut self, bus: &mut B) -> StepResult {
        let inst_bin = self.fetch(bus);
        self.execute(inst_bin, bus)
    }

    /// レジスタの状態をダンプ
    pub fn dump_registers(&self) {
        for (i, reg) in self.regs.iter().enumerate() {
            println!("x{:02}: 0x{:08x}", i, reg);
        }
        println!("pc : 0x{:08x}", self.pc);
    }

    fn fetch<B: bus::Bus>(&mut self, bus: &mut B) -> u32 {
        bus.read32(self.pc)
    }

    fn execute<B: bus::Bus>(&mut self, inst_bin: u32, bus: &mut B) -> StepResult {
        let opcode = inst_bin & 0x7f;
        match opcode {
            0b0110111 => self.lui(inst_bin),
            0b0010111 => self.auipc(inst_bin),
            0b1101111 => self.jal(inst_bin),
            0b1100111 => self.jalr(inst_bin),
            0b0010011 => match self.decode_funct3(inst_bin) {
                0b000 => self.addi(inst_bin),
                0b001 => self.slli(inst_bin),
                0b010 => self.slti(inst_bin),
                0b011 => self.sltiu(inst_bin),
                0b100 => self.xori(inst_bin),
                0b101 => match self.decode_funct7(inst_bin) {
                    0b0000000 => self.srli(inst_bin),
                    0b0100000 => self.srai(inst_bin),
                    _ => self.handle_trap(2),
                },
                0b110 => self.ori(inst_bin),
                0b111 => self.andi(inst_bin),
                _ => self.handle_trap(2),
            },
            0b1100011 => match self.decode_funct3(inst_bin) {
                0b000 => self.beq(inst_bin),
                0b001 => self.bne(inst_bin),
                0b100 => self.blt(inst_bin),
                0b101 => self.bge(inst_bin),
                0b110 => self.bltu(inst_bin),
                0b111 => self.bgeu(inst_bin),
                _ => self.handle_trap(2),
            },
            0b0000011 => match self.decode_funct3(inst_bin) {
                0b000 => self.lb(inst_bin, bus),
                0b001 => self.lh(inst_bin, bus),
                0b010 => self.lw(inst_bin, bus),
                0b100 => self.lbu(inst_bin, bus),
                0b101 => self.lhu(inst_bin, bus),
                _ => self.handle_trap(2),
            },
            0b0100011 => match self.decode_funct3(inst_bin) {
                    0b000 => self.sb(inst_bin, bus),
                    0b001 => self.sh(inst_bin, bus),
                    0b010 => self.sw(inst_bin, bus),
                _ => self.handle_trap(2),
            },
            0b0110011 => match self.decode_funct3(inst_bin) {
                0b000 => match self.decode_funct7(inst_bin) {
                    0b0000000 => self.add(inst_bin),
                    0b0100000 => self.sub(inst_bin),
                    0b0000001 => self.mul(inst_bin),
                    _ => self.handle_trap(2),
                },
                0b001 => match self.decode_funct7(inst_bin) {
                    0b0000000 => self.sll(inst_bin),
                    0b0000001 => self.mulh(inst_bin),
                    _ => self.handle_trap(2),
                },
                0b011 => self.sltu(inst_bin),
                0b010 => match self.decode_funct7(inst_bin) {
                    0b0000000 => self.slt(inst_bin),
                    0b0000001 => self.mulhsu(inst_bin),
                    _ => self.handle_trap(2),
                },
                0b100 => self.xor(inst_bin),
                0b101 => match self.decode_funct7(inst_bin) {
                    0b0000000 => self.srl(inst_bin),
                    0b0100000 => self.sra(inst_bin),
                    _ => self.handle_trap(2),
                },
                0b110 => self.or(inst_bin),
                0b111 => self.and(inst_bin),
                _ => self.handle_trap(2),
            }
            0b0001111 => match self.decode_funct3(inst_bin) {
                0b000 => self.fence(),
                0b001 => self.fence_i(),
                _ => self.handle_trap(2),
            },
            0b1110011 => match self.decode_funct3(inst_bin) {
                0b000 => match (inst_bin >> 20) & 0xfff {
                    0b000000000000 => self.ecall(),
                    0b000000000001 => self.ebreak(),
                    0b001100000010 => self.mret(),
                    _ => self.handle_trap(2),
                },
                0b001 => self.csrrw(inst_bin),
                0b010 => self.csrrs(inst_bin),
                0b011 => self.csrrc(inst_bin),
                0b101 => self.csrrwi(inst_bin),
                0b110 => self.csrrsi(inst_bin),
                0b111 => self.csrrci(inst_bin),
                _ => self.handle_trap(2),
            },
            _ => self.handle_trap(2),
        }
    }
}
