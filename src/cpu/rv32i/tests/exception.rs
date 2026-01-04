#[allow(unused_imports)]
use crate::cpu::Cpu;

#[allow(unused_imports)]
use crate::cpu::privilege_mode::PrivilegeMode;

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

#[test]
fn test_ecall_user() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    cpu.mode = PrivilegeMode::User;
    cpu.csr.mtvec = 0x0200;
    cpu.csr.mstatus = 0b1000; // MIE = 1

    // ECALL instruction
    let inst = 0x00000073;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
    assert_eq!(cpu.csr.mepc, 0x0100);
    assert_eq!(cpu.csr.mcause, 8); // Environment call from U-mode
    assert_eq!((cpu.csr.mstatus >> 7) & 1, 1); // MPIE = MIE(1)
    assert_eq!((cpu.csr.mstatus >> 3) & 1, 0); // MIE = 0
    assert_eq!((cpu.csr.mstatus >> 11) & 0b11, 0); // MPP = User(0)
}

#[test]
fn test_ecall_machine() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    cpu.mode = PrivilegeMode::Machine;
    cpu.csr.mtvec = 0x0200;
    cpu.csr.mstatus = 0b0000; // MIE = 0

    // ECALL instruction
    let inst = 0x00000073;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
    assert_eq!(cpu.csr.mcause, 11); // Environment call from M-mode
    assert_eq!((cpu.csr.mstatus >> 7) & 1, 0); // MPIE = MIE(0)
    assert_eq!((cpu.csr.mstatus >> 11) & 0b11, 3); // MPP = Machine(3)
}

#[test]
fn test_ebreak() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    cpu.csr.mtvec = 0x0200;

    // EBREAK instruction
    let inst = 0x00100073;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0100, inst);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.csr.mcause, 3); // Breakpoint
}

#[test]
fn test_mret() {
    let mut cpu = Cpu::new(0x0200);
    let mut bus = MockBus::new();
    
    // Set up state as if we are in a trap handler
    cpu.mode = PrivilegeMode::Machine;
    cpu.csr.mepc = 0x0104;
    cpu.csr.mstatus = (3 << 11) | (1 << 7) | (0 << 3); // MPP=Machine, MPIE=1, MIE=0

    // MRET instruction
    let inst = 0x30200073;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0200, inst);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x0104);
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
    assert_eq!((cpu.csr.mstatus >> 3) & 1, 1); // MIE = MPIE(1)
    assert_eq!((cpu.csr.mstatus >> 11) & 0b11, 0); // MPP is cleared to User(0)
}

#[test]
fn test_ecall_mret_flow() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();
    cpu.mode = PrivilegeMode::User;
    cpu.csr.mtvec = 0x0200;
    cpu.csr.mstatus = 0b1000; // MIE = 1

    // 1. ECALL at 0x0100
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0100, 0x00000073);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.pc, 0x0200);
    assert_eq!(cpu.mode, PrivilegeMode::Machine);
    assert_eq!(cpu.csr.mepc, 0x0100);

    // 2. In handler, increment mepc (to skip ecall) and then MRET
    cpu.csr.mepc += 4;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0200, 0x30200073); // MRET
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x0104);
    assert_eq!(cpu.mode, PrivilegeMode::User);
    assert_eq!((cpu.csr.mstatus >> 3) & 1, 1); // MIE restored to 1
}

#[test]
fn test_wfi() {
    let mut cpu = Cpu::new(0x0100);
    let mut bus = MockBus::new();

    // WFI instruction (0x10500073)
    let inst = 0x10500073;
    cpu.flush_cache_line(cpu.pc); bus.write_inst32(0x0100, inst);

    let result = cpu.step(&mut bus);

    assert!(matches!(result, crate::cpu::StepResult::Ok(4)));
    assert_eq!(cpu.pc, 0x0104);
}
