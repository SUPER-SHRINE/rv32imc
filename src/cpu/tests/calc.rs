use crate::cpu::Cpu;
use crate::cpu::tests::MockBus;

#[test]
fn test_add() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 10, x2 = 20
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;

    // add x3, x1, x2 (0x002081b3)
    // opcode: 0110011, rd: 3, funct3: 000, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x002081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 30);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_add_negative() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -1 (0xffffffff), x2 = 1
    cpu.regs[1] = 0xffffffff;
    cpu.regs[2] = 1;

    // add x3, x1, x2 (0x002081b3)
    let inst = 0x002081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_sub() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 30, x2 = 10
    cpu.regs[1] = 30;
    cpu.regs[2] = 10;

    // sub x3, x1, x2 (0x402081b3)
    // opcode: 0110011, rd: 3, funct3: 000, rs1: 1, rs2: 2, funct7: 0100000
    let inst = 0x402081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 20);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_sub_negative_result() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 10, x2 = 30
    cpu.regs[1] = 10;
    cpu.regs[2] = 30;

    // sub x3, x1, x2 (0x402081b3)
    let inst = 0x402081b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffec); // -20
    assert_eq!(cpu.pc, 0x4);
}
