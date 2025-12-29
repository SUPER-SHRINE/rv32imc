use crate::cpu::Cpu;
use super::{Bus, MockBus};

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

#[test]
fn test_lbu() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // LBU x1, 4(x2) (rd=1, rs1=2, funct3=4, imm=4, opcode=0000011)
    // inst = (4 << 20) | (2 << 15) | (4 << 12) | (1 << 7) | 0x03
    //      = 0x00414083
    let inst_bin = 0x00414083;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 正の値をロード
    cpu.regs[2] = 0x1000;
    bus.write8(0x1004, 0x7F);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x7F);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負の値（のように見える値）をロード (ゼロ拡張)
    cpu.pc = 0x1000;
    bus.write8(0x1004, 0x80);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x80);
    assert_eq!(cpu.pc, 0x1004);

    // 3. x0 レジスタへのロード (無視される)
    // LBU x0, 4(x2) (rd=0, rs1=2, funct3=4, imm=4, opcode=0000011)
    let inst_bin = 0x00414003;
    bus.write_inst32(0x1000, inst_bin);
    cpu.pc = 0x1000;
    bus.write8(0x1004, 0x55);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1004);
}

#[test]
fn test_lhu() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // LHU x1, 4(x2) (rd=1, rs1=2, funct3=5, imm=4, opcode=0000011)
    // inst = (4 << 20) | (2 << 15) | (5 << 12) | (1 << 7) | 0x03
    //      = 0x00415083
    let inst_bin = 0x00415083;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 正の値をロード
    cpu.regs[2] = 0x1000;
    bus.write16(0x1004, 0x7FFF);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x7FFF);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負の値（のように見える値）をロード (ゼロ拡張)
    cpu.pc = 0x1000;
    bus.write16(0x1004, 0x8000);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 0x8000);
    assert_eq!(cpu.pc, 0x1004);

    // 3. x0 レジスタへのロード (無視される)
    // LHU x0, 4(x2) (rd=0, rs1=2, funct3=5, imm=4, opcode=0000011)
    let inst_bin = 0x00415003;
    bus.write_inst32(0x1000, inst_bin);
    cpu.pc = 0x1000;
    bus.write16(0x1004, 0xAAAA);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1004);
}
