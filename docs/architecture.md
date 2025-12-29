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
  ├── cpu.rs                    (Cpu構造体の定義とメインループ)
  ├── cpu/
  │    ├── decode.rs            (命令のデコードロジック)
  │    ├── execute.rs           (各命令の具体的な実行処理)
  │    ├── csr.rs               (CSR: 制御ステータスレジスタ関連)
  │    ├── privilege_mode.rs    (特権モードの定義)
  │    ├── tests.rs             (テスト用ヘルパーとモジュール定義)
  │    └── tests/               (命令カテゴリごとのユニットテスト)
  │         ├── lui.rs
  │         ├── auipc.rs
  │         └── ...
  ├── bus.rs                    (バス・トレイトの定義)
  └── lib.rs                    (クレートのルート)
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
    /// 特権モード (現在は Machine Mode のみ)
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
デコードロジックは `src/cpu/decode.rs`、各命令の実装は `src/cpu/execute.rs` に集約されています。

---

## 実行サイクル (Fetch-Execute)

エミュレータのメインループは以下の手順を繰り返します。

1.  **Fetch**: 現在の `pc` から命令を読み取ります。
    - 現状は 32ビット固定で読み込みを行っています。
    - 将来的な C拡張サポート時には、圧縮命令の判定（下位2ビットが `11` でなければ16ビット命令）を行う予定です。
2.  **Execute**: 命令バイナリを直接解釈し、処理を実行して `regs` や `pc` を更新します。

```rust
impl Cpu {
    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        let inst_bin = self.fetch(bus);
        self.execute(inst_bin, bus);
    }
}
```

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
- **Instruction Tracing**: `execute` 前に逆アセンブル結果をログ出力する機能。
- **Unit Testing**: 命令単位のテストを `src/cpu/tests/` 配下にカテゴリ別に分割して配置。共有のテスト用モック（`MockBus` 等）は `src/cpu/tests.rs` で定義。
