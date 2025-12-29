# 例外処理 (Exception Handling) 仕様

本エミュレータにおける RISC-V 例外処理の実装方針について記述します。当面はマシンモード（Machine Mode）での例外処理を中心に実装します。

## 1. 例外の種類とコード

`ECALL` 命令が実行された際、現在の特権モードに応じて以下の例外が発生します。これらは `mcause` レジスタに格納されます。

| 例外コード (mcause) | 例外名 (Exception Name) | 説明 |
| :--- | :--- | :--- |
| 8 | Environment call from U-mode | ユーザーモードからのシステムコール |
| 9 | Environment call from S-mode | スーパーバイザーモードからのシステムコール |
| 11 | Environment call from M-mode | マシンモードからのシステムコール |

## 2. トラップ発生時の動作 (Hardware/Emulator side)

`ECALL` や `EBREAK` などの例外（トラップ）が発生した際、プロセッサ（エミュレータ）は以下の処理をアトミックに実行します。

1.  **`mepc` の更新**:
    - 例外が発生した命令の PC（`ECALL` 自身のアドレス）を `mepc` CSR に保存します。
2.  **`mcause` の更新**:
    - 発生した例外に対応する例外コード（例: Mモードからの `ECALL` なら 11）を `mcause` CSR に書き込みます。
3.  **`mstatus` の更新**:
    - 現在の特権モードを `MPP` (Machine Previous Privilege) フィールドに保存します。
    - 現在の割り込み許可状態 (`MIE`) を `MPIE` (Machine Previous Interrupt Enable) に保存します。
    - `MIE` を 0（無効）にし、トラップ処理中の割り込みを禁止します。
4.  **特権モードの遷移**:
    - 現在の特権モードを **Machine Mode** に変更します（トラップ先が M モードの場合）。
5.  **PC のジャンプ**:
    - プログラムカウンタ（PC）を `mtvec` CSR に設定されているトラップハンドラのアドレスに書き換えます。

## 3. トラップからの復帰 (MRET 命令)

トラップハンドラ（OS やファームウェア）の処理が完了し、元のプログラムに戻る際は `MRET` 命令を使用します。

*   **動作**:
    - PC を `mepc` の値に復帰させます。
    - `mstatus` の `MPIE` の値を `MIE` に戻します。
    - 特権モードを `MPP` に保存されていた値に戻します。
*   **注意点**: `ECALL` から復帰する場合、そのまま戻ると再び `ECALL` が実行されるため、通常はトラップハンドラ内で `mepc` を +4 しておく必要があります。

## 4. 実装状況

`Cpu` 構造体にトラップ処理を一括して行う `handle_trap` メソッドが実装されています。

```rust
impl Cpu {
    pub(super) fn handle_trap(&mut self, exception_code: u32) -> StepResult {
        // 1. mepc に現在の PC を保存(復帰時に再度 ECALL が呼ばれないように PC に +4 しておく)
        self.csr.mepc = self.pc + 4;

        // 2. mcause に例外コードを設定
        self.csr.mcause = exception_code;

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

        // 5. mtvec のアドレスへジャンプ
        self.pc = self.csr.mtvec;

        StepResult::Trap(exception_code)
    }
}
```
このように、本エミュレータでは RISC-V の特権アーキテクチャ仕様に準拠した形で例外を再現します。
