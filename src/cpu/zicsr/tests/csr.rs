#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

#[test]
fn test_csrrw() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // x1 = 0x12345678
    cpu.regs[1] = 0x12345678;
    // mtvec = 0xABCDEF00 (Initial value)
    cpu.csr.mtvec = 0xABCDEF00;

    // CSRRW x2, mtvec, x1
    // opcode: 0b1110011 (73)
    // funct3: 0b001
    // rd: x2 (2)
    // rs1: x1 (1)
    // csr: 0x305 (mtvec)
    // 0x305_09_1_73 -> 0x30509173
    let inst = 0x30509173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mtvec value
    assert_eq!(cpu.regs[2], 0xABCDEF00);
    // mtvec should have the new x1 value
    assert_eq!(cpu.csr.mtvec, 0x12345678);
    // PC should increment
    assert_eq!(cpu.pc, 0x0104);
}

#[test]
fn test_csrrw_x0_rd() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // x1 = 0x12345678
    cpu.regs[1] = 0x12345678;
    cpu.csr.mepc = 0x0;

    // CSRRW x0, mepc, x1
    // rd: x0 (0)
    // 0x341_09_0_73 -> 0x34109073
    let inst = 0x34109073;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x0 should remain 0
    assert_eq!(cpu.regs[0], 0);
    // mepc should be updated
    assert_eq!(cpu.csr.mepc, 0x12345678);
}

#[test]
fn test_csrrs() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // x1 = 0b0011 (bits to set)
    cpu.regs[1] = 0b0011;
    // mstatus = 0b1100 (initial value)
    cpu.csr.mstatus = 0b1100;

    // CSRRS x2, mstatus, x1
    // opcode: 0b1110011 (73)
    // funct3: 0b010
    // rd: x2 (2)
    // rs1: x1 (1)
    // csr: 0x300 (mstatus)
    // 0x300_0a_1_73 -> 0x3000a173
    let inst = 0x3000a173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (0b1100 = 12)
    assert_eq!(cpu.regs[2], 12);
    // mstatus should be 0b1100 | 0b0011 = 0b1111 (15)
    assert_eq!(cpu.csr.read(0x300).unwrap(), 15);
}

#[test]
fn test_csrrs_x0_rs1() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mstatus = 0b1100 (initial value)
    cpu.csr.mstatus = 0b1100;

    // CSRRS x2, mstatus, x0
    // rs1: x0 (0)
    // 0x300_0a_0_73 -> 0x30002173
    let inst = 0x30002173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (12)
    assert_eq!(cpu.regs[2], 12);
    // mstatus should NOT be updated
    assert_eq!(cpu.csr.read(0x300).unwrap(), 12);
}

#[test]
fn test_csrrc() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // x1 = 0b1010 (bits to clear)
    cpu.regs[1] = 0b1010;
    // mstatus = 0b1111 (initial value)
    cpu.csr.mstatus = 0b1111;

    // CSRRC x2, mstatus, x1
    // opcode: 0b1110011 (73)
    // funct3: 0b011
    // rd: x2 (2)
    // rs1: x1 (1)
    // csr: 0x300 (mstatus)
    // 0x300_0b_1_73 -> 0x3000b173
    let inst = 0x3000b173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (0b1111 = 15)
    assert_eq!(cpu.regs[2], 15);
    // mstatus should be 0b1111 & !0b1010 = 0b0101 (5)
    assert_eq!(cpu.csr.read(0x300).unwrap(), 5);
}

#[test]
fn test_csrrc_x0_rs1() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mstatus = 0b1111 (initial value)
    cpu.csr.mstatus = 0b1111;

    // CSRRC x2, mstatus, x0
    // rs1: x0 (0)
    // 0x300_0b_0_73 -> 0x30003173
    let inst = 0x30003173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (15)
    assert_eq!(cpu.regs[2], 15);
    // mstatus should NOT be updated
    assert_eq!(cpu.csr.read(0x300).unwrap(), 15);
}

