use crate::cpu::Cpu;
use crate::bus::mock_bus::MockBus;
use crate::bus::Bus;

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
fn test_fence_i_self_modifying() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // 0x0: ADDI x1, x0, 10  (0x00a00093)
    // 0x4: FENCE.I          (0x0000100f)
    // 0x8: ADDI x1, x1, 1   (0x00108093) <- ここを後で書き換える
    
    bus.write_inst32(0x0, 0x00a00093);
    bus.write_inst32(0x4, 0x0000100f);
    bus.write_inst32(0x8, 0x00108093);

    // 最初に 0x0-0xb をフェッチしてキャッシュさせる
    cpu.step(&mut bus); // ADDI x1, x0, 10
    assert_eq!(cpu.regs[1], 10);
    
    // 実行中に 0x8 の命令を書き換える: ADDI x1, x1, 100 (0x06408093)
    // 現在の CPU はストア命令でキャッシュをフラッシュしなくなったので、
    // 0x8 の命令はキャッシュに残ったまま（ADDI x1, x1, 1）になるはず。
    bus.write32(0x8, 0x06408093);

    cpu.step(&mut bus); // FENCE.I (これがないと 0x8 は古い命令が実行されるはず)
    cpu.step(&mut bus); // 書き換え後の ADDI x1, x1, 100

    assert_eq!(cpu.regs[1], 110);
}

#[test]
fn test_no_fence_i_self_modifying_fails() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();

    // 0x0: ADDI x1, x0, 10  (0x00a00093)
    // 0x4: ADDI x0, x0, 0   (0x00000013) NOP
    // 0x8: ADDI x1, x1, 1   (0x00108093)
    
    bus.write_inst32(0x0, 0x00a00093);
    bus.write_inst32(0x4, 0x00000013);
    bus.write_inst32(0x8, 0x00108093);

    cpu.step(&mut bus); // ADDI x1, x0, 10
    
    // 0x8 の命令を書き換える: ADDI x1, x1, 100 (0x06408093)
    bus.write32(0x8, 0x06408093);

    cpu.step(&mut bus); // NOP
    cpu.step(&mut bus); // ADDI x1, x1, 1 (キャッシュされているので古い方が実行される)

    assert_eq!(cpu.regs[1], 11); // 110 ではなく 11 になることを確認
}
