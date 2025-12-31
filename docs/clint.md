# CLINT (Core Local Interruptor) 実装仕様

RISC-V におけるタイマー割り込みおよびソフトウェア割り込みを管理する CLINT モジュールの実装仕様を以下にまとめます。

## 1. 役割と機能
CLINT は、各ハート（コア）に対して以下の機能を提供します。
- **タイマー割り込み**: `mtime` (リアルタイムカウンタ) と `mtimecmp` (比較レジスタ) を管理し、`mtime >= mtimecmp` の時に割り込みを発生させる。
- **ソフトウェア割り込み**: `msip` レジスタへの書き込みにより、IPI (Inter-Processor Interrupt) を発生させる。

## 2. メモリマップ
QEMU 等の標準的な RISC-V 実装（virt マシン等）に準拠し、ベースアドレスを `0x0200_0000` とします。

| アドレスオフセット | 名称 | 説明 |
| :--- | :--- | :--- |
| `0x0000` | `msip` | Machine Software Interrupt Pending (各ハート 4バイト) |
| `0x4000` | `mtimecmp` | Machine Timer Compare Register (各ハート 8バイト) |
| `0xbff8` | `mtime` | Machine Real Time Counter (共有 8バイト) |

※ 本プロジェクトは現在シングルコア（rv32imc）を対象としているため、Hart 0 用のレジスタのみを実装しています。

## 3. 実装詳細

### `Clint` 構造体 (`src/bus/clint.rs`)
CLINT の状態を保持する構造体です。
- `mtime`: 64bit カウンタ
- `mtimecmp`: 64bit 比較レジスタ
- `msip`: 32bit (下位1bitを使用)

### `DefaultBus` への統合 (`src/bus/default_bus.rs`)
- `DefaultBus` に `Clint` インスタンスを保持しています。
- アドレス `0x0200_0000` 付近のアクセスを `Clint` に振り分けています。
- `Bus` トレイトの `get_timer_interrupt_level()` 等のメソッドを通じて、CPU へ割り込み状態を通知します。

### `mtime` のインクリメント
- `Cpu::step()` の開始時に `bus.tick()` を呼び出し、その中で `clint.tick()` を実行してタイマーを進めています。

### CPU の割り込みチェックとの連携
- `Cpu::check_interrupts` 内で `bus.get_timer_interrupt_level()` および `bus.get_software_interrupt_level()` をチェックし、`mip.MTIP` および `mip.MSIP` を更新します。
