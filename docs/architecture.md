# アーキテクチャ設計書 (Architecture Design)

本ドキュメントでは、`docs/overview.md`に基づいたRV32IMCエミュレータの実装構造について記述します。

## 全体構成

エミュレータは、主に以下のコンポーネントで構成されます。

1.  **CPU Core**: レジスタ、PC、CSRの状態を管理し、命令の実行を制御します。
2.  **Memory Bus**: CPUと外部デバイス（メモリ、VRAM、入力など）を接続する抽象化レイヤーです。
3.  **Instruction Decoder**: フェッチしたバイナリを命令オブジェクトに変換します。
4.  **Execution Engine**: 命令をデコードし、CPUの状態を更新します。

---

## 主要な構造体とトレイト

### 1. CPU 構造体 (`Cpu`)

CPUの内部状態を保持します。

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

### 3. 命令列挙型 (`Instruction`)

デコードされた命令を表現します。RV32I, M, C 拡張をそれぞれカバーします。

```rust
pub enum Instruction {
    // RV32I
    Add(Rd, Rs1, Rs2),
    Sub(Rd, Rs1, Rs2),
    // ...
    // RV32M
    Mul(Rd, Rs1, Rs2),
    // ...
    // RV32C (内部的には対応するRV32I命令に展開するか、独自の型として保持)
    C_Addi(Rd, Imm),
}
```

---

## 実行サイクル (Fetch-Decode-Execute)

エミュレータのメインループは以下の手順を繰り返します。

1.  **Fetch**: 現在の `pc` から命令を読み取ります。
    - 圧縮命令 (C拡張) の判定（下位2ビットが `11` でなければ16ビット命令）。
2.  **Decode**: 命令バイナリを `Instruction` 列挙型に変換します。
3.  **Execute**: 命令に応じた処理を実行し、`regs` や `pc` を更新します。

```rust
impl Cpu {
    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        let inst_bin = self.fetch(bus);
        let inst = self.decode(inst_bin);
        self.execute(inst, bus);
    }
}
```

---

## CSR (Control and Status Registers) 管理

最小限のマシンモード用CSRを実装します。
- `mstatus`, `mie`, `mtvec`, `mepc`, `mcause`, `mtval` 等。

## メモリマップの想定

ファンタジーコンソールとしての利用を想定し、以下のようなメモリ領域の分割を可能にします。
- `0x0000_0000`: ROM (プログラム領域)
- `0x8000_0000`: RAM (作業用領域)
- `0x4000_0000`: MMIO (VRAM, Audio, Input)

---

## テストとデバッグ

`overview.md` の方針に従い、以下の機能を実装します。

- **Register Dump**: `Cpu` 構造体に `dump_registers()` メソッドを実装。
- **Instruction Tracing**: `execute` 前に逆アセンブル結果をログ出力する機能。
- **Unit Testing**: 命令単位のテストを `src/` 内の各モジュールに配置。
