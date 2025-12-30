pub(super) use crate::bus::Bus;

pub(super) struct MockBus {
    memory: [u8; 8192],
}

impl MockBus {
    pub(super) fn new() -> Self {
        Self {
            memory: [0; 8192],
        }
    }

    pub(super) fn write_inst32(&mut self, addr: u32, inst: u32) {
        self.write32(addr, inst);
    }
}

impl Bus for MockBus {
    fn read8(&mut self, addr: u32) -> u8 {
        self.memory[addr as usize]
    }

    fn read16(&mut self, addr: u32) -> u16 {
        let addr = addr as usize;
        u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]])
    }

    fn read32(&mut self, addr: u32) -> u32 {
        let addr = addr as usize;
        u32::from_le_bytes([
            self.memory[addr],
            self.memory[addr + 1],
            self.memory[addr + 2],
            self.memory[addr + 3],
        ])
    }

    fn write8(&mut self, addr: u32, val: u8) {
        self.memory[addr as usize] = val;
    }

    fn write16(&mut self, addr: u32, val: u16) {
        let addr = addr as usize;
        self.memory[addr..addr + 2].copy_from_slice(&val.to_le_bytes());
    }

    fn write32(&mut self, addr: u32, val: u32) {
        let addr = addr as usize;
        self.memory[addr..addr + 4].copy_from_slice(&val.to_le_bytes());
    }
}
