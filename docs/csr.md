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
    pub fn read(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            0x300 => Ok(self.mstatus),
            0x180 => Ok(self.satp),
            0x301 => Ok(0x40101104), // misa: RV32IMCU
            0x305 => Ok(self.mtvec),
            0x304 => Ok(self.mie),
            0x340 => Ok(self.mscratch),
            0x341 => Ok(self.mepc),
            0x342 => Ok(self.mcause),
            0x343 => Ok(self.mtval),
            0x344 => Ok(self.mip),
            0x3a0 => Ok(self.pmpcfg0),
            0x3b0..=0x3b3 => Ok(self.pmpaddr[(addr - 0x3b0) as usize]),
            // ... 他の CSR
            _ => Err(()),
        }
    }

    pub fn write(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        // 読み取り専用 CSR (bits 11-10 == 11) への書き込みは不正命令
        if (addr >> 10) & 0b11 == 0b11 {
            return Err(());
        }

        match addr {
            0x300 => {
                // mstatus の更新ロジック (MPP 等のバリデーション含む)
                let mask = 0x807E1888 | 0x0000000F;
                let mut val = val;
                let mpp = (val >> 11) & 0b11;
                if mpp == 1 || mpp == 2 {
                    // S/Hモードは未サポートのためUserモードに丸める
                    val &= !(0b11 << 11);
                }
                self.mstatus = (self.mstatus & !mask) | (val & mask);
                Ok(())
            }
            0x180 => { self.satp = val; Ok(()) }
            0x305 => { self.mtvec = val; Ok(()) }
            0x304 => { self.mie = val; Ok(()) }
            0x340 => { self.mscratch = val; Ok(()) }
            0x341 => { self.mepc = val; Ok(()) }
            0x342 => { self.mcause = val; Ok(()) }
            0x343 => { self.mtval = val; Ok(()) }
            0x344 => { self.mip = val; Ok(()) }
            0x3a0 => { self.pmpcfg0 = val; Ok(()) }
            0x3b0..=0x3b3 => {
                self.pmpaddr[(addr - 0x3b0) as usize] = val;
                Ok(())
            }
            _ => Err(()),
        }
    }
}
```

### 2.2 命令実行ロジック

CSR 命令は以下の共通したステップで処理します。

1.  **CSR アドレスの抽出**: 命令ビット `[31:20]` から 12ビットのアドレスを取得。
2.  **元の値の読み出し**: `csr.read(addr)` で現在の値を取得。
3.  **書き込み値の計算**:
    - `funct3` の最上位ビットが `1` なら `rs1` フィールドを 5ビットのゼロ拡張即値（uimm）として扱う。
    - `0` なら `regs[rs1]` の値を使用する。
    - `funct3` の下位 2ビットに基づき、`RW` (直接代入), `RS` (OR), `RC` (AND NOT) の演算を行う。
4.  **レジスタへの書き戻し**: `rd != 0` の場合、元の CSR の値を `regs[rd]` に格納。
5.  **CSR への書き込み**: `csr.write(addr, new_val)` を実行。
    - `mstatus`, `mtvec`, `mie`, `mepc`, `mcause`, `mtval`, `mip` 等が対象となります。
    - ただし、`CSRRS`/`CSRRC` で `rs1` が `x0` の場合、読み出しのみで書き込みは行われないという仕様に注意する。

### 2.3 特権レベルとアクセス制限

CSR アドレスの上位ビットには以下の意味があります。

- `addr[11:10]`: 読み書き制限 (11 は読み取り専用)
- `addr[9:8]`: 最小特権レベル (00: User, 01: Supervisor, 11: Machine)

現在の実装では、`addr[11:10] == 0b11` の領域（読み取り専用 CSR）への書き込みを試みると例外が発生します。
また、`misa` は `0x40101104` (RV32IMCU) を返し、User モードの存在をサポートしています。将来的に特権レベル（`self.mode`）に応じた厳密なアクセスチェックを追加可能です。

## 3. 実装のステップ

1.  `src/cpu/csr.rs` に `read` / `write` メソッドを追加する。
2.  `src/cpu.rs` の `execute` メソッド内、`0b1110011` (SYSTEM) のマッチアームに CSR 命令のロジックを追加する。
3.  テストコード (`src/cpu/tests/csr.rs`) を作成し、各命令の動作を検証する。
