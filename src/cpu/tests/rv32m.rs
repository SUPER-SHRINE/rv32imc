use crate::cpu::Cpu;
use crate::cpu::tests::MockBus;

#[test]
fn test_mul() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 10, x2 = 20
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;

    // mul x3, x1, x2 (0x022081b3)
    // opcode: 0110011, rd: 3, funct3: 000, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x022081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 200);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mul_negative() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -10 (0xfffffff6), x2 = 5
    cpu.regs[1] = 0xfffffff6;
    cpu.regs[2] = 5;

    // mul x3, x1, x2 (0x022081b3)
    let inst = 0x022081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffce); // -50
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mul_overflow() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0x7fffffff (max i32), x2 = 2
    cpu.regs[1] = 0x7fffffff;
    cpu.regs[2] = 2;

    // mul x3, x1, x2 (0x022081b3)
    let inst = 0x022081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    // 0x7fffffff * 2 = 0xfffffffe (下位32ビット)
    assert_eq!(cpu.regs[3], 0xfffffffe);
    assert_eq!(cpu.pc, 0x4);
}
