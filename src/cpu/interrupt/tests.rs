use crate::cpu::{Cpu, StepResult};
use crate::bus::mock_bus::MockBus;
use crate::bus::plic::Plic;
use crate::bus::Bus;

struct InterruptTestBus {
    mock_bus: MockBus,
    plic: Plic,
}

impl InterruptTestBus {
    fn new() -> Self {
        Self {
            mock_bus: MockBus::new(),
            plic: Plic::new(),
        }
    }
}

impl Bus for InterruptTestBus {
    fn read8(&mut self, addr: u32) -> u8 { self.mock_bus.read8(addr) }
    fn read16(&mut self, addr: u32) -> u16 { self.mock_bus.read16(addr) }
    fn read32(&mut self, addr: u32) -> u32 { self.mock_bus.read32(addr) }
    fn write8(&mut self, addr: u32, val: u8) { self.mock_bus.write8(addr, val) }
    fn write16(&mut self, addr: u32, val: u16) { self.mock_bus.write16(addr, val) }
    fn write32(&mut self, addr: u32, val: u32) { self.mock_bus.write32(addr, val) }
    fn get_interrupt_level(&self) -> bool {
        self.plic.get_interrupt_level()
    }
}

#[test]
fn test_external_interrupt() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. 割り込みを有効化する
    // mstatus.MIE (bit 3) = 1
    cpu.csr.write(0x300, 1 << 3);
    // mie.MEIE (bit 11) = 1
    cpu.csr.write(0x304, 1 << 11);
    // mtvec を設定
    let trap_handler_addr = 0x100;
    cpu.csr.write(0x305, trap_handler_addr);

    // バスにダミー命令を配置 (ADDI x0, x0, 0)
    bus.mock_bus.write_inst32(0, 0x00000013);

    // 2. PLIC で割り込みを発生させる
    bus.plic.enabled |= 1 << 1; // ID 1 を有効化
    bus.plic.set_interrupt(1);  // ID 1 を保留中に

    // 3. 実行
    let result = cpu.step(&mut bus);

    // 4. 検証
    // トラップが発生しているはず (Machine External Interrupt = 0x8000000b)
    assert!(matches!(result, StepResult::Trap(0x8000_000b)));
    
    // PC が trap_handler_addr にジャンプしているはず
    assert_eq!(cpu.pc, trap_handler_addr);

    // mepc に元の PC (0) が保存されているはず
    assert_eq!(cpu.csr.read(0x341), 0);

    // mcause に例外コードが設定されているはず
    assert_eq!(cpu.csr.read(0x342), 0x8000_000b);

    // mstatus.MIE が 0 になっているはず
    assert_eq!((cpu.csr.read(0x300) >> 3) & 1, 0);

    // mstatus.MPIE が 1 (元のMIE) になっているはず
    assert_eq!((cpu.csr.read(0x300) >> 7) & 1, 1);
}

#[test]
fn test_interrupt_disabled_by_mstatus() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. mstatus.MIE = 0 (無効), mie.MEIE = 1 (有効)
    cpu.csr.write(0x300, 0);
    cpu.csr.write(0x304, 1 << 11);

    bus.mock_bus.write_inst32(0, 0x00000013);
    bus.plic.enabled |= 1 << 1;
    bus.plic.set_interrupt(1);

    // 2. 実行
    let result = cpu.step(&mut bus);

    // 3. 検証
    // トラップは発生せず、通常通り命令が実行されるはず
    assert!(matches!(result, StepResult::Ok));
    assert_eq!(cpu.pc, 4);
}

#[test]
fn test_interrupt_disabled_by_mie() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. mstatus.MIE = 1 (有効), mie.MEIE = 0 (無効)
    cpu.csr.write(0x300, 1 << 3);
    cpu.csr.write(0x304, 0);

    bus.mock_bus.write_inst32(0, 0x00000013);
    bus.plic.enabled |= 1 << 1;
    bus.plic.set_interrupt(1);

    // 2. 実行
    let result = cpu.step(&mut bus);

    // 3. 検証
    // トラップは発生せず、通常通り命令が実行されるはず
    assert!(matches!(result, StepResult::Ok));
    assert_eq!(cpu.pc, 4);
}
