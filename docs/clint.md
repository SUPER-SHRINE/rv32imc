# CLINT (Core Local Interruptor) 実装方針

RISC-V におけるタイマー割り込みおよびソフトウェア割り込みを管理する CLINT モジュールの実装方針を以下にまとめます。

## 1. 役割と機能
CLINT は、各ハート（コア）に対して以下の機能を提供します。
- **タイマー割り込み**: `mtime` (リアルタイムカウンタ) と `mtimecmp` (比較レジスタ) を管理し、`mtime >= mtimecmp` の時に割り込みを発生させる。
- **ソフトウェア割り込み**: `msip` レジスタへの書き込みにより、IPI (Inter-Processor Interrupt) を発生させる。

## 2. メモリマップ (予定)
QEMU 等の標準的な RISC-V 実装（virt マシン等）に準拠し、ベースアドレスを `0x0200_0000` とします。

| アドレスオフセット | 名称 | 説明 |
| :--- | :--- | :--- |
| `0x0000` | `msip` | Machine Software Interrupt Pending (各ハート 4バイト) |
| `0x4000` | `mtimecmp` | Machine Timer Compare Register (各ハート 8バイト) |
| `0xbff8` | `mtime` | Machine Real Time Counter (共有 8バイト) |

※ 本プロジェクトは現在シングルコア（rv32imc）を対象としているため、Hart 0 用のレジスタのみを実装します。

## 3. 実装手順

### ステップ 1: `Clint` 構造体の作成 (`src/bus/clint.rs`)
`plic.rs` と同様に、CLINT の状態を保持する構造体を定義します。
- `mtime`: 64bit カウンタ
- `mtimecmp`: 64bit 比較レジスタ
- `msip`: 32bit (下位1bitを使用)
- `read/write` メソッドの実装

### ステップ 2: `DefaultBus` への統合 (`src/bus/default_bus.rs`)
- `DefaultBus` に `Clint` インスタンスを追加。
- アドレス `0x0200_0000` 付近のアクセスを `Clint` に振り分ける。
- `Bus` トレイトの `get_timer_interrupt_level()` 等の実装を `Clint` の状態に基づいて更新する。

### ステップ 3: `mtime` のインクリメント
CPU の実行サイクルに合わせて `mtime` を更新する仕組みを導入します。
- `Cpu::step()` などのタイミングで、バス経由または直接 CLINT のタイマーを進める。

### ステップ 4: CPU の割り込みチェックとの連携
既存の `src/cpu.rs` 内の `check_interrupts` は既に `bus.get_timer_interrupt_level()` をチェックするようになっているため、`DefaultBus` 側の実装が完了すればタイマー割り込みが動作するはずです。

## 4. 考慮事項
- **rv32 の 64bit アクセス**: 32bit CPU では 64bit レジスタ (`mtime`, `mtimecmp`) へのアクセスが 2回に分かれるため、アトミックな更新や読み取りについて考慮が必要になる場合があります（特に `mtime`）。
- **タイマーの精度**: エミュレータ上の 1 サイクルを `mtime` のいくつ分とするか（または実時間と同期させるか）を検討する必要があります。
