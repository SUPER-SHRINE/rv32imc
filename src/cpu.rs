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
        self.execute(inst_bin);
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

    fn execute(&mut self, inst_bin: u32) {
        let opcode = inst_bin & 0x7f;
        match opcode {
            0b0110111 => self.lui(inst_bin),
            0b0010111 => self.auipc(inst_bin),
            _ => { }
        }
        self.pc += 4;
    }

    fn decode_u_type(&self, inst_bin: u32) -> (usize, u32) {
        let rd = ((inst_bin >> 7) & 0x1f) as usize;
        let imm = inst_bin & 0xffff_f000;
        (rd, imm)
    }

    fn lui(&mut self, inst_bin: u32) {
        let (rd, imm) = self.decode_u_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = imm;
        }
    }

    fn auipc(&mut self, inst_bin: u32) {
        let (rd, imm) = self.decode_u_type(inst_bin);
        if rd != 0 {
            self.regs[rd] = self.pc.wrapping_add(imm);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;

    struct MockBus {
        memory: [u8; 8192],
    }

    impl MockBus {
        fn new() -> Self {
            Self {
                memory: [0; 8192],
            }
        }

        fn write_inst32(&mut self, addr: u32, inst: u32) {
            self.write32(addr, inst);
        }
    }

    impl bus::Bus for MockBus {
        fn read8(&mut self, addr: u32) -> u8 {
            self.memory[addr as usize]
        }

        fn read16(&mut self, addr: u32) -> u16 {
            let addr = addr as usize;
            u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]])
        }

        fn read32(&mut self, addr: u32) -> u32 {
            let addr = addr as usize;
            u32::from_le_bytes([
                self.memory[addr],
                self.memory[addr + 1],
                self.memory[addr + 2],
                self.memory[addr + 3],
            ])
        }

        fn write8(&mut self, addr: u32, val: u8) {
            self.memory[addr as usize] = val;
        }

        fn write16(&mut self, addr: u32, val: u16) {
            let addr = addr as usize;
            self.memory[addr..addr + 2].copy_from_slice(&val.to_le_bytes());
        }

        fn write32(&mut self, addr: u32, val: u32) {
            let addr = addr as usize;
            self.memory[addr..addr + 4].copy_from_slice(&val.to_le_bytes());
        }
    }

    // lui 命令によってレジスタの値が正しく設定され、PC が +4 進むことを確認
    #[test]
    fn test_lui() {
        let mut cpu = Cpu::new(0x0);
        let mut bus = MockBus::new();

        // LUI x1, 0x12345 (imm=0x12345000, rd=1, opcode=0110111)
        // 0x12345000 | (1 << 7) | 0x37 = 0x123450b7
        let inst_bin = 0x123450b7;
        bus.write_inst32(0, inst_bin);
        
        cpu.step(&mut bus);

        assert_eq!(cpu.regs[1], 0x12345000);
        assert_eq!(cpu.pc, 4);
    }

    // lui 命令によって x0 レジスタの値が書き換わらず、PC が +4 進むことを確認
    #[test]
    fn test_lui_x0() {
        let mut cpu = Cpu::new(0x0);
        let mut bus = MockBus::new();

        // LUI x0, 0x12345
        let inst_bin = 0x12345037;
        bus.write_inst32(0, inst_bin);

        cpu.step(&mut bus);

        assert_eq!(cpu.regs[0], 0);
        assert_eq!(cpu.pc, 4);
    }

    // auipc 命令によって PC + 0x12345000 がレジスタに設定され、PC が +4 進むことを確認
    #[test]
    fn test_auipc() {
        let mut cpu = Cpu::new(0x1000);
        let mut bus = MockBus::new();

        // AUIPC x1, 0x12345 (imm=0x12345000, rd=1, opcode=0010111)
        // 0x12345000 | (1 << 7) | 0x17 = 0x12345097
        let inst_bin = 0x12345097;
        bus.write_inst32(0x1000, inst_bin);

        cpu.step(&mut bus);

        assert_eq!(cpu.regs[1], 0x1000 + 0x12345000);
        assert_eq!(cpu.pc, 0x1004);
    }

    // auipc 命令によって x0 レジスタの値が書き換わらず、PC が +4 進むことを確認
    #[test]
    fn test_auipc_x0() {
        let mut cpu = Cpu::new(0x1000);
        let mut bus = MockBus::new();

        // AUIPC x0, 0x12345
        let inst_bin = 0x12345017;
        bus.write_inst32(0x1000, inst_bin);

        cpu.step(&mut bus);

        assert_eq!(cpu.regs[0], 0);
        assert_eq!(cpu.pc, 0x1004);
    }
}
