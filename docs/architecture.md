# アーキテクチャ設計書 (Architecture Design)

本ドキュメントでは、`docs/overview.md`に基づいたRV32IMCエミュレータの実装構造について記述します。

## 全体構成

エミュレータは、主に以下のコンポーネントで構成されます。

1.  **CPU Core**: レジスタ、PC、CSRの状態を管理し、命令の実行を制御します。
2.  **Memory Bus**: CPUと外部デバイス（メモリ、VRAM、入力など）を接続する抽象化レイヤーです。
3.  **Execution Engine**: 命令のフェッチ、デコード、および実行を行い、CPUの状態を更新します。

## モジュール構成

`src/cpu.rs` が肥大化したため、機能ごとにモジュールを分割しています。

```text
src/
  ├── main.rs                   (バイナリエントリポイント)
  ├── lib.rs                    (クレートのルート)
  ├── bus.rs                    (バス・トレイトの定義)
  ├── bus/
  │    ├── default_bus.rs       (標準的なメモリ実装)
  │    └── mock_bus.rs          (テスト用のモック実装)
  ├── cpu.rs                    (Cpu構造体の定義とメインループ)
  └── cpu/
       ├── decode.rs            (命令の共通デコードロジック)
       ├── handle_trap.rs       (例外・トラップ処理の実装)
       ├── csr.rs               (CSR: 制御ステータスレジスタ関連)
       ├── privilege_mode.rs    (特権モードの定義)
       ├── rv32i.rs             (RV32I 命令のディスパッチ)
       ├── rv32m.rs             (RV32M 命令のディスパッチ)
       ├── rv32c.rs             (RV32C 命令のディスパッチ)
       ├── rv32i/               (RV32I 命令の各実装とテスト)
       ├── rv32m/               (RV32M 命令の各実装とテスト)
       └── rv32c/               (RV32C 命令の各実装とテスト)
```

---

## 主要な構造体とトレイト

### 1. CPU 構造体 (`Cpu`)

CPUの内部状態を保持します。(`src/cpu.rs`)

```rust
pub struct Cpu {
    /// 32本の汎用レジスタ (x0-x31)
    pub regs: [u32; 32],
    /// プログラムカウンタ
    pub pc: u32,
    /// 制御ステータスレジスタ (CSR)
    pub csr: Csr,
    /// 特権モード
    pub mode: PrivilegeMode,
}
```

### 2. バス・トレイト (`Bus`)

メモリやI/Oへのアクセスを抽象化します。ホスト側からの容易な操作を実現するため、トレイトとして定義します。

```rust
pub trait Bus {
    fn read8(&mut self, addr: u32) -> u8;
    fn read16(&mut self, addr: u32) -> u16;
    fn read32(&mut self, addr: u32) -> u32;
    
    fn write8(&mut self, addr: u32, val: u8);
    fn write16(&mut self, addr: u32, val: u16);
    fn write32(&mut self, addr: u32, val: u32);
}
```

### 3. 命令デコードと実行

現在は速度重視のため、`Instruction` 列挙型を介さず、`execute` メソッド内で直接バイナリをデコードして実行しています。
デコードロジックは `src/cpu/decode.rs`、各拡張ごとの命令実行は `src/cpu/rv32*.rs` および `src/cpu/rv32*/exec.rs` に集約されています。

---

## 実行サイクル (Fetch-Execute)

エミュレータのメインループは以下の手順を繰り返します。

1.  **Fetch**: 現在の `pc` から命令を読み取ります。
    - 下位2ビットが `11` でなければ16ビット命令（C拡張）として扱います。
    - 32ビット命令の場合は `(inst_bin, 0)`、16ビット命令の場合は `(quadrant, inst16)` のような形式でフェッチされます。
2.  **Execute**: 命令バイナリを各拡張モジュール (`rv32i`, `rv32m`, `rv32c`) に振り分けて実行し、`regs` や `pc` を更新します。

```rust
impl Cpu {
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> StepResult {
        let (inst_bin, inst16) = self.fetch(bus);
        if inst16 == 0 {
            self.execute32(inst_bin, bus)
        } else {
            let quadrant = (inst16 & 0b11) as u16;
            self.execute16(inst16, quadrant, bus)
        }
    }
}
```

`StepResult` は、命令実行の結果（正常終了、ジャンプ発生、トラップ発生など）を呼び出し元に伝えるための列挙型です。

---

## CSR (Control and Status Registers) 管理

最小限のマシンモード用CSRを `src/cpu/csr.rs` で管理します。
- `mstatus`, `mie`, `mtvec`, `mepc`, `mcause`, `mtval` 等。

## メモリマップの想定

現状の `DefaultBus` は、指定されたサイズの単一メモリ領域として動作します。
将来的にファンタジーコンソールとして利用する際は、以下のようなメモリ領域の分割を可能にする予定です。
- `0x0000_0000`: ROM (プログラム領域)
- `0x8000_0000`: RAM (作業用領域)
- `0x4000_0000`: MMIO (VRAM, Audio, Input)

---

## テストとデバッグ

`overview.md` の方針に従い、以下の機能を実装します。

- **Register Dump**: `Cpu` 構造体に `dump_registers()` メソッドを実装。
- **Unit Testing**: 命令単位のテストを `src/cpu/tests/` 配下にカテゴリ別に分割して配置。共有のテスト用モック（`MockBus` 等）は `src/cpu/tests.rs` で定義。
