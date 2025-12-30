use crate::cpu::{Cpu, StepResult};
use crate::bus::Bus;

struct TestBus {
    memory: [u8; 1024],
}

impl TestBus {
    fn new() -> Self {
        Self { memory: [0; 1024] }
    }
}

impl Bus for TestBus {
    fn read8(&mut self, addr: u32) -> u8 { self.memory[addr as usize] }
    fn read16(&mut self, addr: u32) -> u16 {
        (self.memory[addr as usize] as u16) | ((self.memory[addr as usize + 1] as u16) << 8)
    }
    fn read32(&mut self, addr: u32) -> u32 {
        (self.memory[addr as usize] as u32) |
        ((self.memory[addr as usize + 1] as u32) << 8) |
        ((self.memory[addr as usize + 2] as u32) << 16) |
        ((self.memory[addr as usize + 3] as u32) << 24)
    }
    fn write8(&mut self, addr: u32, val: u8) { self.memory[addr as usize] = val; }
    fn write16(&mut self, addr: u32, val: u16) {
        self.memory[addr as usize] = val as u8;
        self.memory[addr as usize + 1] = (val >> 8) as u8;
    }
    fn write32(&mut self, addr: u32, val: u32) {
        self.memory[addr as usize] = val as u8;
        self.memory[addr as usize + 1] = (val >> 8) as u8;
        self.memory[addr as usize + 2] = (val >> 16) as u8;
        self.memory[addr as usize + 3] = (val >> 24) as u8;
    }
}

#[test]
fn test_c_ebreak() {
    let mut cpu = Cpu::new(0x0);
    let mut bus = TestBus::new();
    
    // C.EBREAK: 1001 00000 00000 10 -> 0x9002
    bus.write16(0, 0x9002);
    
    let result = cpu.step(&mut bus);
    
    match result {
        StepResult::Trap(cause) => assert_eq!(cause, 3), // Breakpoint exception
        _ => panic!("Expected trap, but got {:?}", result),
    }
}
