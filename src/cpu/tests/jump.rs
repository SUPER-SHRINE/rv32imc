use crate::cpu::Cpu;
use super::MockBus;

// jal 命令によって pc + 4 がレジスタに設定され、PC がジャンプすることを確認
#[test]
fn test_jal() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // JAL x1, 0x100 (imm=0x100, rd=1, opcode=1101111)
    // imm[20]=0, imm[10:1]=0x80, imm[11]=0, imm[19:12]=0
    // inst[31]=0, inst[30:21]=0x80, inst[20]=0, inst[19:12]=0
    // inst = 0x00000000 | (0x80 << 21) | (0 << 20) | (0 << 12) | (1 << 7) | 0x6f
    //      = 0x100000ef
    let inst_bin = 0x100000ef;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1004);
    assert_eq!(cpu.pc, 0x1100);
}

// jal 命令で x0 レジスタが指定された場合、戻り先アドレスが保存されないことを確認
#[test]
fn test_jal_x0() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // JAL x0, 0x100
    let inst_bin = 0x1000006f;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1100);
}

// jal 命令で負のオフセットを指定した場合の動作を確認
#[test]
fn test_jal_neg() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // JAL x1, -0x100
    // imm = -0x100 = 0xffffff00
    // imm[20:1] = 0x1ff00 >> 1 = 0xff80
    // imm[20] = 1 (inst[31])
    // imm[10:1] = 0x380 (inst[30:21])
    // imm[11] = 1 (inst[20])
    // imm[19:12] = 0xff (inst[19:12])
    // inst = (1 << 31) | (0x380 << 21) | (1 << 20) | (0xff << 12) | (1 << 7) | 0x6f
    //      = 0xf00ff0ef (Corrected: inst[30:21] is 0x380, so 0x380 << 21 = 0x70000000)
    // 0x80000000 | 0x70000000 | 0x00100000 | 0x000ff000 | 0x00000080 | 0x6f
    // = 0xf01ff0ef
    let inst_bin = 0xf01ff0ef;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1004);
    assert_eq!(cpu.pc, 0x0f00);
}

// jalr 命令によってターゲットアドレスへジャンプし、pc + 4 がレジスタに設定されることを確認
#[test]
fn test_jalr() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[2] = 0x2000;
    // JALR x1, 0x10(x2) (imm=0x10, rs1=2, funct3=0, rd=1, opcode=1100111)
    // inst = 0x010100e7
    let inst_bin = 0x010100e7;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1004);
    assert_eq!(cpu.pc, 0x2010);
}

// jalr 命令でターゲットアドレスの最下位ビットがクリアされることを確認
#[test]
fn test_jalr_align() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    cpu.regs[2] = 0x2001;
    // JALR x0, 0x11(x2) (imm=0x11, rs1=2, funct3=0, rd=0, opcode=1100111)
    // 0x2001 + 0x11 = 0x2012. 0x2012 & ~1 = 0x2012
    // inst = 0x01110067
    let inst_bin = 0x01110067;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x2012);

    // 最下位ビットがセットされるケース
    cpu.pc = 0x1004;
    cpu.regs[2] = 0x2000;
    // JALR x0, 0x11(x2) -> 0x2011 & ~1 = 0x2010
    let inst_bin = 0x01110067;
    bus.write_inst32(0x1004, inst_bin);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x2010);
}
