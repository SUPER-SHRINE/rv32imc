use crate::bus::default_bus::{DefaultBus, CLINT_BASE};
use crate::bus::Bus;
use crate::cpu::Cpu;

#[test]
fn test_clint_timer_interrupt() {
    let mut bus = DefaultBus::new(0x1000);
    let mut cpu = Cpu::new(0);

    // メモリに NOP 命令を配置
    for i in (0..0x100).step_by(4) {
        bus.write32(i, 0x00000013); // addi x0, x0, 0
    }

    // mie.MTIE (ビット7) をセット
    cpu.csr.write(0x304, 1 << 7).unwrap();
    // mstatus.MIE (ビット3) をセット
    cpu.csr.write(0x300, 1 << 3).unwrap();
    // mtvec をセット (0x100)
    cpu.csr.write(0x305, 0x100).unwrap();

    // mtimecmp を 10 に設定
    bus.write32(CLINT_BASE + 0x4000, 10);
    bus.write32(CLINT_BASE + 0x4004, 0);

    // 9ステップ実行 (mtime が 9 になる)
    for _ in 0..9 {
        cpu.step(&mut bus);
    }
    
    // CPU の PC を 0 に戻す
    cpu.pc = 0;
    
    // 次のステップで mtime が 10 になり、割り込みが発生するはず
    // mtime=10, mtimecmp=10 -> interrupt!
    cpu.step(&mut bus);

    // 割り込みハンドラ (mtvec = 0x100) に飛んでいるはず
    assert_eq!(cpu.pc, 0x100);
    // mcause が 0x80000007 (Machine Timer Interrupt) であること
    assert_eq!(cpu.csr.read(0x342), Ok(0x8000_0007));
}

#[test]
fn test_clint_software_interrupt() {
    let mut bus = DefaultBus::new(0x1000);
    let mut cpu = Cpu::new(0);

    // メモリに NOP 命令を配置
    bus.write32(0, 0x00000013);

    // mie.MSIE (ビット3) をセット
    cpu.csr.write(0x304, 1 << 3).unwrap();
    // mstatus.MIE (ビット3) をセット
    cpu.csr.write(0x300, 1 << 3).unwrap();
    // mtvec をセット (0x200)
    cpu.csr.write(0x305, 0x200).unwrap();

    // msip を 1 に設定
    bus.write32(CLINT_BASE + 0x0000, 1);

    // 次のステップで割り込みが発生するはず
    cpu.step(&mut bus);

    // 割り込みハンドラ (mtvec = 0x200) に飛んでいるはず
    assert_eq!(cpu.pc, 0x200);
    // mcause が 0x80000003 (Machine Software Interrupt) であること
    assert_eq!(cpu.csr.read(0x342), Ok(0x8000_0003));
}

#[test]
fn test_clint_mtime_increment() {
    let mut bus = DefaultBus::new(0x1000);
    let mut cpu = Cpu::new(0);

    assert_eq!(bus.clint.mtime, 0);
    cpu.step(&mut bus);
    assert_eq!(bus.clint.mtime, 1);
    cpu.step(&mut bus);
    assert_eq!(bus.clint.mtime, 2);
}
