use crate::cpu::{Cpu, StepResult};
use crate::bus::mock_bus::MockBus;
use crate::bus::plic::Plic;
use crate::bus::Bus;

struct InterruptTestBus {
    mock_bus: MockBus,
    plic: Plic,
    timer_interrupt: bool,
}

impl InterruptTestBus {
    fn new() -> Self {
        Self {
            mock_bus: MockBus::new(),
            plic: Plic::new(),
            timer_interrupt: false,
        }
    }
}

const PLIC_BASE: u32 = 0x0c00_0000;
const PLIC_SIZE: u32 = 0x0040_0000;

impl Bus for InterruptTestBus {
    fn read8(&mut self, addr: u32) -> u8 {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.read(addr - PLIC_BASE) as u8
        } else {
            self.mock_bus.read8(addr)
        }
    }
    fn read16(&mut self, addr: u32) -> u16 {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.read(addr - PLIC_BASE) as u16
        } else {
            self.mock_bus.read16(addr)
        }
    }
    fn read32(&mut self, addr: u32) -> u32 {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.read(addr - PLIC_BASE)
        } else {
            self.mock_bus.read32(addr)
        }
    }
    fn write8(&mut self, addr: u32, val: u8) {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.write(addr - PLIC_BASE, val as u32);
        } else {
            self.mock_bus.write8(addr, val)
        }
    }
    fn write16(&mut self, addr: u32, val: u16) {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.write(addr - PLIC_BASE, val as u32);
        } else {
            self.mock_bus.write16(addr, val)
        }
    }
    fn write32(&mut self, addr: u32, val: u32) {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.write(addr - PLIC_BASE, val);
        } else {
            self.mock_bus.write32(addr, val)
        }
    }
    fn get_interrupt_level(&self) -> bool {
        self.plic.get_interrupt_level()
    }
    fn get_timer_interrupt_level(&self) -> bool {
        self.timer_interrupt
    }
}

#[test]
fn test_external_interrupt() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. 割り込みを有効化する
    // mstatus.MIE (bit 3) = 1
    let _ = cpu.csr.write(0x300, 1 << 3);
    // mie.MEIE (bit 11) = 1
    let _ = cpu.csr.write(0x304, 1 << 11);
    // mtvec を設定
    let trap_handler_addr = 0x100;
    let _ = cpu.csr.write(0x305, trap_handler_addr);

    // バスにダミー命令を配置 (ADDI x0, x0, 0)
    bus.mock_bus.write_inst32(0, 0x00000013);

    // 2. PLIC で割り込みを発生させる
    bus.plic.enabled |= 1 << 1; // ID 1 を有効化
    bus.plic.priorities[1] = 1; // ID 1 の優先度を 1 に設定 (閾値 0 より大きくする)
    bus.plic.set_interrupt(1);  // ID 1 を保留中に

    // 3. 実行
    let (result, _) = cpu.step(&mut bus);

    // 4. 検証
    // トラップが発生しているはず (Machine External Interrupt = 0x8000000b)
    assert!(matches!(result, StepResult::Trap(0x8000_000b)));
    
    // PC が trap_handler_addr にジャンプしているはず
    assert_eq!(cpu.pc, trap_handler_addr);

    // mepc に元の PC (0) が保存されているはず
    assert_eq!(cpu.csr.read(0x341).unwrap(), 0);

    // mcause に例外コードが設定されているはず
    assert_eq!(cpu.csr.read(0x342).unwrap(), 0x8000_000b);

    // mstatus.MIE が 0 になっているはず
    assert_eq!((cpu.csr.read(0x300).unwrap() >> 3) & 1, 0);

    // mstatus.MPIE が 1 (元のMIE) になっているはず
    assert_eq!((cpu.csr.read(0x300).unwrap() >> 7) & 1, 1);
}

