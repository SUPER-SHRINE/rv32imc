use crate::cpu::Cpu;
use crate::cpu::tests::mock_bus::{Bus, MockBus};

#[test]
fn test_c_sw() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // rs1 = x8 (s0) = 0x100
    cpu.regs[8] = 0x100;
    // rs2 = x9 (s1) = 0x12345678
    cpu.regs[9] = 0x12345678;

    // c.sw x9, 0(x8)
    // quadrant: 00
    // funct3: 110
    // rs1': 000 (x8)
    // rs2': 001 (x9)
    // imm: 0 (imm[5:3]=000, imm[2]=0, imm[6]=0)
    // inst bits: 110 000 000 0 0 001 00 -> 0b1100_0000_0000_0100 = 0xc004
    let inst = 0xc004;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(bus.read32(0x100), 0x12345678);
    assert_eq!(cpu.pc, 0x2);
}

#[test]
fn test_c_sw_offset() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // rs1 = x8 (s0) = 0x100
    cpu.regs[8] = 0x100;
    // rs2 = x15 (a7) = 0xDEADBEEF
    cpu.regs[15] = 0xDEADBEEF;

    // c.sw x15, 124(x8)
    // imm: 124 = 0b1111100 -> imm[6]=1, imm[5:3]=111, imm[2]=1
    // rs1': 000 (x8)
    // rs2': 111 (x15)
    // inst bits: 110 (funct3) 111 (imm[5:3]) 000 (rs1') 1 (imm[2]) 1 (imm[6]) 111 (rs2') 00 (op)
    // inst bits: 110 111 000 1 1 111 00 -> 0b1101_1100_0111_1100 = 0xdc7c
    let inst = 0xdc7c;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(bus.read32(0x17C), 0xDEADBEEF);
    assert_eq!(cpu.pc, 0x2);
}

#[test]
fn test_c_sw_various_regs() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // rs1 = x15 (a7) = 0x200
    cpu.regs[15] = 0x200;
    // rs2 = x8 (s0) = 0xAAAABBBB
    cpu.regs[8] = 0xAAAABBBB;

    // c.sw x8, 4(x15)
    // imm: 4 = 0b0000100 -> imm[6]=0, imm[5:3]=000, imm[2]=1
    // rs1': 111 (x15)
    // rs2': 000 (x8)
    // inst bits: 110 (funct3) 000 (imm[5:3]) 111 (rs1') 1 (imm[2]) 0 (imm[6]) 000 (rs2') 00 (op)
    // inst bits: 110 000 111 1 0 000 00 -> 0b1100_0011_1100_0000 = 0xc3c0
    let inst = 0xc3c0;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(bus.read32(0x204), 0xAAAABBBB);
    assert_eq!(cpu.pc, 0x2);
}
