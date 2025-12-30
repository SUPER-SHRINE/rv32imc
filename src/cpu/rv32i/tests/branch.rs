#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

// beq 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_beq() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[1] = 10;
    cpu.regs[2] = 10;
    // BEQ x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=0, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (0 << 12) | (0 << 7) | 0x63
    //      = 0x10208063
    let inst_bin = 0x10208063;
    bus.write_inst32(0x1000, inst_bin);

    // 条件一致
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 条件不一致
    cpu.pc = 0x1000;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// beq 命令で負のオフセットを指定した場合の動作を確認
#[test]
fn test_beq_neg() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[1] = 10;
    cpu.regs[2] = 10;
    // BEQ x1, x2, -0x100 (imm=-0x100 = 0xffffff00)
    // imm[12]=1, imm[11]=1, imm[10:5]=0x38, imm[4:1]=0
    // inst[31]=1, inst[7]=1, inst[30:25]=0x38, inst[11:8]=0
    // inst = (1 << 31) | (0x38 << 25) | (2 << 20) | (1 << 15) | (0 << 12) | (1 << 7) | 0x63
    //      = 0xf02080e3
    let inst_bin = 0xf02080e3;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x0f00);
}

// bne 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bne() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    // BNE x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=1, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (1 << 12) | (0 << 7) | 0x63
    //      = 0x10209063
    let inst_bin = 0x10209063;
    bus.write_inst32(0x1000, inst_bin);

    // 条件一致 (10 != 20)
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 条件不一致 (10 != 10)
    cpu.pc = 0x1000;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// blt 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_blt() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 < rs2 (10 < 20) -> ジャンプ
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    // BLT x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=4, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (4 << 12) | (0 << 7) | 0x63
    //      = 0x1020c063
    let inst_bin = 0x1020c063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件不一致: rs1 == rs2 (20 == 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 3. 条件不一致: rs1 > rs2 (30 > 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 30;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号付き比較の確認: rs1 < rs2 (-10 < 10) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号付き比較の確認: rs1 > rs2 (10 > -10) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// bge 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bge() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 > rs2 (20 > 10) -> ジャンプ
    cpu.regs[1] = 20;
    cpu.regs[2] = 10;
    // BGE x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=5, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (5 << 12) | (0 << 7) | 0x63
    //      = 0x1020d063
    let inst_bin = 0x1020d063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件一致: rs1 == rs2 (20 == 20) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 3. 条件不一致: rs1 < rs2 (10 < 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号付き比較の確認: rs1 > rs2 (10 > -10) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号付き比較の確認: rs1 < rs2 (-10 < 10) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// bltu 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bltu() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 < rs2 (10 < 20) -> ジャンプ
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    // BLTU x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=6, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (6 << 12) | (0 << 7) | 0x63
    //      = 0x1020e063
    let inst_bin = 0x1020e063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件不一致: rs1 == rs2 (20 == 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 3. 条件不一致: rs1 > rs2 (30 > 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 30;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号なし比較の確認: rs1 < rs2 (10 < -10 as u32) -> ジャンプ
    // -10i32 as u32 は 0xfffffff6 なので、10 (0x0000000a) より大きい
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号なし比較の確認: rs1 > rs2 (-10 as u32 > 10) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}

// bgeu 命令によって条件一致時にジャンプし、条件不一致時に PC + 4 進むことを確認
#[test]
fn test_bgeu() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // 1. 条件一致: rs1 > rs2 (20 > 10) -> ジャンプ
    cpu.regs[1] = 20;
    cpu.regs[2] = 10;
    // BGEU x1, x2, 0x100 (imm=0x100, rs1=1, rs2=2, funct3=7, opcode=1100011)
    // imm[12]=0, imm[11]=0, imm[10:5]=0x08, imm[4:1]=0
    // inst[31]=0, inst[7]=0, inst[30:25]=0x08, inst[11:8]=0
    // inst = (0 << 31) | (0x08 << 25) | (2 << 20) | (1 << 15) | (7 << 12) | (0 << 7) | 0x63
    //      = 0x1020f063
    let inst_bin = 0x1020f063;
    bus.write_inst32(0x1000, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 2. 条件一致: rs1 == rs2 (20 == 20) -> ジャンプ
    cpu.pc = 0x1000;
    cpu.regs[1] = 20;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 3. 条件不一致: rs1 < rs2 (10 < 20) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = 20;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);

    // 4. 符号なし比較の確認: rs1 > rs2 (-10 as u32 > 10) -> ジャンプ
    // -10i32 as u32 は 0xfffffff6 なので、10 (0x0000000a) より大きい
    cpu.pc = 0x1000;
    cpu.regs[1] = -10i32 as u32;
    cpu.regs[2] = 10;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1100);

    // 5. 符号なし比較の確認: rs1 < rs2 (10 < -10 as u32) -> PC + 4
    cpu.pc = 0x1000;
    cpu.regs[1] = 10;
    cpu.regs[2] = -10i32 as u32;
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1004);
}
