# PLIC (Platform-Level Interrupt Controller) 実装方針

PLIC は、複数の外部デバイスからの割り込み要求を管理し、優先度に基づいて CPU（ハート）へ通知する役割を担います。

## 1. 役割と基本動作
- **調停 (Arbitration)**: 複数の割り込みが同時に発生した場合、優先度（Priority）が最も高く、かつ閾値（Threshold）を超えているものを選択して CPU に通知します。
- **状態管理**: 各割り込みソースに対して「Pending（保留）」「Claimed（処理中）」「Completed（完了）」の状態を管理します。
- **MMIO インターフェース**: CPU はメモリマップド IO を通じて PLIC の設定および割り込みの取得・完了通知を行います。

## 2. メモリマップ (想定)
QEMU の `virt` マシンなどの構成を参考に、以下のレジスタセットを MMIO 領域に配置します。
ベースアドレスは `0x0c00_0000` を想定します。

| アドレスオフセット | 名称 | 説明 |
| :--- | :--- | :--- |
| `0x000000` + `4*id` | `priority` | 各割り込みソース（ID 1〜）の優先度 |
| `0x001000` | `pending` | 割り込みが発生しているソースのビットマスク |
| `0x002000` | `enable` | 割り込みを有効にするかどうかのビットマスク |
| `0x200000` | `threshold` | この値より高い優先度の割り込みのみを受け付ける |
| `0x200004` | `claim/complete` | 読み出し時は割り込み ID の取得 (Claim)、書き込み時は完了通知 (Complete) |

## 3. 割り込み処理フロー

1.  **発生 (Pending)**:
    - デバイスが PLIC の入力線を High にする。
    - PLIC 内部で `pending` ビットがセットされる。
2.  **通知 (Notification)**:
    - PLIC は `(pending & enable)` の中で最も高い `priority` を選ぶ。
    - その `priority` が `threshold` を超えていれば、CPU の `mip.MEIP` (Machine External Interrupt Pending) ビットを立てる。
3.  **取得 (Claim)**:
    - CPU はトラップハンドラ内で PLIC の `claim` レジスタを読み出す。
    - PLIC は現在発生している最高優先度の割り込み ID を返し、その割り込みを「処理中」とする（一時的に pending から外す）。
    - ID を読み出すと、CPU の `mip.MEIP` は自動的に（PLIC によって）下げられる。
4.  **完了 (Complete)**:
    - ソフトウェアがデバイスの処理を終えた後、PLIC の `complete` レジスタに取得した ID を書き込む。
    - PLIC はその ID の割り込みを再度受け入れ可能な状態に戻す。

## 4. エミュレータでの実装戦略

### PLIC 構造体
`src/bus/plic.rs` に実装されています。

```rust
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
    pub claimed: u32,
    /// 外部からの割り込み信号（レベルトリガー用）
    pub ip: u32,
}
```

### Bus との統合
`Bus` トレイトの実装内で、`0x0c00_0000` 付近のアドレスへのアクセスを `Plic` 構造体へ振り分けます。

### CPU との連携
- `Cpu::step` のループ内で、毎ステップ `bus.tick()` を通じてデバイス状態が更新された後、`check_interrupts` 内で `bus.get_interrupt_level()` を確認します。
- `true` であれば `cpu.csr.mip` の `MEIP` ビットをセットします。
- 割り込みが発生した場合、RISC-V の仕様（外部 > ソフトウェア > タイマー）に従って `handle_trap` を呼び出します。
- `Cpu` 構造体には `claim_interrupt` と `complete_interrupt` メソッドが用意されており、これらを通じて PLIC の状態を操作可能です。

## 5. 段階的な実装
1.  **最小構成**: `priority` と `threshold` を無視し、単一の割り込みソースが `mip.MEIP` を立てるだけの仕組みを作る。
2.  **Claim/Complete**: ソフトウェアから ID の取得と完了通知ができるようにレジスタを実装する。
3.  **フル実装**: 優先度、閾値、複数ソースのビットマスク管理をサポートする。
