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
    pub mscratch: u32,
    pub mcounteren: u32,
    pub pmpcfg0: u32,
    pub pmpaddr: [u32; 4],
    pub satp:    u32,
}

impl Csr {
    pub fn read(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            0x300 => Ok(self.mstatus),
            0x180 => Ok(self.satp),
            0x301 => Ok(0x40101104), // misa: RV32IMCU (I=1<<8, M=1<<12, C=1<<2, U=1<<20, MXL=1(RV32)<<30)
            0x302 => Ok(0), // medeleg
            0x303 => Ok(0), // mideleg
            0x306 => Ok(self.mcounteren),
            0x744 => Ok(0), // mtp (Machine Trap Pointer?) - Not standard, but sometimes used in tests or specific impls
            0x7a0..=0x7a3 => Ok(0), // tselect, tdata1, tdata2, tdata3
            0x7a5 => Ok(0), // tinfo
            0x320..=0x33f => Ok(0), // mcountinhibit, mhpmevent
            0xb00..=0xb1f => Ok(0), // mcycle, minstret, etc. (Machine read-only/read-write)
            0xb80..=0xb9f => Ok(0), // mcycleh, minstreth, etc.
            0xc00..=0xc1f => Ok(0), // cycle, time, instret, etc. (Read-only)
            0xc80..=0xc9f => Ok(0), // cycleh, timeh, instreth, etc. (Read-only)
            0x305 => Ok(self.mtvec),
            0x304 => Ok(self.mie),
            0x340 => Ok(self.mscratch),
            0x341 => Ok(self.mepc),
            0x342 => Ok(self.mcause),
            0x343 => Ok(self.mtval),
            0x344 => Ok(self.mip),
            0x3a0 => Ok(self.pmpcfg0),
            0x3b0..=0x3b3 => Ok(self.pmpaddr[(addr - 0x3b0) as usize]),
            0xf11 => Ok(0), // mvendorid
            0xf12 => Ok(0), // marchid
            0xf13 => Ok(0), // mimpid
            0xf14 => Ok(0), // mhartid
            _ => Err(()),
        }
    }

    pub fn write(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        // 読み取り専用 CSR (bits 11-10 == 11) への書き込みは不正命令
        // ただし、0xc00-0xc1f (counters), 0xc80-0xc9f (counters high) は読み取り専用
        if (addr >> 10) & 0b11 == 0b11 {
            return Err(());
        }

        match addr {
            0x300 => {
                // mstatus: 制限付き書き込み
                // MPP は Machine (3) と User (0) のみサポート
                let mask = 0x807E1888;
                let mut val = val;
                
                let mpp = (val >> 11) & 0b11;
                if mpp == 1 || mpp == 2 {
                    // Sモードや保留モードが指定されたら Userモードに丸める
                    val &= !(0b11 << 11);
                }
                
                let new_val = (self.mstatus & !mask) | (val & mask);
                self.mstatus = new_val;
                Ok(())
            }
            0x301 => Ok(()), // misa: WARL (とりあえず固定)
            0x180 => { self.satp = val; Ok(()) }
            0x302 => Ok(()), // medeleg
            0x303 => Ok(()), // mideleg
            0x306 => { self.mcounteren = val; Ok(()) }
            0x744 => Ok(()), // mtp
            0x7a0..=0x7a3 => Ok(()), // tselect, tdata1, tdata2, tdata3
            0x7a5 => Ok(()), // tinfo
            0x320..=0x33f => Ok(()), // mcountinhibit, mhpmevent
            0xb00..=0xb1f => Ok(()), // mcycle, minstret, etc. (Machine read-only/read-write)
            0xb80..=0xb9f => Ok(()), // mcycleh, minstreth, etc.
            0xc00..=0xc1f => Err(()), // cycle, time, instret, etc. (Read-only)
            0xc80..=0xc9f => Err(()), // cycleh, timeh, instreth, etc. (Read-only)
            0x305 => { self.mtvec = val; Ok(()) }
            0x304 => { self.mie = val; Ok(()) }
            0x340 => { self.mscratch = val; Ok(()) }
            0x341 => { self.mepc = val; Ok(()) }
            0x342 => { self.mcause = val; Ok(()) }
            0x343 => { self.mtval = val; Ok(()) }
            0x344 => { self.mip = val; Ok(()) }
            0x3a0 => { self.pmpcfg0 = val; Ok(()) }
            0x3b0..=0x3b3 => {
                self.pmpaddr[(addr - 0x3b0) as usize] = val;
                Ok(())
            }
            _ => Err(()),
        }
    }
}
