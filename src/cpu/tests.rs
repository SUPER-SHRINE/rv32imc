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

impl Bus for MockBus {
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

// jal 命令によって pc + 4 がレジスタに設定され、PC がジャンプすることを確認
#[test]
fn test_jal() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // JAL x1, 0x100 (imm=0x100, rd=1, opcode=1101111)
    // imm[20]=0, imm[10:1]=0x80, imm[11]=0, imm[19:12]=0
    // inst[31]=0, inst[30:21]=0x80, inst[20]=0, inst[19:12]=0
    // inst = 0x00000000 | (0x80 << 21) | (0 << 20) | (0 << 12) | (1 << 7) | 0x6f
    //      = 0x100000ef
    let inst_bin = 0x100000ef;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1004);
    assert_eq!(cpu.pc, 0x1100);
}

// jal 命令で x0 レジスタが指定された場合、戻り先アドレスが保存されないことを確認
#[test]
fn test_jal_x0() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // JAL x0, 0x100
    let inst_bin = 0x1000006f;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1100);
}

// jal 命令で負のオフセットを指定した場合の動作を確認
#[test]
fn test_jal_neg() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // JAL x1, -0x100
    // imm = -0x100 = 0xffffff00
    // imm[20:1] = 0x1ff00 >> 1 = 0xff80
    // imm[20] = 1 (inst[31])
    // imm[10:1] = 0x380 (inst[30:21])
    // imm[11] = 1 (inst[20])
    // imm[19:12] = 0xff (inst[19:12])
    // inst = (1 << 31) | (0x380 << 21) | (1 << 20) | (0xff << 12) | (1 << 7) | 0x6f
    //      = 0xf00ff0ef (Corrected: inst[30:21] is 0x380, so 0x380 << 21 = 0x70000000)
    // 0x80000000 | 0x70000000 | 0x00100000 | 0x000ff000 | 0x00000080 | 0x6f
    // = 0xf01ff0ef
    let inst_bin = 0xf01ff0ef;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1004);
    assert_eq!(cpu.pc, 0x0f00);
}

// jalr 命令によってターゲットアドレスへジャンプし、pc + 4 がレジスタに設定されることを確認
#[test]
fn test_jalr() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[2] = 0x2000;
    // JALR x1, 0x10(x2) (imm=0x10, rs1=2, funct3=0, rd=1, opcode=1100111)
    // inst = 0x010100e7
    let inst_bin = 0x010100e7;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1004);
    assert_eq!(cpu.pc, 0x2010);
}

// jalr 命令でターゲットアドレスの最下位ビットがクリアされることを確認
#[test]
fn test_jalr_align() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[2] = 0x2001;
    // JALR x0, 0x11(x2) (imm=0x11, rs1=2, funct3=0, rd=0, opcode=1100111)
    // 0x2001 + 0x11 = 0x2012. 0x2012 & ~1 = 0x2012
    // inst = 0x01110067
    let inst_bin = 0x01110067;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x2012);

    // 最下位ビットがセットされるケース
    cpu.pc = 0x1004;
    cpu.regs[2] = 0x2000;
    // JALR x0, 0x11(x2) -> 0x2011 & ~1 = 0x2010
    let inst_bin = 0x01110067;
    bus.write_inst32(0x1004, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x2010);
}

// beq 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_beq() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[1] = 10;
    cpu.regs[2] = 10;
    // BEQ x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=0, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (0 << 12) | (0 << 7) | 0x63
    //      = 0x10208063
    let inst_bin = 0x10208063;
    bus.write_inst32(0x1000, inst_bin);

    // 条件一致
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 条件不一致
    cpu.pc = 0x1000;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// beq 命令で負のオフセットを指定した場合の動作を確認
#[test]
fn test_beq_neg() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[1] = 10;
    cpu.regs[2] = 10;
    // BEQ x1, x2, -0x100 (imm=-0x100 = 0xffffff00)
    // imm[12]=1, imm[11]=1, imm[10:5]=0x38, imm[4:1]=0
    // inst[31]=1, inst[7]=1, inst[30:25]=0x38, inst[11:8]=0
    // inst = (1 << 31) | (0x38 << 25) | (2 << 20) | (1 << 15) | (0 << 12) | (1 << 7) | 0x63
    //      = 0xf02080e3
    let inst_bin = 0xf02080e3;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x0f00);
}

