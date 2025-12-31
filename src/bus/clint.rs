/// CLINT (Core Local Interruptor)
/// タイマー割り込みとソフトウェア割り込みを管理する

pub struct Clint {
    /// Machine Software Interrupt Pending (MSIP)
    /// Hart 0 用 (4バイト)
    pub msip: u32,
    /// Machine Timer Compare Register (mtimecmp)
    /// Hart 0 用 (8バイト)
    pub mtimecmp: u64,
    /// Machine Real Time Counter (mtime)
    /// システム共有 (8バイト)
    pub mtime: u64,
}

impl Clint {
    pub fn new() -> Self {
        Self {
            msip: 0,
            mtimecmp: 0,
            mtime: 0,
        }
    }

    pub fn read(&self, addr: u32) -> u32 {
        match addr {
            // 0x0000: msip for hart 0
            0x0000 => self.msip,
            // 0x4000: mtimecmp for hart 0 (low 32-bit)
            0x4000 => (self.mtimecmp & 0xffff_ffff) as u32,
            // 0x4004: mtimecmp for hart 0 (high 32-bit)
            0x4004 => (self.mtimecmp >> 32) as u32,
            // 0xbff8: mtime (low 32-bit)
            0xbff8 => (self.mtime & 0xffff_ffff) as u32,
            // 0xbffc: mtime (high 32-bit)
            0xbffc => (self.mtime >> 32) as u32,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u32, val: u32) {
        match addr {
            // 0x0000: msip for hart 0
            0x0000 => {
                // 下位1ビットのみが有効
                self.msip = val & 1;
            }
            // 0x4000: mtimecmp for hart 0 (low 32-bit)
            0x4000 => {
                self.mtimecmp = (self.mtimecmp & !0xffff_ffff) | (val as u64);
            }
            // 0x4004: mtimecmp for hart 0 (high 32-bit)
            0x4004 => {
                self.mtimecmp = (self.mtimecmp & 0xffff_ffff) | ((val as u64) << 32);
            }
            // 0xbff8: mtime (low 32-bit)
            0xbff8 => {
                self.mtime = (self.mtime & !0xffff_ffff) | (val as u64);
            }
            // 0xbffc: mtime (high 32-bit)
            0xbffc => {
                self.mtime = (self.mtime & 0xffff_ffff) | ((val as u64) << 32);
            }
            _ => {}
        }
    }

    /// タイマーを進める
    pub fn tick(&mut self) {
        self.mtime = self.mtime.wrapping_add(1);
    }

    /// タイマー割り込みが発生しているか
    pub fn get_timer_interrupt_level(&self) -> bool {
        self.mtime >= self.mtimecmp
    }

    /// ソフトウェア割り込みが発生しているか
    pub fn get_software_interrupt_level(&self) -> bool {
        (self.msip & 1) != 0
    }
}
