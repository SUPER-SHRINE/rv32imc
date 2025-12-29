use super::Bus;
use std::fs;
use std::io;

pub struct DefaultBus {
    pub memory: Vec<u8>,
}

impl DefaultBus {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
        }
    }

    pub fn load_bin(&mut self, path: &str, offset: usize) -> io::Result<()> {
        let data = fs::read(path)?;
        for (i, byte) in data.iter().enumerate() {
            if offset + i < self.memory.len() {
                self.memory[offset + i] = *byte;
            }
        }
        Ok(())
    }
}

impl Bus for DefaultBus {
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
