# トラップ処理 (Trap Handling) 仕様

本エミュレータにおける RISC-V のトラップ（例外および割り込み）処理の実装方針について記述します。

## 1. トラップの種類とコード

### 例外 (Exceptions)
`mcause` の最上位ビットが `0` の場合です。

| 例外コード (mcause) | 例外名 (Exception Name) | 説明 |
| :--- | :--- | :--- |
| 2 | Illegal Instruction | 不正命令 |
| 8 | Environment call from U-mode | ユーザーモードからのシステムコール |
| 9 | Environment call from S-mode | スーパーバイザーモードからのシステムコール |
| 11 | Environment call from M-mode | マシンモードからのシステムコール |

### 割り込み (Interrupts)
`mcause` の最上位ビットが `1` の場合です。

| 例外コード (mcause) | 割り込み名 (Interrupt Name) | 説明 |
| :--- | :--- | :--- |
| 0x80000003 | Machine Software Interrupt | マシンモード・ソフトウェア割り込み (CLINT) |
| 0x80000007 | Machine Timer Interrupt | マシンモード・タイマー割り込み (CLINT) |
| 0x8000000b | Machine External Interrupt | マシンモード・外部割り込み (PLIC) |

## 2. トラップ発生時の動作 (Hardware/Emulator side)

例外や割り込みが発生した際、プロセッサ（エミュレータ）は以下の処理をアトミックに実行します。

1.  **`mepc` の更新**:
    - 例外が発生した命令、または割り込みによって中断された命令の PC を `mepc` CSR に保存します。
2.  **`mcause` および `mtval` の更新**:
    - 発生したトラップに応じた例外コードを `mcause` に、補助情報を `mtval` に書き込みます。
3.  **`mstatus` の更新**:
    - 現在の特権モードを `MPP` フィールドに、割り込み許可状態 (`MIE`) を `MPIE` に保存します。
    - `MIE` を 0（無効）にし、トラップ処理中の割り込みを禁止します。
4.  **特権モードの遷移**:
    - 現在の特権モードを **Machine Mode** に変更します。
5.  **PC のジャンプ**:
    - `mtvec` CSR の設定（Direct または Vectored）に従い、トラップハンドラのアドレスへジャンプします。

## 3. トラップからの復帰 (MRET 命令)

トラップハンドラの処理が完了し、元のプログラムに戻る際は `MRET` 命令を使用します。

*   **動作**:
    - PC を `mepc` の値に復帰させます。
    - `mstatus` の `MPIE` の値を `MIE` に戻します。
    - 特権モードを `MPP` に保存されていた値に戻します。

## 4. 実装状況

`Cpu` 構造体にトラップ処理を一括して行う `handle_trap` メソッドが実装されています。

```rust
impl Cpu {
    pub(super) fn handle_trap(&mut self, exception_code: u32, mtval: u32) -> StepResult {
        // 1. mepc に現在の PC を保存
        self.csr.mepc = self.pc;

        // 2. mcause, mtval の更新
        self.csr.mcause = exception_code;
        self.csr.mtval = mtval;

        // 3. mstatus の更新 (MPP, MPIE, MIE)
        let mie = (self.csr.mstatus >> 3) & 1;
        self.csr.mstatus &= !(1 << 7); // MPIE = 0
        self.csr.mstatus |= mie << 7;  // MPIE = MIE
        self.csr.mstatus &= !(1 << 3); // MIE = 0

        let mpp = self.mode as u32;
        self.csr.mstatus &= !(0b11 << 11); // MPP = 0
        self.csr.mstatus |= mpp << 11;     // MPP = mode

        // 4. 特権モードを Machine に遷移
        self.mode = PrivilegeMode::Machine;

        // 5. mtvec の設定に従ってジャンプ
        let is_interrupt = (exception_code >> 31) & 1;
        let mtvec_mode = self.csr.mtvec & 0b11;
        let mtvec_base = self.csr.mtvec & !0b11;

        if is_interrupt == 1 && mtvec_mode == 1 {
            // Vectored mode
            let code = exception_code & 0x7fff_ffff;
            self.pc = mtvec_base + 4 * code;
        } else {
            // Direct mode or exception
            self.pc = mtvec_base;
        }

        StepResult::Trap(exception_code)
    }
}
```
