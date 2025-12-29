# CSR (Control and Status Register) 命令の実装仕様

本エミュレータにおける RISC-V の CSR 系命令（CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI）の実装方針について記述します。

## 1. 実装対象の命令

RV32I で定義されている以下の 6 つの命令を実装します。これらはすべて `SYSTEM` オコード (`0b1110011`) を持ちます。

| 命令 | funct3 | 動作 |
| :--- | :--- | :--- |
| **CSRRW** | `001` | CSR をレジスタと入れ替える (Atomic Read/Write) |
| **CSRRS** | `010` | CSR のビットをセットする (Atomic Read and Set Bits) |
| **CSRRC** | `011` | CSR のビットをクリアする (Atomic Read and Clear Bits) |
| **CSRRWI** | `101` | 即値で CSR を入れ替える |
| **CSRRSI** | `110` | 即値で CSR のビットをセットする |
| **CSRRCI** | `111` | 即値で CSR のビットをクリアする |

## 2. 内部実装方針

### 2.1 CSR アドレス管理 (`src/cpu/csr.rs`)

`Csr` 構造体に、12ビットのアドレスを指定して読み書きを行うための共通メソッドが実装されています。

```rust
impl Csr {
    pub fn read(&self, addr: u32) -> u32 {
        match addr {
            0x300 => self.mstatus,
            0x305 => self.mtvec,
            0x304 => self.mie,
            0x341 => self.mepc,
            0x342 => self.mcause,
            0x343 => self.mtval,
            0x344 => self.mip,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u32, val: u32) {
        match addr {
            0x300 => self.mstatus = val,
            0x305 => self.mtvec = val,
            0x304 => self.mie = val,
            0x341 => self.mepc = val,
            0x342 => self.mcause = val,
            0x343 => self.mtval = val,
            0x344 => self.mip = val,
            _ => {}
        }
    }
}
```

### 2.2 命令実行ロジック (`src/cpu.rs`)

CSR 命令は以下の共通したステップで処理します。

1.  **CSR アドレスの抽出**: 命令ビット `[31:20]` から 12ビットのアドレスを取得。
2.  **元の値の読み出し**: `csr.read(addr)` で現在の値を取得。
3.  **書き込み値の計算**:
    - `funct3` の最上位ビットが `1` なら `rs1` フィールドを 5ビットのゼロ拡張即値（uimm）として扱う。
    - `0` なら `regs[rs1]` の値を使用する。
    - `funct3` の下位 2ビットに基づき、`RW` (直接代入), `RS` (OR), `RC` (AND NOT) の演算を行う。
4.  **レジスタへの書き戻し**: `rd != 0` の場合、元の CSR の値を `regs[rd]` に格納。
5.  **CSR への書き込み**: `csr.write(addr, new_val)` を実行。
    - ただし、`CSRRS`/`CSRRC` で `rs1` が `x0` の場合、読み出しのみで書き込みは行われないという仕様に注意する。

### 2.3 特権レベルとアクセス制限 (将来的な拡張)

CSR アドレスの上位ビットには以下の意味があります。
- `addr[11:10]`: 読み書き制限 (11 は読み書き可能、11 以外の一部は読み取り専用)
- `addr[9:8]`: 最小特権レベル (00: User, 01: Supervisor, 11: Machine)

現在は簡易実装のため、特権チェックは行わずに全てのアクセスを許可しますが、将来的に `self.mode` と比較して違法命令例外を投げるように拡張可能です。

## 3. 実装のステップ

1.  `src/cpu/csr.rs` に `read` / `write` メソッドを追加する。
2.  `src/cpu.rs` の `execute` メソッド内、`0b1110011` (SYSTEM) のマッチアームに CSR 命令のロジックを追加する。
3.  テストコード (`src/cpu/tests/csr.rs`) を作成し、各命令の動作を検証する。
