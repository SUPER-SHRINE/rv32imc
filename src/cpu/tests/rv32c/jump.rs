use crate::cpu::Cpu;
use crate::cpu::tests::mock_bus::MockBus;

#[test]
fn test_c_jal_positive() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.jal 100
    // PC = 0x200
    // ra = 0x200 + 2 = 0x202
    // PC = 0x200 + 100 = 0x264
    
    // imm = 100 (0b000001100100)
    // imm[11] = 0
    // imm[10] = 0
    // imm[9:8] = 00
    // imm[7] = 0
    // imm[6] = 1
    // imm[5] = 1
    // imm[4] = 0
    // imm[3:1] = 010
    
    // Bits for inst[15:0]:
    // 15:13 = 001 (funct3)
    // 12    = 0 (imm[11])
    // 11    = 0 (imm[4])
    // 10:9  = 00 (imm[9:8])
    // 8     = 0 (imm[10])
    // 7     = 1 (imm[6])
    // 6     = 0 (imm[7]) -- Wait, imm[7] is 0
    // 5:3   = 010 (imm[3:1])
    // 2     = 1 (imm[5])
    // 1:0   = 01 (quadrant)
    
    // Re-evaluating imm = 100 (0b0000_0110_0100)
    // Bit index: 11 10 9 8 7 6 5 4 3 2 1 0
    // Value:      0  0 0 0 0 1 1 0 0 1 0 0
    // imm[11]   = 0 (inst[12])
    // imm[10]   = 0 (inst[8])
    // imm[9:8]  = 00 (inst[10:9])
    // imm[7]    = 0 (inst[6])
    // imm[6]    = 1 (inst[7])
    // imm[5]    = 1 (inst[2])
    // imm[4]    = 0 (inst[11])
    // imm[3:1]  = 010 (inst[5:3])
    
    // inst[15:13] = 001
    // inst[12]    = 0
    // inst[11]    = 0
    // inst[10:9]  = 00
    // inst[8]     = 0
    // inst[7]     = 1
    // inst[6]     = 0
    // inst[5:3]   = 010
    // inst[2]     = 1
    // inst[1:0]   = 01
    
    // 0b001_0_0_00_0_1_0_010_1_01 = 0b0010000010010101 = 0x2095
    
    let inst = 0x2095;
    bus.write_inst16(0x200, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x200 + 100);
    assert_eq!(cpu.regs[1], 0x202);
}

