use super::bus;

/// RISC-V の特権モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeMode {
    User       = 0,
    Supervisor = 1,
    Machine    = 3,
}

/// 制御ステータスレジスタ (CSR)
#[derive(Default)]
pub struct Csr {
    // 主要なマシンモードCSR
    pub mstatus: u32,
    pub mtvec:   u32,
    pub mie:     u32,
    pub mepc:    u32,
    pub mcause:  u32,
    pub mtval:   u32,
    pub mip:     u32,
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
    pub fn step<B: bus::Bus>(&mut self, bus: &mut B) {
        let inst_bin = self.fetch(bus);
        let inst = self.decode(inst_bin);
        self.execute(inst, bus);
    }

    /// レジスタの状態をダンプ
    pub fn dump_registers(&self) {
        for (i, reg) in self.regs.iter().enumerate() {
            println!("x{:02}: 0x{:08x}", i, reg);
        }
        println!("pc : 0x{:08x}", self.pc);
    }

    fn fetch<B: bus::Bus>(&mut self, _bus: &mut B) -> u32 {
        // TODO: 命令フェッチの実装
        0
    }

    fn decode(&self, _inst_bin: u32) -> Instruction {
        // TODO: デコードの実装
        Instruction::Nop
    }

    fn execute<B: bus::Bus>(&mut self, _inst: Instruction, _bus: &mut B) {
        // TODO: 命令実行の実装
    }
}

/// デコードされた命令
enum Instruction {
    Nop,

    // RV32I
    // TODO: RV32I の命令を定義する

    // RV32M
    // TODO: M拡張の命令を定義する

    // RV32C
    // TODO: C拡張の命令を定義する
}

#[cfg(test)]
mod test {
    // TODO: テストはここに書いていく
}