#[test]
fn test_timer_interrupt() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. 割り込みを有効化する
    // mstatus.MIE = 1
    let _ = cpu.csr.write(0x300, 1 << 3);
    // mie.MTIE (bit 7) = 1
    let _ = cpu.csr.write(0x304, 1 << 7);
    // mtvec を設定
    let trap_handler_addr = 0x200;
    let _ = cpu.csr.write(0x305, trap_handler_addr);

    bus.mock_bus.write_inst32(0, 0x00000013); // NOP

    // 2. タイマー割り込みを発生させる
    bus.timer_interrupt = true;

    // 3. 実行
    let (result, _) = cpu.step(&mut bus);

    // 4. 検証
    assert!(matches!(result, StepResult::Trap(0x8000_0007)));
    assert_eq!(cpu.pc, trap_handler_addr);
    assert_eq!(cpu.csr.read(0x342).unwrap(), 0x8000_0007);
}

#[test]
fn test_timer_interrupt_after_csr_op() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. 割り込みを有効化する
    // mstatus.MIE = 1, mie.MTIE = 1, mtvec = 0x200
    let _ = cpu.csr.write(0x300, 1 << 3);
    let _ = cpu.csr.write(0x304, 1 << 7);
    let _ = cpu.csr.write(0x305, 0x200);

    // CSR 命令を配置 (CSRRW x0, mscratch, x1) -> rd=0 なので読み出さないはず
    // mscratch(0x340) に値を書いておき、x1(初期値0) で上書きする
    let _ = cpu.csr.write(0x340, 0xdeadbeef);
    cpu.regs[1] = 0x12345678;
    // 0x00000013 は ADDI x0, x0, 0 (NOP)
    // 0x34001073 は csrrw x0, mscratch, x1
    bus.mock_bus.write_inst32(0, 0x34009073); 

    // 2. タイマー割り込みを発生させる
    bus.timer_interrupt = true;

    // 3. 実行
    // このステップで CSRRW が実行され、割り込みチェックが行われるはず
    // 割り込みは命令実行の直前にチェックされるので、最初の step で Trap になる
    let (result, _) = cpu.step(&mut bus);

    // 4. 検証
    assert!(matches!(result, StepResult::Trap(0x8000_0007)));
    
    // 5. 割り込み処理から戻った後にもう一度 CSR 操作をしてみる
    cpu.pc = 0x0;
    cpu.csr.mstatus |= 1 << 3; // MIE を戻す
    bus.timer_interrupt = false;
    
    // rd=0 の CSRRW を実行 (読み出し副作用がないことを期待)
    let _ = cpu.step(&mut bus); 
    assert_eq!(cpu.csr.read(0x340).unwrap(), 0x12345678);
}

#[test]
fn test_interrupt_priority() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. 外部割り込みとタイマー割り込みの両方を有効化
    let _ = cpu.csr.write(0x300, 1 << 3);
    let _ = cpu.csr.write(0x304, (1 << 11) | (1 << 7));
    let _ = cpu.csr.write(0x305, 0x100);

    bus.mock_bus.write_inst32(0, 0x00000013);

    // 2. 両方の割り込みを発生させる
    bus.timer_interrupt = true;
    bus.plic.enabled |= 1 << 1;
    bus.plic.priorities[1] = 1;
    bus.plic.set_interrupt(1);

    // 3. 実行
    let (result, _) = cpu.step(&mut bus);

    // 4. 検証: 外部割り込みが優先されるはず
    assert!(matches!(result, StepResult::Trap(0x8000_000b)));
    assert_eq!(cpu.csr.read(0x342).unwrap(), 0x8000_000b);
}

