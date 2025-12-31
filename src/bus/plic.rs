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
}
