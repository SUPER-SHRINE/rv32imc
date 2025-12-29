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

#[test]
fn test_sll() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0x00000001, x2 = 5
    cpu.regs[1] = 0x00000001;
    cpu.regs[2] = 5;

    // sll x3, x1, x2 (0x002091b3)
    // opcode: 0110011, rd: 3, funct3: 001, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x002091b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 1 << 5);
    assert_eq!(cpu.pc, 0x4);

    // テスト2: シフト量の下位5ビットのみが使用されることを確認
    // x1 = 0x00000001, x2 = 32 (0x20) -> shamt = 0
    cpu.regs[2] = 32;
    bus.write_inst32(0x4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x00000001);
}

#[test]
fn test_slt() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 10, x2 = 20 (x1 < x2)
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;

    // slt x3, x1, x2 (0x0020a1b3)
    // opcode: 0110011, rd: 3, funct3: 010, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x0020a1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 1);
    assert_eq!(cpu.pc, 0x4);

    // x1 = 20, x2 = 10 (x1 > x2)
    cpu.regs[1] = 20;
    cpu.regs[2] = 10;
    bus.write_inst32(0x4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0);

    // x1 = -10 (0xfffffff6), x2 = 10 (-10 < 10)
    cpu.regs[1] = 0xfffffff6;
    cpu.regs[2] = 10;
    bus.write_inst32(0x8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 1);

    // x1 = 10, x2 = -10 (10 > -10)
    cpu.regs[1] = 10;
    cpu.regs[2] = 0xfffffff6;
    bus.write_inst32(0xc, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0);
}

#[test]
fn test_sltu() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 10, x2 = 20 (x1 < x2)
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;

    // sltu x3, x1, x2 (0x0020b1b3)
    // opcode: 0110011, rd: 3, funct3: 011, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x0020b1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 1);
    assert_eq!(cpu.pc, 0x4);

    // x1 = 20, x2 = 10 (x1 > x2)
    cpu.regs[1] = 20;
    cpu.regs[2] = 10;
    bus.write_inst32(0x4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0);

    // x1 = 0xffffffff, x2 = 10 (unsigned: 0xffffffff > 10)
    cpu.regs[1] = 0xffffffff;
    cpu.regs[2] = 10;
    bus.write_inst32(0x8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0);

    // x1 = 10, x2 = 0xffffffff (unsigned: 10 < 0xffffffff)
    cpu.regs[1] = 10;
    cpu.regs[2] = 0xffffffff;
    bus.write_inst32(0xc, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 1);
}

#[test]
fn test_xor() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0b1010, x2 = 0b1100
    cpu.regs[1] = 0b1010;
    cpu.regs[2] = 0b1100;

    // xor x3, x1, x2 (0x0020c1b3)
    // opcode: 0110011, rd: 3, funct3: 100, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x0020c1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0b0110);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_srl() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0x80000000, x2 = 1
    cpu.regs[1] = 0x80000000;
    cpu.regs[2] = 1;

    // srl x3, x1, x2 (0x0020d1b3)
    // opcode: 0110011, rd: 3, funct3: 101, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x0020d1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x40000000);
    assert_eq!(cpu.pc, 0x4);

    // テスト2: シフト量の下位5ビットのみが使用されることを確認
    // x1 = 0x80000000, x2 = 33 (0x21) -> shamt = 1
    cpu.regs[1] = 0x80000000;
    cpu.regs[2] = 33;
    bus.write_inst32(0x4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x40000000);
}

#[test]
fn test_sra() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0x80000000, x2 = 1
    cpu.regs[1] = 0x80000000;
    cpu.regs[2] = 1;

    // sra x3, x1, x2 (0x4020d1b3)
    // opcode: 0110011, rd: 3, funct3: 101, rs1: 1, rs2: 2, funct7: 0100000
    let inst = 0x4020d1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xc0000000);
    assert_eq!(cpu.pc, 0x4);

    // テスト2: 正の数に対する算術シフト (0x40000000 >> 1 = 0x20000000)
    cpu.regs[1] = 0x40000000;
    cpu.regs[2] = 1;
    bus.write_inst32(0x4, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0x20000000);

    // テスト3: シフト量の下位5ビットのみが使用される
    cpu.regs[1] = 0x80000000;
    cpu.regs[2] = 33; // shamt = 1
    bus.write_inst32(0x8, inst);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0xc0000000);
}

#[test]
fn test_or() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x1 = 0b1010, x2 = 0b1100
    cpu.regs[1] = 0b1010;
    cpu.regs[2] = 0b1100;

    // or x3, x1, x2 (0x0020e1b3)
    // opcode: 0110011, rd: 3, funct3: 110, rs1: 1, rs2: 2, funct7: 0000000
    let inst = 0x0020e1b3;
    bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[3], 0b1110);
    assert_eq!(cpu.pc, 0x4);
}
