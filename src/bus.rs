pub mod default_bus;
pub mod mock_bus;
pub mod plic;
pub mod clint;

#[cfg(test)]
mod tests;

/// メモリの読み書きを行うためのバス。
/// 実装は外側に任せる。
pub trait Bus {
    fn read8(&mut self, addr: u32) -> u8;
    fn read16(&mut self, addr: u32) -> u16;
    fn read32(&mut self, addr: u32) -> u32;

    fn write8(&mut self, addr: u32, val: u8);
    fn write16(&mut self, addr: u32, val: u16);
    fn write32(&mut self, addr: u32, val: u32);

    /// PLIC からの割り込み要求レベルを取得する（デフォルトは false）
    fn get_interrupt_level(&self) -> bool {
        false
    }

    /// タイマー割り込み要求レベルを取得する（デフォルトは false）
    fn get_timer_interrupt_level(&self) -> bool {
        false
    }

    /// ソフトウェア割り込み要求レベルを取得する（デフォルトは false）
    fn get_software_interrupt_level(&self) -> bool {
        false
    }

    /// クロックを進める
    fn tick(&mut self) {}

    /// PLIC からの割り込みを取得する (Claim)
    fn plic_claim(&mut self) -> u32 {
        0
    }

    /// PLIC に割り込みの完了を通知する (Complete)
    fn plic_complete(&mut self, _source_id: u32) {}
}
