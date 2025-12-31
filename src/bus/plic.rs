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
    /// 現在処理中の割り込み ID (Option で管理するか、0x200004 レジスタの状態として管理するか)
    /// RISC-V PLIC の仕様では、Claim すると ID が返り、Complete されるまでその ID は再送されない。
    pub claimed: u32,
}

impl Plic {
    pub fn new() -> Self {
        Self {
            priorities: [0; SOURCE_COUNT],
            pending: 0,
            enabled: 0,
            threshold: 0,
            claimed: 0,
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

        let pending_enabled = self.pending & self.enabled;
        for id in 1..SOURCE_COUNT {
            if (pending_enabled >> id) & 1 == 1 {
                if self.priorities[id] > max_priority {
                    max_priority = self.priorities[id];
                    max_id = id as u32;
                }
            }
        }

        if max_id > 0 && max_priority > self.threshold {
            // Claim されたら pending をクリアする
            self.pending &= !(1 << max_id);
            self.claimed = max_id;
            max_id
        } else {
            0
        }
    }

    fn complete(&mut self, source_id: u32) {
        if self.claimed == source_id {
            self.claimed = 0;
            // 本来はここで再度 pending を受け入れ可能にするなどの処理が必要な場合があるが
            // 今回の簡易実装では claim 時に pending をクリアしているので特になし
        }
    }

    /// 外部からの割り込み信号をセットする（デバッグ・テスト用またはデバイス接続用）
    pub fn set_interrupt(&mut self, source_id: u32) {
        if source_id > 0 && source_id < SOURCE_COUNT as u32 {
            self.pending |= 1 << source_id;
        }
    }

    /// CPU への割り込み通知が必要かどうかを判定する
    pub fn get_interrupt_level(&self) -> bool {
        // 有効かつ保留中の割り込みの中で、最大の優先度が閾値を超えているか
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
