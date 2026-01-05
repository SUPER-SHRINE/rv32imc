#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

// lui 命令によってレジスタの値が正しく設定され、PC が +4 進むことを確認
#[test]
fn test_lui() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // LUI x1, 0x12345 (imm=0x12345000, rd=1, opcode=0110111)
    // 0x12345000 | (1 << 7) | 0x37 = 0x123450b7
    let inst_bin = 0x123450b7;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x12345000);
    assert_eq!(cpu.pc, 4);
}

// lui 命令によって x0 レジスタの値が書き換わらず、PC が +4 進むことを確認
#[test]
fn test_lui_x0() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // LUI x0, 0x12345
    let inst_bin = 0x12345037;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 4);
}
