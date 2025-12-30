#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

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

#[test]
fn test_mulh() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0x7fffffff (max i32), x2 = 0x7fffffff
    // 0x7fffffff * 0x7fffffff = 0x3fffffff_00000001
    // Upper 32 bits = 0x3fffffff
    cpu.regs[1] = 0x7fffffff;
    cpu.regs[2] = 0x7fffffff;

    // mulh x3, x1, x2 (0x022091b3)
    // opcode: 0110011, rd: 3, funct3: 001, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x022091b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x3fffffff);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mulh_negative() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -2 (0xfffffffe), x2 = -2 (0xfffffffe)
    // -2 * -2 = 4 (0x00000000_00000004)
    // Upper 32 bits = 0x00000000
    cpu.regs[1] = 0xfffffffe;
    cpu.regs[2] = 0xfffffffe;

    // mulh x3, x1, x2 (0x022091b3)
    let inst = 0x022091b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x0);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mulh_mixed() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -1 (0xffffffff), x2 = 1 (0x00000001)
    // -1 * 1 = -1 (0xffffffff_ffffffff)
    // Upper 32 bits = 0xffffffff
    cpu.regs[1] = 0xffffffff;
    cpu.regs[2] = 0x00000001;

    // mulh x3, x1, x2 (0x022091b3)
    let inst = 0x022091b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffff);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mulhsu() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -1 (signed), x2 = 1 (unsigned)
    // -1 * 1 = -1 (64-bit: 0xffffffff_ffffffff)
    // Upper 32 bits = 0xffffffff
    cpu.regs[1] = 0xffffffff;
    cpu.regs[2] = 0x00000001;

    // mulhsu x3, x1, x2 (0x0220a1b3)
    // opcode: 0110011, rd: 3, funct3: 010, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x0220a1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffff);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mulhsu_positive() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 1 (signed), x2 = 0xffffffff (unsigned)
    // 1 * 0xffffffff = 0x00000000_ffffffff
    // Upper 32 bits = 0x00000000
    cpu.regs[1] = 0x00000001;
    cpu.regs[2] = 0xffffffff;

    // mulhsu x3, x1, x2 (0x0220a1b3)
    let inst = 0x0220a1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x00000000);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mulhu() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0xffffffff (4294967295), x2 = 0xffffffff (4294967295)
    // 0xffffffff * 0xffffffff = 0xfffffffe_00000001
    // Upper 32 bits = 0xfffffffe
    cpu.regs[1] = 0xffffffff;
    cpu.regs[2] = 0xffffffff;

    // mulhu x3, x1, x2 (0x0220b1b3)
    // opcode: 0110011, rd: 3, funct3: 011, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x0220b1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xfffffffe);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_mulhsu_both_negative_interpretation() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -1 (signed), x2 = 0xffffffff (unsigned)
    // -1 * 0xffffffff = -4294967295
    // -4294967295 in 64-bit signed:
    // 4294967295 is 0x00000000_ffffffff
    // -4294967295 is 0xffffffff_00000001
    // Upper 32 bits = 0xffffffff
    cpu.regs[1] = 0xffffffff; // -1
    cpu.regs[2] = 0xffffffff; // 4294967295

    // mulhsu x3, x1, x2 (0x0220a1b3)
    let inst = 0x0220a1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffff);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_div() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 20, x2 = 3
    cpu.regs[1] = 20;
    cpu.regs[2] = 3;

    // div x3, x1, x2 (0x0220c1b3)
    // opcode: 0110011, rd: 3, funct3: 100, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x0220c1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 6);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_div_negative() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -20 (0xffffffec), x2 = 3
    cpu.regs[1] = 0xffffffec;
    cpu.regs[2] = 3;

    // div x3, x1, x2
    let inst = 0x0220c1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xfffffffa); // -6
}

#[test]
fn test_div_by_zero() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    cpu.regs[1] = 20;
    cpu.regs[2] = 0;

    // div x3, x1, x2
    let inst = 0x0220c1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffff); // -1
}

#[test]
fn test_div_overflow() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -2^31 (0x80000000), x2 = -1 (0xffffffff)
    cpu.regs[1] = 0x80000000;
    cpu.regs[2] = 0xffffffff;

    // div x3, x1, x2
    let inst = 0x0220c1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x80000000);
}

#[test]
fn test_divu() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 20, x2 = 3
    cpu.regs[1] = 20;
    cpu.regs[2] = 3;

    // divu x3, x1, x2 (0x0220d1b3)
    // opcode: 0110011, rd: 3, funct3: 101, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x0220d1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 6);
}

#[test]
fn test_divu_by_zero() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    cpu.regs[1] = 20;
    cpu.regs[2] = 0;

    // divu x3, x1, x2
    let inst = 0x0220d1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xffffffff); // 2^32 - 1
}

#[test]
fn test_rem() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 20, x2 = 3
    cpu.regs[1] = 20;
    cpu.regs[2] = 3;

    // rem x3, x1, x2 (0x0220e1b3)
    // opcode: 0110011, rd: 3, funct3: 110, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x0220e1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 2);
}

#[test]
fn test_rem_negative() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = -20 (0xffffffec), x2 = 3
    cpu.regs[1] = 0xffffffec;
    cpu.regs[2] = 3;

    // rem x3, x1, x2
    let inst = 0x0220e1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xfffffffe); // -2
}

#[test]
fn test_rem_by_zero() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    cpu.regs[1] = 20;
    cpu.regs[2] = 0;

    // rem x3, x1, x2
    let inst = 0x0220e1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 20); // dividend
}

#[test]
fn test_remu() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 20, x2 = 3
    cpu.regs[1] = 20;
    cpu.regs[2] = 3;

    // remu x3, x1, x2 (0x0220f1b3)
    // opcode: 0110011, rd: 3, funct3: 111, rs1: 1, rs2: 2, funct7: 0000001
    let inst = 0x0220f1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 2);
}

#[test]
fn test_remu_by_zero() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    cpu.regs[1] = 20;
    cpu.regs[2] = 0;

    // remu x3, x1, x2
    let inst = 0x0220f1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 20); // dividend
}
