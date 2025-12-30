use crate::cpu::Cpu;
use crate::cpu::tests::mock_bus::{Bus, MockBus};

#[test]
fn test_sb() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // SB x2, 4(x1) (rs1=1, rs2=2, funct3=0, imm=4, opcode=0100011)
    // imm[11:5] = 0000000
    // rs2 = 00010 (x2)
    // rs1 = 00001 (x1)
    // funct3 = 000
    // imm[4:0] = 00100
    // opcode = 0100011
    // inst = 0000000_00010_00001_000_00100_0100011
    //      = 0x00208223
    let inst_bin = 0x00208223;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 通常のストア
    cpu.regs[1] = 0x1000;
    cpu.regs[2] = 0x12345678;
    cpu.step(&mut bus);
    assert_eq!(bus.read8(0x1004), 0x78);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負のオフセット
    // SB x2, -4(x1) (rs1=1, rs2=2, funct3=0, imm=-4, opcode=0100011)
    // imm = -4 = 0b1111_1111_1100
    // imm[11:5] = 1111111
    // imm[4:0] = 11100
    // inst = 1111111_00010_00001_000_11100_0100011
    //      = 0xFE208E23
    let inst_bin = 0xFE208E23;
    bus.write_inst32(0x1004, inst_bin);
    cpu.regs[1] = 0x1008;
    cpu.regs[2] = 0xDEADBEEF;
    cpu.step(&mut bus);
    assert_eq!(bus.read8(0x1004), 0xEF);
    assert_eq!(cpu.pc, 0x1008);
}

#[test]
fn test_sh() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // SH x2, 4(x1) (rs1=1, rs2=2, funct3=1, imm=4, opcode=0100011)
    // imm[11:5] = 0000000
    // rs2 = 00010 (x2)
    // rs1 = 00001 (x1)
    // funct3 = 001
    // imm[4:0] = 00100
    // opcode = 0100011
    // inst = 0000000_00010_00001_001_00100_0100011
    //      = 0x00209223
    let inst_bin = 0x00209223;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 通常のストア
    cpu.regs[1] = 0x1000;
    cpu.regs[2] = 0x12345678;
    cpu.step(&mut bus);
    assert_eq!(bus.read16(0x1004), 0x5678);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負のオフセット
    // SH x2, -4(x1) (rs1=1, rs2=2, funct3=1, imm=-4, opcode=0100011)
    // imm = -4 = 0b1111_1111_1100
    // imm[11:5] = 1111111
    // imm[4:0] = 11100
    // inst = 1111111_00010_00001_001_11100_0100011
    //      = 0xFE209E23
    let inst_bin = 0xFE209E23;
    bus.write_inst32(0x1004, inst_bin);
    cpu.regs[1] = 0x1008;
    cpu.regs[2] = 0xDEADBEEF;
    cpu.step(&mut bus);
    assert_eq!(bus.read16(0x1004), 0xBEEF);
    assert_eq!(cpu.pc, 0x1008);
}

#[test]
fn test_sw() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // SW x2, 4(x1) (rs1=1, rs2=2, funct3=2, imm=4, opcode=0100011)
    // imm[11:5] = 0000000
    // rs2 = 00010 (x2)
    // rs1 = 00001 (x1)
    // funct3 = 010
    // imm[4:0] = 00100
    // opcode = 0100011
    // inst = 0000000_00010_00001_010_00100_0100011
    //      = 0x0020A223
    let inst_bin = 0x0020A223;
    bus.write_inst32(0x1000, inst_bin);

    // 1. 通常のストア
    cpu.regs[1] = 0x1000;
    cpu.regs[2] = 0x12345678;
    cpu.step(&mut bus);
    assert_eq!(bus.read32(0x1004), 0x12345678);
    assert_eq!(cpu.pc, 0x1004);

    // 2. 負のオフセット
    // SW x2, -4(x1) (rs1=1, rs2=2, funct3=2, imm=-4, opcode=0100011)
    // imm = -4 = 0b1111_1111_1100
    // imm[11:5] = 1111111
    // imm[4:0] = 11100
    // inst = 1111111_00010_00001_010_11100_0100011
    //      = 0xFE20AE23
    let inst_bin = 0xFE20AE23;
    bus.write_inst32(0x1004, inst_bin);
    cpu.regs[1] = 0x1008;
    cpu.regs[2] = 0xDEADBEEF;
    cpu.step(&mut bus);
    assert_eq!(bus.read32(0x1004), 0xDEADBEEF);
    assert_eq!(cpu.pc, 0x1008);
}
