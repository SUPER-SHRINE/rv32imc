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

#[test]
fn test_c_addi() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x10 = 100
    cpu.regs[10] = 100;

    // c.addi x10, 10
    // quadrant: 01, funct3: 000
    // rd: x10 (01010)
    // imm: 10 (001010)
    // imm[5]: 0, imm[4:0]: 01010
    // inst: 000 0 01010 01010 01 -> 0x0529
    let inst = 0x0529;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 110);
    assert_eq!(cpu.pc, 0x2);

    // c.addi x10, -1
    // imm: -1 (111111)
    // imm[5]: 1 (bit 12), imm[4:0]: 11111 (bits 6:2)
    // inst: 000 1 11111 01010 01 -> 0b000 1 01010 11111 01
    // rd: 10 (01010)
    // ビット 15:13 = 000
    // ビット 12 = 1 (imm[5])
    // ビット 11:7 = 01010 (rd)
    // ビット 6:2 = 11111 (imm[4:0])
    // ビット 1:0 = 01
    // 000 1 01010 11111 01 -> 0b0001010101111101 -> 0x157d
    let inst = 0x157d;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 109);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_c_li() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // c.li x10, 10
    // quadrant: 01, funct3: 010
    // rd: x10 (01010)
    // imm: 10 (001010)
    // imm[5]: 0, imm[4:0]: 01010
    // inst: 010 0 01010 01010 01 -> 0b0100010100101001 -> 0x4529
    let inst = 0x4529;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 10);
    assert_eq!(cpu.pc, 0x2);

    // c.li x11, -1
    // imm: -1 (111111)
    // imm[5]: 1, imm[4:0]: 11111
    // rd: x11 (01011)
    // inst: 010 1 01011 11111 01 -> 0b0101010111111101 -> 0x55fd
    let inst = 0x55fd;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[11], 0xffff_ffff);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_c_li_reserved() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // mtvec = 0x100
    cpu.csr.mtvec = 0x100;

    // c.li x0, 10
    // quadrant: 01, funct3: 010
    // rd: x0 (00000)
    // imm: 10 (001010)
    // inst: 010 0 00000 01010 01 -> 0b0100000000101001 -> 0x4029
    let inst = 0x4029;
    bus.write_inst16(0x0, inst);

    let result = cpu.step(&mut bus);
    
    // Should trap with exception code 2 (Illegal Instruction)
    match result {
        crate::cpu::StepResult::Trap(code) => assert_eq!(code, 2),
        _ => panic!("Should trap"),
    }
    assert_eq!(cpu.pc, 0x100);
    assert_eq!(cpu.csr.mcause, 2);
    assert_eq!(cpu.csr.mepc, 0x0);
}

#[test]
fn test_c_addi16sp() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    cpu.regs[2] = 1024; // sp = 1024

    // c.addi16sp sp, 16
    // quadrant: 01, funct3: 011
    // rd: x2 (00010)
    // imm: 16 (0000010000) -> 16の倍数なので imm=1 (0b000001)
    // imm[9|4|6|8:7|5]
    // 9: 0, 8: 0, 7: 0, 6: 0, 5: 0, 4: 1
    // inst[12] = imm[9] = 0
    // inst[6] = imm[4] = 1
    // inst[5] = imm[6] = 0
    // inst[4:3] = imm[8:7] = 00
    // inst[2] = imm[5] = 0
    // inst: 011 0 00010 10000 01 -> 0b0110000101000001 -> 0x6141
    let inst = 0x6141;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 1024 + 16);
    assert_eq!(cpu.pc, 0x2);

    // c.addi16sp sp, -16
    // imm: -16 (1111110000) -> imm = -1 (0b111111)
    // 9: 1, 8: 1, 7: 1, 6: 1, 5: 1, 4: 1
    // inst[12] = imm[9] = 1
    // inst[6] = imm[4] = 1
    // inst[5] = imm[6] = 1
    // inst[4:3] = imm[8:7] = 11
    // inst[2] = imm[5] = 1
    // inst: 011 1 00010 11111 01 -> 0b0111000101111101 -> 0x717d
    let inst = 0x717d;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 1024);
    assert_eq!(cpu.pc, 0x4);

    // c.addi16sp sp, 496 (max positive)
    // imm: 496 = 0b0111110000 -> imm[9:4] = 011111
    // 9: 0, 8: 1, 7: 1, 6: 1, 5: 1, 4: 1
    // inst[12] = 0, inst[6] = 1, inst[5] = 1, inst[4:3] = 11, inst[2] = 1
    // inst: 011 0 00010 11111 01 -> 0b0110000101111101 -> 0x617d
    let inst = 0x617d;
    bus.write_inst16(0x4, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 1024 + 496);
    assert_eq!(cpu.pc, 0x6);

    // c.addi16sp sp, -512 (max negative)
    // imm: -512 = 0b1000000000 -> imm[9:4] = 100000
    // 9: 1, 8: 0, 7: 0, 6: 0, 5: 0, 4: 0
    // inst[12] = 1, inst[6] = 0, inst[5] = 0, inst[4:3] = 00, inst[2] = 0
    // inst: 011 1 00010 00000 01 -> 0b0111000100000001 -> 0x7101
    let inst = 0x7101;
    bus.write_inst16(0x6, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[2], 1024 + 496 - 512);
    assert_eq!(cpu.pc, 0x8);
}