#[test]
fn test_full_interrupt_handler_flow() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. 初期設定
    let _ = cpu.csr.write(0x300, 1 << 3); // mstatus.MIE = 1
    let _ = cpu.csr.write(0x304, 1 << 11); // mie.MEIE = 1
    let _ = cpu.csr.write(0x305, 0x100); // mtvec = 0x100

    // メインプログラム
    bus.mock_bus.write_inst32(0x0, 0x00000013); // 0x0: NOP
    bus.mock_bus.write_inst32(0x4, 0x00100013); // 0x4: ADDI x0, x0, 1 (Return point markers)

    // トラップハンドラ (0x100)
    // a. Claim: x5 = *0x0c200004
    bus.mock_bus.write_inst32(0x100, 0x0c2002b7); // lui x5, 0x0c200
    bus.mock_bus.write_inst32(0x104, 0x0042a283); // lw x5, 4(x5) (Claim)
    // b. Complete: *0x0c200004 = x5
    bus.mock_bus.write_inst32(0x108, 0x0c200337); // lui x6, 0x0c200
    bus.mock_bus.write_inst32(0x10c, 0x00532223); // sw x5, 4(x6) (Complete)
    // c. mret
    bus.mock_bus.write_inst32(0x110, 0x30200073); // mret

    // 2. 外部割り込みを発生させる
    bus.plic.enabled |= 1 << 1;
    bus.plic.priorities[1] = 1;
    bus.plic.set_interrupt(1);

    // 3. 実行
    // 1ステップ目: 割り込み検知 -> トラップハンドラへジャンプ
    let (result1, _) = cpu.step(&mut bus);
    assert!(matches!(result1, StepResult::Trap(0x8000_000b)));
    assert_eq!(cpu.pc, 0x100);

    // ハンドラ内実行
    cpu.step(&mut bus); // lui x5, 0x0c200
    cpu.step(&mut bus); // lw x5, 4(x5) (Claim)
    assert_eq!(cpu.regs[5], 1); // ID 1 が Claim されたはず
    assert_eq!(bus.plic.claimed, 1 << 1); // PLIC 側でも Claimed になっているはず

    cpu.step(&mut bus); // lui x6, 0x0c200
    cpu.step(&mut bus); // sw x5, 4(x6) (Complete)
    assert_eq!(bus.plic.claimed, 0); // PLIC 側で Claimed がクリアされたはず

    // PLIC 信号をクリアしておく（そうしないと再度割り込みが発生する）
    bus.plic.clear_interrupt(1);

    let (result_mret, _) = cpu.step(&mut bus); // mret
    assert!(matches!(result_mret, StepResult::Jumped));
    assert_eq!(cpu.pc, 0x0); // mepc (0) に戻るはず

    // メインプログラムの続き
    cpu.step(&mut bus); // 0x0: NOP
    assert_eq!(cpu.pc, 0x4);
}

#[test]
fn test_interrupt_disabled_by_mstatus() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. mstatus.MIE = 0 (無効), mie.MEIE = 1 (有効)
    let _ = cpu.csr.write(0x300, 0);
    let _ = cpu.csr.write(0x304, 1 << 11);

    bus.mock_bus.write_inst32(0, 0x00000013);
    bus.plic.enabled |= 1 << 1;
    bus.plic.set_interrupt(1);

    // 2. 実行
    let (result, _) = cpu.step(&mut bus);

    // 3. 検証
    // トラップは発生せず、通常通り命令が実行されるはず
    assert!(matches!(result, StepResult::Ok(4)));
    assert_eq!(cpu.pc, 4);
}

#[test]
fn test_interrupt_disabled_by_mie() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = InterruptTestBus::new();

    // 1. mstatus.MIE = 1 (有効), mie.MEIE = 0 (無効)
    let _ = cpu.csr.write(0x300, 1 << 3);
    let _ = cpu.csr.write(0x304, 0);

    bus.mock_bus.write_inst32(0, 0x00000013);
    bus.plic.enabled |= 1 << 1;
    bus.plic.set_interrupt(1);

    // 2. 実行
    let (result, _) = cpu.step(&mut bus);

    // 3. 検証
    // トラップは発生せず、通常通り命令が実行されるはず
    assert!(matches!(result, StepResult::Ok(4)));
    assert_eq!(cpu.pc, 4);
}
