use super::Bus;
use super::plic::Plic;
use std::fs;
use std::io;

pub const PLIC_BASE: u32 = 0x0c00_0000;
pub const PLIC_SIZE: u32 = 0x0040_0000;

pub struct DefaultBus {
    pub memory: Vec<u8>,
    pub plic: Plic,
}

impl DefaultBus {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            plic: Plic::new(),
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
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.read(addr - PLIC_BASE) as u8
        } else {
            self.memory[addr as usize]
        }
    }

    fn read16(&mut self, addr: u32) -> u16 {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.read(addr - PLIC_BASE) as u16
        } else {
            let addr = addr as usize;
            u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]])
        }
    }

    fn read32(&mut self, addr: u32) -> u32 {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.read(addr - PLIC_BASE)
        } else {
            let addr = addr as usize;
            u32::from_le_bytes([
                self.memory[addr],
                self.memory[addr + 1],
                self.memory[addr + 2],
                self.memory[addr + 3],
            ])
        }
    }

    fn write8(&mut self, addr: u32, val: u8) {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.write(addr - PLIC_BASE, val as u32);
        } else {
            self.memory[addr as usize] = val;
        }
    }

    fn write16(&mut self, addr: u32, val: u16) {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.write(addr - PLIC_BASE, val as u32);
        } else {
            let addr = addr as usize;
            self.memory[addr..addr + 2].copy_from_slice(&val.to_le_bytes());
        }
    }

    fn write32(&mut self, addr: u32, val: u32) {
        if addr >= PLIC_BASE && addr < PLIC_BASE + PLIC_SIZE {
            self.plic.write(addr - PLIC_BASE, val);
        } else {
            let addr = addr as usize;
            self.memory[addr..addr + 4].copy_from_slice(&val.to_le_bytes());
        }
    }

    fn get_interrupt_level(&self) -> bool {
        self.plic.get_interrupt_level()
    }
}
