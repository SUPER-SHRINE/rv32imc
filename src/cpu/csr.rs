/// 制御ステータスレジスタ (CSR)
#[derive(Default)]
pub struct Csr {
    // 主要なマシンモードCSR
    pub mstatus: u32,
    pub mtvec:   u32,
    pub mie:     u32,
    pub mepc:    u32,
    pub mcause:  u32,
    pub mtval:   u32,
    pub mip:     u32,
}

impl Csr {
    pub fn read(&self, addr: u32) -> u32 {
        match addr {
            0x300 => self.mstatus,
            0x305 => self.mtvec,
            0x304 => self.mie,
            0x341 => self.mepc,
            0x342 => self.mcause,
            0x343 => self.mtval,
            0x344 => self.mip,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u32, val: u32) {
        match addr {
            0x300 => self.mstatus = val,
            0x305 => self.mtvec = val,
            0x304 => self.mie = val,
            0x341 => self.mepc = val,
            0x342 => self.mcause = val,
            0x343 => self.mtval = val,
            0x344 => self.mip = val,
            _ => {}
        }
    }
}
