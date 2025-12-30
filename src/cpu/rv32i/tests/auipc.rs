#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

// auipc 命令によって PC + 0x12345000 がレジスタに設定され、PC が +4 進むことを確認
#[test]
fn test_auipc() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // AUIPC x1, 0x12345 (imm=0x12345000, rd=1, opcode=0010111)
    // 0x12345000 | (1 << 7) | 0x17 = 0x12345097
    let inst_bin = 0x12345097;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[1], 0x1000 + 0x12345000);
    assert_eq!(cpu.pc, 0x1004);
}

// auipc 命令によって x0 レジスタの値が書き換わらず、PC が +4 進むことを確認
#[test]
fn test_auipc_x0() {
    let mut cpu = Cpu::new(0x1000);
    let mut bus = MockBus::new();

    // AUIPC x0, 0x12345
    let inst_bin = 0x12345017;
    bus.write_inst32(0x1000, inst_bin);

    cpu.step(&mut bus);

    assert_eq!(cpu.regs[0], 0);
    assert_eq!(cpu.pc, 0x1004);
}