#[test]
fn test_c_addi16sp_reserved() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();
    cpu.csr.mtvec = 0x100;

    // imm = 0 is reserved
    // inst: 011 0 00010 00000 01 -> 0b0110000100000001 -> 0x6101
    let inst = 0x6101;
    bus.write_inst16(0x0, inst);

    let result = cpu.step(&mut bus);
    match result {
        crate::cpu::StepResult::Trap(code) => assert_eq!(code, 2),
        _ => panic!("Should trap"),
    }
}

#[test]
fn test_c_lui() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // c.lui x10, 1
    // quadrant: 01, funct3: 011
    // rd: x10 (01010)
    // imm: 1 (000001)
    // imm[5]: 0, imm[4:0]: 00001
    // inst: 011 0 01010 00001 01 -> 0b0110010100000101 -> 0x6505
    let inst = 0x6505;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0x1000);
    assert_eq!(cpu.pc, 0x2);

    // c.lui x11, -1 (0b111111) -> imm = -1
    // rd: x11 (01011)
    // imm[5]: 1, imm[4:0]: 11111
    // inst: 011 1 01011 11111 01 -> 0b0111010111111101 -> 0x75fd
    let inst = 0x75fd;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[11], 0xffff_f000);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_c_lui_reserved() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    cpu.csr.mtvec = 0x100;

    // rd=0 is reserved
    // c.lui x0, 1
    // inst: 011 0 00000 00001 01 -> 0b0110000000000101 -> 0x6005
    let inst = 0x6005;
    bus.write_inst16(0x0, inst);

    let result = cpu.step(&mut bus);
    match result {
        crate::cpu::StepResult::Trap(code) => assert_eq!(code, 2),
        _ => panic!("Should trap"),
    }
}

#[test]
fn test_c_srli() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // rd' = x8 (s0), x8 に 0x0000_00F0 をセット
    cpu.regs[8] = 0x0000_00F0;

    // c.srli x8, 4
    // quadrant: 01, funct3: 100, funct2: 00
    // rd': x8-x15 のオフセット (x8 は 000)
    // shamt: 4 (000100)
    // shamt[5] (inst[12]): 0
    // shamt[4:0] (inst[6:2]): 00100
    // inst: 100 0 00 000 00100 01 -> 0b1000000000001001 -> 0x8011
    let inst = 0x8011;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0x0000_000F);
    assert_eq!(cpu.pc, 0x2);

    // c.srli x9, 1
    // rd' = x9 (s1) -> 001
    // shamt: 1 (000001)
    // cpu.regs[9] = 1
    cpu.regs[9] = 1;
    // inst: 100 0 00 001 00001 01 -> 0b1000000010000101 -> 0x8085
    let inst = 0x8085;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[9], 0);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_c_srli_hint_and_reserved() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();
    cpu.csr.mtvec = 0x100;

    // c.srli x8, 0 (HINT)
    // inst: 100 0 00 000 00000 01 -> 0x8001
    cpu.regs[8] = 0x1234;
    bus.write_inst16(0x0, 0x8001);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0x1234); // Should not change
    assert_eq!(cpu.pc, 0x2);

    // c.srli x8, 32 (shamt[5] = 1, Reserved for RV32C)
    // inst: 100 1 00 000 00000 01 -> 0x9001
    bus.write_inst16(0x2, 0x9001);
    let result = cpu.step(&mut bus);
    match result {
        crate::cpu::StepResult::Trap(code) => assert_eq!(code, 2),
        _ => panic!("Should trap for shamt[5]=1 in RV32C"),
    }
    assert_eq!(cpu.pc, 0x100);
}

