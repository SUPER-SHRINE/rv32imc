use crate::cpu::Cpu;
use crate::cpu::tests::mock_bus::MockBus;

#[test]
fn test_c_addi4spn_min() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // sp (x2) = 100
    cpu.regs[2] = 100;

    // c.addi4spn x8, 4 (0x0060)
    // funct3: 000, nzuimm[9:2]: imm[5:4|9:6|2|3]
    // imm=4 (0b0000000100) -> imm[2]=1, others=0
    // nzuimm[9:2] bits: imm[5:4]=00, imm[9:6]=0000, imm[2]=1, imm[3]=0
    // ビット 12-5: 00 0000 1 0 -> 0b00000010 = 0x02
    // inst: 000 00000010 000 00 -> 0x0040
    let inst = 0x0040;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 104);
    assert_eq!(cpu.pc, 0x2);
}

#[test]
fn test_c_addi4spn_max() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // sp (x2) = 1000
    cpu.regs[2] = 1000;

    // c.addi4spn x15, 1020 (0x1ffc)
    // imm=1020 (0b1111111100) -> all bits 1
    // nzuimm[9:2] bits: imm[5:4]=11, imm[9:6]=1111, imm[2]=1, imm[3]=1
    // ビット 12-5: 11 1111 1 1 -> 0b11111111 = 0xff
    // inst: 000 11111111 111 00 -> 0x1ffc
    let inst = 0x1ffc;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[15], 2020);
    assert_eq!(cpu.pc, 0x2);
}

#[test]
fn test_c_addi4spn_various_imm() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // sp (x2) = 0
    cpu.regs[2] = 0;

    // nzuimm = 404 (0b0110010100)
    // imm[5:4] = 01 (1)
    // imm[9:6] = 0110 (6)
    // imm[2] = 1
    // imm[3] = 0
    // ビット 12-5: 01 0110 1 0 -> 0b01011010 = 0x5a
    // rd' = 010 (x10)
    // inst: 000 01011010 010 00 -> 0x0b48
    let inst = 0x0b48;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 404);
    assert_eq!(cpu.pc, 0x2);
}
