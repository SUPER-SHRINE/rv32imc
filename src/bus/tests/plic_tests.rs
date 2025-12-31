use crate::bus::plic::Plic;

#[test]
fn test_plic_basic_priority_threshold() {
    let mut plic = Plic::new();
    
    // ソース 1 の優先度を 5 に設定
    plic.write(0x000004, 5);
    // 閾値を 3 に設定
    plic.write(0x200000, 3);
    // ソース 1 を有効化
    plic.write(0x002000, 1 << 1);
    
    // まだ割り込みは発生していない
    assert!(!plic.get_interrupt_level());
    
    // ソース 1 の割り込みを発生させる
    plic.set_interrupt(1);
    
    // 優先度 5 > 閾値 3 なので割り込みが発生するはず
    assert!(plic.get_interrupt_level());
    
    // 閾値を 6 に上げると割り込みが止まるはず
    plic.write(0x200000, 6);
    assert!(!plic.get_interrupt_level());
}

#[test]
fn test_plic_claim_complete() {
    let mut plic = Plic::new();
    
    plic.write(0x000004, 5); // Source 1 priority = 5
    plic.write(0x000008, 10); // Source 2 priority = 10
    plic.write(0x200000, 3); // Threshold = 3
    plic.write(0x002000, (1 << 1) | (1 << 2)); // Enable 1 and 2
    
    plic.set_interrupt(1);
    plic.set_interrupt(2);
    
    // 高い優先度 (Source 2) が Claim されるはず
    let claimed_id = plic.read(0x200004);
    assert_eq!(claimed_id, 2);
    
    // Claim されたので、再度読み出すと次は Source 1 が返るはず
    let claimed_id2 = plic.read(0x200004);
    assert_eq!(claimed_id2, 1);
    
    // 全て Claim されたので 0 が返るはず
    let claimed_id3 = plic.read(0x200004);
    assert_eq!(claimed_id3, 0);

    // Source 2 を Complete する
    plic.write(0x200004, 2);

    // デバイス信号がまだ High (set_interrupt されたまま) なので、再度 Pending になるはず
    assert_eq!(plic.read(0x001000), 1 << 2);
    assert!(plic.get_interrupt_level());

    // 再度 Claim すると 2 が返る
    assert_eq!(plic.read(0x200004), 2);
}

#[test]
fn test_cpu_plic_access() {
    use crate::bus::default_bus::DefaultBus;
    use crate::cpu::Cpu;
    use crate::bus::Bus;

    let mut bus = DefaultBus::new(1024);
    let mut cpu = Cpu::new(0);

    // PLIC 設定 (Source 1)
    bus.write32(0x0c000004, 5); // Priority
    bus.write32(0x0c002000, 1 << 1); // Enable
    bus.write32(0x0c200000, 3); // Threshold

    // 割り込み発生
    bus.plic.set_interrupt(1);

    // CPU から Claim
    let id = cpu.claim_interrupt(&mut bus);
    assert_eq!(id, 1);

    // CPU から Complete
    cpu.complete_interrupt(&mut bus, 1);

    // Complete されたので pending がクリアされているはず（ipをクリアしない限り再度pendingになるが、ここでは一旦完了を確認）
}

#[test]
fn test_plic_pending_mask() {
    let mut plic = Plic::new();
    plic.set_interrupt(1);
    plic.set_interrupt(3);
    
    assert_eq!(plic.read(0x001000), (1 << 1) | (1 << 3));
}