#[test]
fn test_c_jal_negative() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.jal -100
    // PC = 0x200
    // ra = 0x200 + 2 = 0x202
    // PC = 0x200 - 100 = 0x19c
    
    // imm = -100 (12 bits)
    // 100 = 0b0000_0110_0100
    // !100 = 0b1111_1001_1011
    // -100 = 0b1111_1001_1100
    
    // Bit index: 11 10 9 8 7 6 5 4 3 2 1 0
    // Value:      1  1 1 1 1 0 0 1 1 1 0 0
    // imm[11]   = 1 (inst[12])
    // imm[10]   = 1 (inst[8])
    // imm[9:8]  = 11 (inst[10:9])
    // imm[7]    = 0 (inst[6])
    // imm[6]    = 0 (inst[7])
    // imm[5]    = 0 (inst[2])
    // imm[4]    = 1 (inst[11])
    // imm[3:1]  = 110 (inst[5:3])
    
    // inst[15:13] = 001
    // inst[12]    = 1
    // inst[11]    = 1
    // inst[10:9]  = 11
    // inst[8]     = 1
    // inst[7]     = 0
    // inst[6]     = 0
    // inst[5:3]   = 110
    // inst[2]     = 0
    // inst[1:0]   = 01
    
    // Bits in order:
    // 15 14 13 | 12 | 11 | 10 9 | 8 | 7 | 6 | 5 4 3 | 2 | 1 0
    //  0  0  1 |  1 |  1 |  1 1 | 1 | 0 | 0 | 1 1 0 | 0 | 0 1
    // 0b0011_1111_0011_0001 = 0x3f31
    // Wait, 0b0011_1111_0011_0001
    // Bits:
    // 15:13 = 001
    // 12    = 1
    // 11    = 1
    // 10:9  = 11
    // 8     = 1
    // 7     = 0 (inst[7] -> imm[6])
    // 6     = 0 (inst[6] -> imm[7])
    // 5:3   = 110
    // 2     = 0 (inst[2] -> imm[5])
    // 1:0   = 01
    // 001 1 1 11 1 0 0 110 0 01 -> 0011 1111 0011 0001 (Correct binary)
    
    // Let's re-check manual decoding of 0x3f31 with my current decode_cj_type
    // inst = 0x3f31 = 0b0011_1111_0011_0001
    // i11 = (inst >> 12) & 1 = 1
    // i10 = (inst >> 8) & 1 = 1
    // i9_8 = (inst >> 9) & 3 = 3
    // i7 = (inst >> 6) & 1 = 0
    // i6 = (inst >> 7) & 1 = 0
    // i5 = (inst >> 2) & 1 = 0
    // i4 = (inst >> 11) & 1 = 1
    // i3_1 = (inst >> 3) & 7 = 6
    
    // imm = (1<<11) | (1<<10) | (3<<8) | (0<<7) | (0<<6) | (0<<5) | (1<<4) | (6<<1)
    // imm = 0x800 | 0x400 | 0x300 | 0x0 | 0x0 | 0x0 | 0x10 | 0x0c
    // imm = 0xf00 | 0x1c = 0xf1c
    // sign_extend(0xf1c) = 0xffff_ff1c = -228
    
    // Target was -100 (0xf9c)
    // We need imm = 0xf9c (12 bits) = 0b1111_1001_1100
    // i11 = 1 (inst[12])
    // i10 = 1 (inst[8])
    // i9_8 = 3 (0b11) (inst[10:9])
    // i7 = 1 (inst[6])
    // i6 = 0 (inst[7])
    // i5 = 0 (inst[2])
    // i4 = 1 (inst[11])
    // i3_1 = 6 (0b110) (inst[5:3])
    
    // inst[15:13] = 001
    // inst[12] = 1
    // inst[11] = 1
    // inst[10:9] = 11
    // inst[8] = 1
    // inst[7] = 0 (i6)
    // inst[6] = 1 (i7)
    // inst[5:3] = 110
    // inst[2] = 0 (i5)
    // inst[1:0] = 01
    
    // 0b001 1 1 11 1 0 1 110 0 01 = 0b0011_1111_0111_0001 = 0x3f71
    
    let inst = 0x3f71;
    bus.write_inst16(0x200, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x200 - 100);
    assert_eq!(cpu.regs[1], 0x202);
}

#[test]
fn test_c_jal_max_forward() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // c.jal 2046
    // imm = 2046 (0b0111_1111_1110)
    // imm[11]   = 0 (inst[12])
    // imm[10]   = 1 (inst[8])
    // imm[9:8]  = 11 (inst[10:9])
    // imm[7]    = 1 (inst[6])
    // imm[6]    = 1 (inst[7])
    // imm[5]    = 1 (inst[2])
    // imm[4]    = 1 (inst[11])
    // imm[3:1]  = 111 (inst[5:3])
    
    // inst[15:13] = 001
    // inst[12]    = 0
    // inst[11]    = 1
    // inst[10:9]  = 11
    // inst[8]     = 1
    // inst[7]     = 1
    // inst[6]     = 1
    // inst[5:3]   = 111
    // inst[2]     = 1
    // inst[1:0]   = 01
    
    // 0b001_0_1_11_1_1_1_111_1_01 = 0b0010111111111101 = 0x2ffd
    
    let inst = 0x2ffd;
    bus.write_inst16(0x1000, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1000 + 2046);
    assert_eq!(cpu.regs[1], 0x1002);
}