#[test]
fn test_c_srai() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // rd' = x8 (s0), x8 に 0xFFFF_FF00 (-256) をセット
    cpu.regs[8] = 0xFFFF_FF00;

    // c.srai x8, 4
    // quadrant: 01, funct3: 100, funct2: 01
    // rd': x8-x15 のオフセット (x8 は 000)
    // shamt: 4 (000100)
    // shamt[5] (inst[12]): 0
    // shamt[4:0] (inst[6:2]): 00100
    // inst: 100 0 01 000 00100 01 -> 0b1000010000001001 -> 0x8411
    let inst = 0x8411;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    // 0xFFFF_FF00 >>s 4 = 0xFFFF_FFF0
    assert_eq!(cpu.regs[8], 0xFFFF_FFF0);
    assert_eq!(cpu.pc, 0x2);

    // 正の数の場合も確認
    cpu.regs[9] = 0x0000_00F0;
    // c.srai x9, 1
    // rd': x9 (001)
    // shamt: 1 (00001)
    // inst: 100 0 01 001 00001 01 -> 0b1000010010000101 -> 0x8485
    let inst = 0x8485;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[9], 0x0000_0078);
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_c_srai_hint_and_reserved() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();
    cpu.csr.mtvec = 0x100;

    // c.srai x8, 0 (HINT)
    // inst: 100 0 01 000 00000 01 -> 0x8401
    cpu.regs[8] = 0x1234;
    bus.write_inst16(0x0, 0x8401);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0x1234); // Should not change
    assert_eq!(cpu.pc, 0x2);

    // c.srai x8, 32 (shamt[5] = 1, Reserved for RV32C)
    // inst: 100 1 01 000 00000 01 -> 0x9401
    bus.write_inst16(0x2, 0x9401);
    let result = cpu.step(&mut bus);
    match result {
        crate::cpu::StepResult::Trap(code) => assert_eq!(code, 2),
        _ => panic!("Should trap for shamt[5]=1 in RV32C"),
    }
    assert_eq!(cpu.pc, 0x100);
}

#[test]
fn test_c_andi() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // rd' = x8 (s0), x8 に 0b1111 (15) をセット
    cpu.regs[8] = 0b1111;

    // c.andi x8, 1
    // quadrant: 01, funct3: 100, funct2: 10
    // rd': x8 (000)
    // imm: 1 (000001)
    // imm[5] (inst[12]): 0
    // imm[4:0] (inst[6:2]): 00001
    // inst: 100 0 10 000 00001 01 -> 0b1000100000000101 -> 0x8805
    let inst = 0x8805;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 1);
    assert_eq!(cpu.pc, 0x2);

    // 負の即値の場合 (imm = -1)
    cpu.regs[9] = 0xAAAA_AAAA;
    // c.andi x9, -1
    // rd': x9 (001)
    // imm: -1 (111111)
    // imm[5]: 1
    // imm[4:0]: 11111
    // inst: 100 1 10 001 11111 01 -> 0b1001100011111101 -> 0x98FD
    let inst = 0x98FD;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[9], 0xAAAA_AAAA);
    assert_eq!(cpu.pc, 0x4);

    // imm = -32 (最小値)
    cpu.regs[10] = 0xFFFF_FFFF;
    // rd': x10 (010)
    // imm: -32 (100000)
    // imm[5]: 1
    // imm[4:0]: 00000
    // inst: 100 1 10 010 00000 01 -> 0b1001100100000001 -> 0x9901
    let inst = 0x9901;
    bus.write_inst16(0x4, inst);

    cpu.step(&mut bus);
    // 0xFFFF_FFFF & 0xFFFF_FFE0 = 0xFFFF_FFE0
    assert_eq!(cpu.regs[10], 0xFFFF_FFE0);
    assert_eq!(cpu.pc, 0x6);
}

#[test]
fn test_c_sub() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x8 (s0) = 100, x9 (s1) = 30
    cpu.regs[8] = 100;
    cpu.regs[9] = 30;

    // c.sub x8, x9
    // quadrant: 01, funct6: 100011, funct2: 00
    // rd': x8 (000), rs2': x9 (001)
    // inst: 100011 000 00 001 01 -> 0b1000110000000101 -> 0x8c05
    let inst = 0x8c05;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 70);
    assert_eq!(cpu.pc, 0x2);

    // 負の結果になる場合
    // x10 (a0) = 10, x11 (a1) = 20
    cpu.regs[10] = 10;
    cpu.regs[11] = 20;

    // c.sub x10, x11
    // rd': x10 (010), rs2': x11 (011)
    // inst: 100011 010 00 011 01 -> 0b1000110100001101 -> 0x8d0d
    let inst = 0x8d0d;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xffff_fff6); // -10
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_c_xor() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // x8 (s0) = 0b1010, x9 (s1) = 0b1100
    cpu.regs[8] = 0b1010;
    cpu.regs[9] = 0b1100;

    // c.xor x8, x9
    // quadrant: 01, funct6: 100011, funct2: 01
    // rd': x8 (000), rs2': x9 (001)
    // inst: 100011 000 01 001 01 -> 0b1000110000100101 -> 0x8c25
    let inst = 0x8c25;
    bus.write_inst16(0x0, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[8], 0b0110);
    assert_eq!(cpu.pc, 0x2);

    // x10 (a0) = 0xFFFF_FFFF, x11 (a1) = 0x1234_5678
    cpu.regs[10] = 0xFFFF_FFFF;
    cpu.regs[11] = 0x1234_5678;

    // c.xor x10, x11
    // rd': x10 (010), rs2': x11 (011)
    // inst: 100011 010 01 011 01 -> 0b1000110100101101 -> 0x8d2d
    let inst = 0x8d2d;
    bus.write_inst16(0x2, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xEDCB_A987);
    assert_eq!(cpu.pc, 0x4);
}
