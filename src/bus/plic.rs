/// PLIC (Platform-Level Interrupt Controller)
/// 割り込みソースの最大数（とりあえず31とする。0はソースなし用）
pub const SOURCE_COUNT: usize = 32;

pub struct Plic {
    /// 各割り込みソースの優先度 (0x000000 + 4*id)
    pub priorities: [u32; SOURCE_COUNT],
    /// 割り込みが発生しているソースのビットマスク (0x001000)
    pub pending: u32,
    /// 割り込みを有効にするかどうかのビットマスク (0x002000)
    pub enabled: u32,
    /// 割り込みを受け付ける優先度の閾値 (0x200000)
    pub threshold: u32,
    /// 現在処理中の割り込み ID を管理するビットマスク
    /// RISC-V PLIC の仕様では、Claim すると ID が返り、Complete されるまでその ID は再送されない。
    pub claimed: u32,
    /// 外部からの割り込み信号（レベルトリガー用）
    pub ip: u32,
}

impl Plic {
    pub fn new() -> Self {
        Self {
            priorities: [0; SOURCE_COUNT],
            pending: 0,
            enabled: 0,
            threshold: 0,
            claimed: 0,
            ip: 0,
        }
    }

    pub fn read(&mut self, addr: u32) -> u32 {
        match addr {
            // 0x000000..0x000080: priority
            0x000000..=0x00007c => {
                let id = (addr / 4) as usize;
                if id < SOURCE_COUNT {
                    self.priorities[id]
                } else {
                    0
                }
            }
            // 0x001000: pending
            0x001000 => self.pending,
            // 0x002000: enabled
            0x002000 => self.enabled,
            // 0x200000: threshold
            0x200000 => self.threshold,
            // 0x200004: claim
            0x200004 => self.claim(),
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u32, val: u32) {
        match addr {
            // 0x000000..0x000080: priority
            0x000000..=0x00007c => {
                let id = (addr / 4) as usize;
                if id < SOURCE_COUNT {
                    self.priorities[id] = val;
                }
            }
            // 0x001000: pending (RO)
            0x001000 => {}
            // 0x002000: enabled
            0x002000 => self.enabled = val,
            // 0x200000: threshold
            0x200000 => self.threshold = val,
            // 0x200004: complete
            0x200004 => self.complete(val),
            _ => {}
        }
    }

    fn claim(&mut self) -> u32 {
        let mut max_priority = 0;
        let mut max_id = 0;

        // 有効かつ保留中で、かつ現在処理中（Claimed）ではないものを対象とする
        let pending_enabled_not_claimed = self.pending & self.enabled & !self.claimed;
        for id in 1..SOURCE_COUNT {
            if (pending_enabled_not_claimed >> id) & 1 == 1 {
                if self.priorities[id] > max_priority {
                    max_priority = self.priorities[id];
                    max_id = id as u32;
                }
            }
        }

        if max_id > 0 && max_priority > self.threshold {
            // Claim されたら pending をクリアし、claimed にセットする
            self.pending &= !(1 << max_id);
            self.claimed |= 1 << max_id;
            max_id
        } else {
            0
        }
    }

    fn complete(&mut self, source_id: u32) {
        if source_id > 0 && source_id < SOURCE_COUNT as u32 {
            // claimed ビットをクリアする
            self.claimed &= !(1 << source_id);
            // レベルトリガーの考慮：もしデバイス信号がまだ High なら再度 pending にする
            if (self.ip >> source_id) & 1 == 1 {
                self.pending |= 1 << source_id;
            }
        }
    }

    /// 外部からの割り込み信号をセットする（レベルトリガーを想定）
    pub fn set_interrupt(&mut self, source_id: u32) {
        if source_id > 0 && source_id < SOURCE_COUNT as u32 {
            self.ip |= 1 << source_id;
            // Claimed でなければ pending に反映
            if (self.claimed >> source_id) & 1 == 0 {
                self.pending |= 1 << source_id;
            }
        }
    }

    /// 外部からの割り込み信号をクリアする
    pub fn clear_interrupt(&mut self, source_id: u32) {
        if source_id > 0 && source_id < SOURCE_COUNT as u32 {
            self.ip &= !(1 << source_id);
            // pending もクリア（ただし、すでに Claimed なものは pending はすでに 0 なはず）
            self.pending &= !(1 << source_id);
        }
    }

    /// CPU への割り込み通知が必要かどうかを判定する
    pub fn get_interrupt_level(&self) -> bool {
        // 有効かつ保留中（かつ未処理）の割り込みの中で、最大の優先度が閾値を超えているか
        // pending にはすでに !claimed のロジックが含まれている（claim 時にクリアされるため）
        let pending_enabled = self.pending & self.enabled;
        if pending_enabled == 0 {
            return false;
        }

        let mut max_priority = 0;
        for id in 1..SOURCE_COUNT {
            if (pending_enabled >> id) & 1 == 1 {
                if self.priorities[id] > max_priority {
                    max_priority = self.priorities[id];
                }
            }
        }

        max_priority > self.threshold
    }
}