#[test]
fn test_c_jal_max_backward() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // c.jal -2048
    // imm = -2048 (0b1000_0000_0000)
    // imm[11]   = 1 (inst[12])
    // imm[10]   = 0 (inst[8])
    // imm[9:8]  = 00 (inst[10:9])
    // imm[7]    = 0 (inst[6])
    // imm[6]    = 0 (inst[7])
    // imm[5]    = 0 (inst[2])
    // imm[4]    = 0 (inst[11])
    // imm[3:1]  = 000 (inst[5:3])
    
    // inst[15:13] = 001
    // inst[12]    = 1
    // inst[11]    = 0
    // inst[10:9]  = 00
    // inst[8]     = 0
    // inst[7]     = 0
    // inst[6]     = 0
    // inst[5:3]   = 000
    // inst[2]     = 0
    // inst[1:0]   = 01
    
    // 0b001_1_0_00_0_0_0_000_0_01 = 0b0011000000000001 = 0x3001
    
    let inst = 0x3001;
    bus.write_inst16(0x1000, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1000 - 2048);
    assert_eq!(cpu.regs[1], 0x1002);
}

#[test]
fn test_c_j_positive() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.j 100
    // PC = 0x200
    // PC = 0x200 + 100 = 0x264
    
    // c.jal 100 was 0x2095 (funct3 = 001)
    // c.j 100 has funct3 = 101
    // inst = 0x2095 | (0b101 << 13) & (0x101 << 13 is wrong since 001 was already there)
    // 0x2095 bits: 001 0 0 00 0 1 0 010 1 01
    // c.j 100 bits: 101 0 0 00 0 1 0 010 1 01 = 0xa095
    
    let inst = 0xa095;
    bus.write_inst16(0x200, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x200 + 100);
    assert_eq!(cpu.regs[1], 0); // x1 should not be changed
}

#[test]
fn test_c_j_negative() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.j -100
    // PC = 0x200
    // PC = 0x200 - 100 = 0x19c
    
    // c.jal -100 was 0x3f71 (funct3 = 001)
    // c.j -100 bits: 101 1 1 11 1 0 1 110 0 01 = 0xbf71
    
    let inst = 0xbf71;
    bus.write_inst16(0x200, inst);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x200 - 100);
    assert_eq!(cpu.regs[1], 0); // x1 should not be changed
}

