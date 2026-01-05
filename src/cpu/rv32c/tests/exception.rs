#[allow(unused_imports)]
use crate::cpu::{Cpu, StepResult};

#[allow(unused_imports)]
use crate::bus::mock_bus::MockBus;

#[allow(unused_imports)]
use crate::bus::Bus;

#[test]
fn test_c_ebreak() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = MockBus::new();
    
    // C.EBREAK: 1001 00000 00000 10 -> 0x9002
    bus.write16(0, 0x9002);
    
    let (result, _) = cpu.step(&mut bus);
    
    match result {
        StepResult::Trap(cause) => assert_eq!(cause, 3), // Breakpoint exception
        _ => panic!("Expected trap, but got {:?}", result),
    }
}
