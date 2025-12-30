#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

#[test]
fn test_addi() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // ADDI x1, x0, 10 (x1 = 0 + 10)
    // opcode: 0010011, funct3: 000, rd: 00001, rs1: 00000, imm: 000000001010
    // 000000001010 00000 000 00001 0010011
    // 0x00a00093
    let inst = 0x00a00093;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 10);
    assert_eq!(cpu.pc, 4);

    // ADDI x2, x1, -5 (x2 = 10 - 5)
    // imm: -5 -> 0xffb (12bit)
    // 111111111011 00001 000 00010 0010011
    // 0xffb08113
    let inst = 0xffb08113;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 5);
    assert_eq!(cpu.pc, 8);

    // ADDI x0, x1, 10 (x0 is always 0)
    let inst = 0x00a08013;
    bus.write_inst32(8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 12);
}

#[test]
fn test_slti() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // SLTI x1, x0, 10 (x1 = 0 < 10 ? 1 : 0) -> 1
    // opcode: 0010011, funct3: 010, rd: 00001, rs1: 00000, imm: 000000001010
    // 000000001010 00000 010 00001 0010011
    // 0x00a02093
    let inst = 0x00a02093;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 1);
    assert_eq!(cpu.pc, 4);

    // SLTI x2, x0, -10 (x2 = 0 < -10 ? 1 : 0) -> 0
    // imm: -10 -> 0xff6 (12bit)
    // 111111110110 00000 010 00010 0010011
    // 0xff602113
    let inst = 0xff602113;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0);
    assert_eq!(cpu.pc, 8);

    // x3 = -20
    cpu.regs[3] = -20i32 as u32;
    // SLTI x4, x3, -10 (x4 = -20 < -10 ? 1 : 0) -> 1
    // 111111110110 00011 010 00100 0010011
    // 0xff61a213
    let inst = 0xff61a213;
    bus.write_inst32(8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[4], 1);
    assert_eq!(cpu.pc, 12);

    // SLTI x5, x3, -30 (x5 = -20 < -30 ? 1 : 0) -> 0
    // imm: -30 -> 0xfe2 (12bit)
    // 111111100010 00011 010 00101 0010011
    // 0xfe21a293
    let inst = 0xfe21a293;
    bus.write_inst32(12, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[5], 0);
    assert_eq!(cpu.pc, 16);
}

#[test]
fn test_sltiu() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // SLTIU x1, x0, 10 (x1 = 0 < 10 ? 1 : 0) -> 1
    // opcode: 0010011, funct3: 011, rd: 00001, rs1: 00000, imm: 000000001010
    // 000000001010 00000 011 00001 0010011
    // 0x00a03093
    let inst = 0x00a03093;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[1], 1);
    assert_eq!(cpu.pc, 4);

    // SLTIU x2, x0, -1 (x2 = 0 < (unsigned)0xffffffff ? 1 : 0) -> 1
    // imm: -1 -> 0xfff (12bit) -> sign_extend -> 0xffffffff
    // 111111111111 00000 011 00010 0010011
    // 0xfff03113
    let inst = 0xfff03113;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 1);
    assert_eq!(cpu.pc, 8);

    // x3 = 0xffffffff
    cpu.regs[3] = 0xffff_ffff;
    // SLTIU x4, x3, 10 (x4 = 0xffffffff < 10 ? 1 : 0) -> 0
    // 000000001010 00011 011 00100 0010011
    // 0x00a1b213
    let inst = 0x00a1b213;
    bus.write_inst32(8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[4], 0);
    assert_eq!(cpu.pc, 12);

    // x5 = 0xfffffffe
    cpu.regs[5] = 0xffff_fffe;
    // SLTIU x6, x5, -1 (x6 = 0xfffffffe < 0xffffffff ? 1 : 0) -> 1
    // imm: -1 -> 0xfff -> 0xffffffff
    // 111111111111 00101 011 00110 0010011
    // 0xfff2b313
    let inst = 0xfff2b313;
    bus.write_inst32(12, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[6], 1);
    assert_eq!(cpu.pc, 16);
}

#[test]
fn test_xori() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // x1 = 0xAAAA_AAAA
    cpu.regs[1] = 0xAAAA_AAAA;
    // XORI x2, x1, 0x555 (x2 = 0xAAAA_AAAA ^ 0x00000555) -> 0xAAAA_AFFF
    // opcode: 0010011, funct3: 100, rd: 00010, rs1: 00001, imm: 010101010101
    // 010101010101 00001 100 00010 0010011
    // 0x5550c113
    let inst = 0x5550c113;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xAAAA_AAAA ^ 0x0000_0555);
    assert_eq!(cpu.pc, 4);

    // XORI x3, x1, -1 (NOT x1) (x3 = 0xAAAA_AAAA ^ 0xFFFF_FFFF) -> 0x5555_5555
    // imm: -1 -> 0xfff (12bit)
    // 111111111111 00001 100 00011 0010011
    // 0xfff0c193
    let inst = 0xfff0c193;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x5555_5555);
    assert_eq!(cpu.pc, 8);
}

#[test]
fn test_ori() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // x1 = 0xAAAA_5555
    cpu.regs[1] = 0xAAAA_5555;
    // ORI x2, x1, 0x555 (x2 = 0xAAAA_5555 | 0x0000_0555) -> 0xAAAA_5555
    // opcode: 0010011, funct3: 110, rd: 00010, rs1: 00001, imm: 010101010101
    // 010101010101 00001 110 00010 0010011
    // 0x5550e113
    let inst = 0x5550e113;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xAAAA_5555 | 0x0000_0555);
    assert_eq!(cpu.pc, 4);

    // ORI x3, x1, -1 (x3 = 0xAAAA_5555 | 0xFFFF_FFFF) -> 0xFFFF_FFFF
    // imm: -1 -> 0xfff (12bit)
    // 111111111111 00001 110 00011 0010011
    // 0xfff0e193
    let inst = 0xfff0e193;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffff_ffff);
    assert_eq!(cpu.pc, 8);
}