#[test]
fn test_c_beqz_taken() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.beqz x8, 10
    // x8 (s0) is index 8 in cpu.regs.
    // Initial x8 is 0.
    // PC = 0x200
    // imm = 10 (0b0000_0101_0) -> imm[8:1] = 0000_0101
    // Bit structure: imm[8|4:3|7:6|2:1|5]
    // imm[8] = 0
    // imm[7:6] = 00
    // imm[5] = 0
    // imm[4:3] = 01
    // imm[2:1] = 01

    // inst bits:
    // 15:13 = 110 (funct3)
    // 12    = 0 (imm[8])
    // 11:10 = 11 (reserved for C.BEQZ/C.BNEZ)
    // 9:7   = 000 (rs1' = x8)
    // 6:5   = 00 (imm[7:6])
    // 4:3   = 01 (imm[4:3])
    // 2     = 0 (imm[5])
    // 1     = 01 (quadrant) -> Wait, 2:0 is imm[2:1] and then quadrant 01?
    // Let's check format again:
    // | 15 13 | 12 | 11 10 | 9 7 | 6 5 | 4 3 | 2 | 1 0 |
    // | 110   |i[8]| 11    |rs1' |i[7:6]|i[4:3]|i[5]| 01  |
    // Wait, the doc said: | 110 | imm[8] | 11 | rs1' | imm[4:3|7:6|2:1|5] | 01 |
    // Re-checking rv32c.md:
    // | 110 | imm[8] | 11 | rs1' | imm[4:3|7:6|2:1|5] | 01 |
    // Bits 6-2: imm[4:3|7:6|2:1|5] is NOT 5 bits total.
    // Ah, imm[4:3] is 2 bits, imm[7:6] is 2 bits, imm[2:1] is 2 bits? No.
    // Total bits for imm: 8, 7, 6, 5, 4, 3, 2, 1 (8 bits) + bit 0 is always 0.
    // Instruction bits 6-2 (5 bits) are:
    // 6:5 -> imm[7:6]
    // 4:3 -> imm[2:1]
    // 2   -> imm[5]
    // Wait, let's look at the standard:
    // C.BEQZ: funct3=110, imm[8|4:3|7:6|2:1|5], rs1'
    // inst[12] = imm[8]
    // inst[11:10] = 11
    // inst[9:7] = rs1'
    // inst[6:5] = imm[7:6]
    // inst[4:3] = imm[4:3] -- Wait, I see different versions. Let me check the official spec.
    // RISC-V Compressed Spec:
    // C.BEQZ (CB-type): imm[8|4:3|7:6|2:1|5]
    // inst[12] -> imm[8]
    // inst[11:10] -> 11
    // inst[9:7] -> rs1'
    // inst[6:5] -> imm[7:6]
    // inst[4:3] -> imm[2:1]
    // inst[2] -> imm[5]
    // wait, that's only 5 bits for 6:2.
    // inst[6:5] (2 bits)
    // inst[4:3] (2 bits)
    // inst[2] (1 bit)
    // Total 5 bits.
    // imm bits: 8 (1), 7:6 (2), 5 (1), 4:3 (2), 2:1 (2). Total 8 bits (+ 1 implied).
    // Let's re-verify:
    // inst[12]   -> imm[8]
    // inst[11:10] -> 11
    // inst[9:7]   -> rs1'
    // inst[6:5]   -> imm[7:6]
    // inst[4:3]   -> imm[2:1]
    // inst[2]     -> imm[5]
    // Wait, where are imm[4:3]?
    // Let me check again.
    // RV32 Compressed Instruction Formats:
    // CB Format: | funct3 | offset[8|4:3] | rs1' | offset[7:6|2:1|5] | op |
    // funct3 (15:13)
    // offset[8] (12)
    // offset[4:3] (11:10)
    // rs1' (9:7)
    // offset[7:6] (6:5)
    // offset[2:1] (4:3)
    // offset[5] (2)
    // op (1:0)
    
    // So for imm = 10 (0b0_0000_1010):
    // imm[8] = 0
    // imm[7:6] = 00
    // imm[5] = 0
    // imm[4:3] = 01
    // imm[2:1] = 01
    
    // inst bits:
    // 15:13 = 110
    // 12    = 0 (imm[8])
    // 11:10 = 01 (imm[4:3])
    // 9:7   = 000 (rs1' = x8)
    // 6:5   = 00 (imm[7:6])
    // 4:3   = 01 (imm[2:1])
    // 2     = 0 (imm[5])
    // 1:0   = 01
    
    // 0b110_0_01_000_00_01_0_01 = 0xc409
    
    let inst = 0xc409;
    bus.write_inst16(0x200, inst);
    cpu.regs[8] = 0;

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x200 + 10);
}

#[test]
fn test_c_beqz_not_taken() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.beqz x8, 10 (0xc409)
    let inst = 0xc409;
    bus.write_inst16(0x200, inst);
    cpu.regs[8] = 1;

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn test_c_beqz_negative() {
    let mut cpu = Cpu::new(0x200);
    let mut bus = MockBus::new();

    // c.beqz x8, -10
    // imm = -10 (9 bits)
    // 10 = 0b0_0000_1010
    // -10 = 0b1_1111_0110 (9 bits)
    // imm[8] = 1
    // imm[7:6] = 11
    // imm[5] = 1
    // imm[4:3] = 10
    // imm[2:1] = 11
    
    // inst bits:
    // 15:13 = 110
    // 12    = 1 (imm[8])
    // 11:10 = 10 (imm[4:3])
    // 9:7   = 000 (rs1' = x8)
    // 6:5   = 11 (imm[7:6])
    // 4:3   = 11 (imm[2:1])
    // 2     = 1 (imm[5])
    // 1:0   = 01
    
    // 0b110_1_10_000_11_11_1_01 = 0xd87d
    
    let inst = 0xd87d;
    bus.write_inst16(0x200, inst);
    cpu.regs[8] = 0;

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x200 - 10);
}
