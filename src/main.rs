use rv32imc::bus::default_bus::DefaultBus;
use rv32imc::cpu::Cpu;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <binary_file>", args[0]);
        return;
    }

    let bin_path = &args[1];
    let mut cpu = Cpu::new(0x0);
    let mut bus = DefaultBus::new(1024 * 1024); // 1MB

    if let Err(e) = bus.load_bin(bin_path, 0) {
        eprintln!("Error loading binary: {}", e);
        return;
    }

    println!("Starting execution of {}...", bin_path);

    let mut step_count = 0;
    loop {
        cpu.step(&mut bus);
        step_count += 1;

        // 終了条件の例: 特定のメモリアドレスへの書き込みや無限ループの検知
        // ここでは、一旦 10000 ステップで停止するようにしておきます
        if step_count > 10000 {
            println!("Execution reached step limit.");
            break;
        }

        // riscv-tests の終了条件（多くのテストでは特定の命令で停止するか
        // ecall を投げたりしますが、ここでは簡易的に PC の変化がなくなったら停止など検討の余地あり）
        // とりあえず、デバッグ用にレジスタをダンプするなどの機能も欲しくなるかもしれません。
    }

    println!("Final state:");
    println!("PC: {:#x}", cpu.pc);
    for (i, reg) in cpu.regs.iter().enumerate() {
        if *reg != 0 {
            println!("x{:02}: {:#010x}", i, reg);
        }
    }
}