#[test]
fn test_andi() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // x1 = 0xAAAA_5555
    cpu.regs[1] = 0xAAAA_5555;
    // ANDI x2, x1, 0x555 (x2 = 0xAAAA_5555 & 0x0000_0555) -> 0x0000_0505
    // opcode: 0010011, funct3: 111, rd: 00010, rs1: 00001, imm: 010101010101
    // 010101010101 00001 111 00010 0010011
    // 0x5550f113
    let inst = 0x5550f113;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xAAAA_5555 & 0x0000_0555);
    assert_eq!(cpu.pc, 4);

    // ANDI x3, x1, -1 (x3 = 0xAAAA_5555 & 0xFFFF_FFFF) -> 0xAAAA_5555
    // imm: -1 -> 0xfff (12bit)
    // 111111111111 00001 111 00011 0010011
    // 0xfff0f193
    let inst = 0xfff0f193;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xAAAA_5555);
    assert_eq!(cpu.pc, 8);
}

#[test]
fn test_slli() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // x1 = 0x0000_0001
    cpu.regs[1] = 0x0000_0001;
    // SLLI x2, x1, 1 (x2 = 0x0000_0001 << 1) -> 0x0000_0002
    // opcode: 0010011, funct3: 001, rd: 00010, rs1: 00001, shamt: 00001, imm[11:5]: 0000000
    // 0000000 00001 00001 001 00010 0010011
    // 0x00109113
    let inst = 0x00109113;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x0000_0002);
    assert_eq!(cpu.pc, 4);

    // x3 = 0x8000_0000
    cpu.regs[3] = 0x8000_0000;
    // SLLI x4, x3, 1 (x4 = 0x8000_0000 << 1) -> 0x0000_0000
    // 0000000 00001 00011 001 00100 0010011
    // 0x00119213
    let inst = 0x00119213;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[4], 0x0000_0000);
    assert_eq!(cpu.pc, 8);

    // x5 = 0x0000_00FF
    cpu.regs[5] = 0x0000_00FF;
    // SLLI x6, x5, 24 (x6 = 0x0000_00FF << 24) -> 0xFF00_0000
    // shamt: 24 (0b11000)
    // 0000000 11000 00101 001 00110 0010011
    // 0x01829313
    let inst = 0x01829313;
    bus.write_inst32(8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[6], 0xFF00_0000);
    assert_eq!(cpu.pc, 12);
}

#[test]
fn test_srli() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // x1 = 0x8000_0000
    cpu.regs[1] = 0x8000_0000;
    // SRLI x2, x1, 1 (x2 = 0x8000_0000 >> 1) -> 0x4000_0000
    // opcode: 0010011, funct3: 101, rd: 00010, rs1: 00001, shamt: 00001, imm[11:5]: 0000000
    // 0000000 00001 00001 101 00010 0010011
    // 0x0010d113
    let inst = 0x0010d113;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0x4000_0000);
    assert_eq!(cpu.pc, 4);

    // x3 = 0xFFFF_FFFF
    cpu.regs[3] = 0xFFFF_FFFF;
    // SRLI x4, x3, 8 (x4 = 0xFFFF_FFFF >> 8) -> 0x00FF_FFFF
    // shamt: 8 (0b01000)
    // 0000000 01000 00011 101 00100 0010011
    // 0x0081d213
    let inst = 0x0081d213;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[4], 0x00FF_FFFF);
    assert_eq!(cpu.pc, 8);
}

#[test]
fn test_srai() {
    let mut cpu = Cpu::new(0);
    let mut bus = MockBus::new();

    // x1 = 0x8000_0000
    cpu.regs[1] = 0x8000_0000;
    // SRAI x2, x1, 1 (x2 = 0x8000_0000 >> 1 arithmetic) -> 0xC000_0000
    // opcode: 0010011, funct3: 101, rd: 00010, rs1: 00001, shamt: 00001, imm[11:5]: 0100000
    // 0100000 00001 00001 101 00010 0010011
    // 0x4010d113
    let inst = 0x4010d113;
    bus.write_inst32(0, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 0xC000_0000);
    assert_eq!(cpu.pc, 4);

    // x3 = 0x0000_0001
    cpu.regs[3] = 0x0000_0001;
    // SRAI x4, x3, 1 (x4 = 0x0000_0001 >> 1 arithmetic) -> 0x0000_0000
    // 0100000 00001 00011 101 00100 0010011
    // 0x4011d213
    let inst = 0x4011d213;
    bus.write_inst32(4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[4], 0x0000_0000);
    assert_eq!(cpu.pc, 8);

    // x5 = 0xFFFF_FFFF (-1)
    cpu.regs[5] = 0xFFFF_FFFF;
    // SRAI x6, x5, 8 (x6 = 0xFFFF_FFFF >> 8 arithmetic) -> 0xFFFF_FFFF
    // shamt: 8 (0b01000)
    // 0100000 01000 00101 101 00110 0010011
    // 0x4082d313
    let inst = 0x4082d313;
    bus.write_inst32(8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[6], 0xFFFF_FFFF);
    assert_eq!(cpu.pc, 12);
}
