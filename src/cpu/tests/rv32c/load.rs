use crate::cpu::Cpu;
use crate::cpu::tests::mock_bus::{Bus, MockBus};

#[test]
fn test_c_lw() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // メモリ準備: アドレス 0x100 に 0x12345678 を書き込む
    bus.write32(0x100, 0x12345678);

    // rs1 = x8 (s0) = 0x100
    cpu.regs[8] = 0x100;

    // c.lw x9, 0(x8)
    // quadrant: 00
    // funct3: 010
    // rs1': 000 (x8)
    // rd': 001 (x9)
    // imm: 0 (imm[5:3]=000, imm[2]=0, imm[6]=0)
    // inst: 010 000 000 0 0 001 00 -> 0x4004
    let inst = 0x4004;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[9], 0x12345678);
    assert_eq!(cpu.pc, 0x2);
}

#[test]
fn test_c_lw_offset() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // メモリ準備: アドレス 0x100 + 124 = 0x17C に 0xDEADBEEF を書き込む
    bus.write32(0x17C, 0xDEADBEEF);

    // rs1 = x8 (s0) = 0x100
    cpu.regs[8] = 0x100;

    // c.lw x15, 124(x8)
    // imm: 124 = 0b1111100 -> imm[6]=1, imm[5:3]=111, imm[2]=1
    // rs1': 000 (x8)
    // rd': 111 (x15)
    // inst: 010 (funct3) 111 (imm[5:3]) 000 (rs1') 1 (imm[2]) 1 (imm[6]) 111 (rd') 00 (op)
    // inst bits: 010 111 000 1 1 111 00 -> 0b0101_1100_0111_1100 = 0x5c7c
    let inst = 0x5c7c;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[15], 0xDEADBEEF);
    assert_eq!(cpu.pc, 0x2);
}

#[test]
fn test_c_lw_various_regs() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // メモリ準備: アドレス 0x200 に 0xAAAA_BBBB を書き込む
    bus.write32(0x204, 0xAAAABBBB);

    // rs1 = x15 (a7) = 0x200
    cpu.regs[15] = 0x200;

    // c.lw x8, 4(x15)
    // imm: 4 = 0b0000100 -> imm[6]=0, imm[5:3]=000, imm[2]=1
    // rs1': 111 (x15)
    // rd': 000 (x8)
    // inst: 010 (funct3) 000 (imm[5:3]) 111 (rs1') 1 (imm[2]) 0 (imm[6]) 000 (rd') 00 (op)
    // inst bits: 010 000 111 1 0 000 00 -> 0b0100_0011_1100_0000 = 0x43c0
    let inst = 0x43c0;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0xAAAABBBB);
    assert_eq!(cpu.pc, 0x2);
}
