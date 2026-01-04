#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

#[test]
fn test_fence() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // FENCE instruction (fm=0, pred=W, succ=R, rs1=0, rd=0)
    // opcode: 0001111, funct3: 000
    // 0000 1000 0100 00000 000 00000 0001111
    // pred: 1000 (W), succ: 0100 (R)
    // 0x0840000f
    let inst = 0x0840000f;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_fence_i() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // FENCE.I instruction (imm=0, rs1=0, rd=0)
    // opcode: 0001111, funct3: 001
    // 000000000000 00000 001 00000 0001111
    // 0x0000100f
    let inst = 0x0000100f;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0, inst);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x4);
}