#[test]
fn test_csrrwi() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mepc = 0x12345678 (initial value)
    cpu.csr.mepc = 0x12345678;

    // CSRRWI x2, mepc, 31
    // opcode: 0b1110011 (73)
    // funct3: 0b101
    // rd: x2 (2)
    // uimm: 31 (0x1f)
    // csr: 0x341 (mepc)
    // 0x341fd173
    let inst = 0x341fd173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mepc value
    assert_eq!(cpu.regs[2], 0x12345678);
    // mepc should be updated with uimm (31)
    assert_eq!(cpu.csr.mepc, 31);
}

#[test]
fn test_csrrwi_x0_rd() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mepc = 0x12345678 (initial value)
    cpu.csr.mepc = 0x12345678;

    // CSRRWI x0, mepc, 15
    // rd: x0 (0)
    // uimm: 15 (0x0f)
    // 0x341_0f_5_00_73 -> 0x3417d073
    let inst = 0x3417d073;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x0 should remain 0
    assert_eq!(cpu.regs[0], 0);
    // mepc should be updated with uimm (15)
    assert_eq!(cpu.csr.mepc, 15);
}

#[test]
fn test_csrrsi() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mstatus = 0b1100 (initial value)
    cpu.csr.mstatus = 0b1100;

    // CSRRSI x2, mstatus, 0b0011 (3)
    // opcode: 0b1110011 (73)
    // funct3: 0b110
    // rd: x2 (2)
    // uimm: 3 (0x03)
    // csr: 0x300 (mstatus)
    // 0x300_03_6_10 -> 0x3001e173
    let inst = 0x3001e173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (12)
    assert_eq!(cpu.regs[2], 12);
    // mstatus should be 0b1100 | 0b0011 = 0b1111 (15)
    assert_eq!(cpu.csr.read(0x300).unwrap(), 15);
}

#[test]
fn test_csrrsi_x0_uimm() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mstatus = 0b1100 (initial value)
    cpu.csr.mstatus = 0b1100;

    // CSRRSI x2, mstatus, 0
    // uimm: 0
    // 0x300_00_6_10 -> 0x30006173
    let inst = 0x30006173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (12)
    assert_eq!(cpu.regs[2], 12);
    // mstatus should NOT be updated
    assert_eq!(cpu.csr.read(0x300).unwrap(), 12);
}

#[test]
fn test_csrrci() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mstatus = 0b1111 (initial value)
    cpu.csr.mstatus = 0b1111;

    // CSRRCI x2, mstatus, 0b1010 (10)
    // opcode: 0b1110011 (73)
    // funct3: 0b111
    // rd: x2 (2)
    // uimm: 10 (0x0a)
    // csr: 0x300 (mstatus)
    // 0x300_0a_7_10 -> 0x30057173 (誤り: 0x30057173 ではない)
    // 正確な計算:
    // csr [31:20] = 0x300
    // rs1/uimm [19:15] = 0x0a
    // funct3 [14:12] = 0x7
    // rd [11:7] = 0x02
    // opcode [6:0] = 0x73
    // 0x30057173 は rs1=10, rd=2, funct3=7, opcode=73, csr=0x300
    // 0x300_00000 | (0x0a << 15) | (0x07 << 12) | (0x02 << 7) | 0x73
    // 0x30000000 | 0x00050000 | 0x00007000 | 0x00000100 | 0x00000073 = 0x30057173
    let inst = 0x30057173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (15)
    assert_eq!(cpu.regs[2], 15);
    // mstatus should be 0b1111 & !0b1010 = 0b0101 (5)
    assert_eq!(cpu.csr.read(0x300).unwrap(), 5);
}

#[test]
fn test_csrrci_x0_uimm() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    
    // mstatus = 0b1111 (initial value)
    cpu.csr.mstatus = 0b1111;

    // CSRRCI x2, mstatus, 0
    // uimm: 0
    // funct3: 0b111 (7)
    // 0x300_00_7_10 -> 0x30007173
    let inst = 0x30007173;
    bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    // x2 should have the old mstatus value (15)
    assert_eq!(cpu.regs[2], 15);
    // mstatus should NOT be updated
    assert_eq!(cpu.csr.read(0x300).unwrap(), 15);
}
