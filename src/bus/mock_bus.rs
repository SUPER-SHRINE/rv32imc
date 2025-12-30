pub(crate) use crate::bus::Bus;

// テストコードでのみ使用している構造体のため通常ビルド時に未使用の警告が出る(邪魔なので握り潰す)
#[allow(unused)]
pub(crate) struct MockBus {
    memory: [u8; 8192],
}

impl MockBus {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self {
            memory: [0; 8192],
        }
    }

    #[allow(unused)]
    pub(crate) fn write_inst32(&mut self, addr: u32, inst: u32) {
        self.write32(addr, inst);
    }

    #[allow(unused)]
    pub(crate) fn write_inst16(&mut self, addr: u32, inst: u16) {
        self.write16(addr, inst);
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
