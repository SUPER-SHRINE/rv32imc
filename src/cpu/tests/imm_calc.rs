use crate::cpu::Cpu;
use super::MockBus;

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