// bne 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bne() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    // BNE x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=1, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (1 << 12) | (0 << 7) | 0x63
    //      = 0x10209063
    let inst_bin = 0x10209063;
    bus.write_inst32(0x1000, inst_bin);

    // 条件一致 (10 != 20)
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 条件不一致 (10 != 10)
    cpu.pc = 0x1000;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// blt 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_blt() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 < rs2 (10 < 20) -> ジャンプ
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    // BLT x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=4, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (4 << 12) | (0 << 7) | 0x63
    //      = 0x1020c063
    let inst_bin = 0x1020c063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件不一致: rs1 == rs2 (20 == 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 3. 条件不一致: rs1 > rs2 (30 > 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 30;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号付き比較の確認: rs1 < rs2 (-10 < 10) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号付き比較の確認: rs1 > rs2 (10 > -10) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// bge 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bge() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 > rs2 (20 > 10) -> ジャンプ
    cpu.regs[1] = 20;
    cpu.regs[2] = 10;
    // BGE x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=5, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (5 << 12) | (0 << 7) | 0x63
    //      = 0x1020d063
    let inst_bin = 0x1020d063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件一致: rs1 == rs2 (20 == 20) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 3. 条件不一致: rs1 < rs2 (10 < 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号付き比較の確認: rs1 > rs2 (10 > -10) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号付き比較の確認: rs1 < rs2 (-10 < 10) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// bltu 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bltu() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 < rs2 (10 < 20) -> ジャンプ
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    // BLTU x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=6, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (6 << 12) | (0 << 7) | 0x63
    //      = 0x1020e063
    let inst_bin = 0x1020e063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件不一致: rs1 == rs2 (20 == 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 3. 条件不一致: rs1 > rs2 (30 > 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 30;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号なし比較の確認: rs1 < rs2 (10 < -10 as u32) -> ジャンプ
    // -10i32 as u32 は 0xfffffff6 なので、10 (0x0000000a) より大きい
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号なし比較の確認: rs1 > rs2 (-10 as u32 > 10) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// bgeu 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bgeu() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 > rs2 (20 > 10) -> ジャンプ
    cpu.regs[1] = 20;
    cpu.regs[2] = 10;
    // BGEU x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=7, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (7 << 12) | (0 << 7) | 0x63
    //      = 0x1020f063
    let inst_bin = 0x1020f063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件一致: rs1 == rs2 (20 == 20) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 3. 条件不一致: rs1 < rs2 (10 < 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号なし比較の確認: rs1 > rs2 (-10 as u32 > 10) -> ジャンプ
    // -10i32 as u32 は 0xfffffff6 なので、10 (0x0000000a) より大きい
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号なし比較の確認: rs1 < rs2 (10 < -10 as u32) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

#[test]
fn test_lb() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // LB x1, 4(x2) (rd=1, rs1=2, funct3=0, imm=4, opcode=0000011)
    // inst = (4 << 20) | (2 << 15) | (0 << 12) | (1 << 7) | 0x03
    //      = 0x00410083
    let inst_bin = 0x00410083;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 正の値をロード
    cpu.regs[2] = 0x1000;
    bus.write8(0x1004, 0x7F);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x7F);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負の値をロード (符号拡張)
    cpu.pc = 0x1000;
    bus.write8(0x1004, 0x80);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0xFFFF_FF80);
    assert_eq!(cpu.pc, 0x1004);

    // 3. x0 レジスタへのロード (無視される)
    // LB x0, 4(x2) (rd=0, rs1=2, funct3=0, imm=4, opcode=0000011)
    let inst_bin = 0x00410003;
    bus.write_inst32(0x1000, inst_bin);
    cpu.pc = 0x1000;
    bus.write8(0x1004, 0x55);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1004);
}

#[test]
fn test_lh() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // LH x1, 4(x2) (rd=1, rs1=2, funct3=1, imm=4, opcode=0000011)
    // inst = (4 << 20) | (2 << 15) | (1 << 12) | (1 << 7) | 0x03
    //      = 0x00411083
    let inst_bin = 0x00411083;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 正の値をロード
    cpu.regs[2] = 0x1000;
    bus.write16(0x1004, 0x7FFF);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x7FFF);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負の値をロード (符号拡張)
    cpu.pc = 0x1000;
    bus.write16(0x1004, 0x8000);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0xFFFF_8000);
    assert_eq!(cpu.pc, 0x1004);

    // 3. x0 レジスタへのロード (無視される)
    // LH x0, 4(x2) (rd=0, rs1=2, funct3=1, imm=4, opcode=0000011)
    let inst_bin = 0x00411003;
    bus.write_inst32(0x1000, inst_bin);
    cpu.pc = 0x1000;
    bus.write16(0x1004, 0x1234);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1004);
}

#[test]
fn test_lw() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // LW x1, 4(x2) (rd=1, rs1=2, funct3=2, imm=4, opcode=0000011)
    // inst = (4 << 20) | (2 << 15) | (2 << 12) | (1 << 7) | 0x03
    //      = 0x00412083
    let inst_bin = 0x00412083;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 通常のロード
    cpu.regs[2] = 0x1000;
    bus.write32(0x1004, 0x12345678);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x12345678);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負の値（のように見える値）のロード
    cpu.pc = 0x1000;
    bus.write32(0x1004, 0x80000000);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x80000000);
    assert_eq!(cpu.pc, 0x1004);

    // 3. x0 レジスタへのロード (無視される)
    // LW x0, 4(x2) (rd=0, rs1=2, funct3=2, imm=4, opcode=0000011)
    let inst_bin = 0x00412003;
    bus.write_inst32(0x1000, inst_bin);
    cpu.pc = 0x1000;
    bus.write32(0x1004, 0xDEADBEEF);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1004);
}
