use rv32imc::bus::default_bus::DefaultBus;
use rv32imc::cpu::Cpu;
use std::env;
use rv32imc::cpu;

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

    loop {
         match cpu.step(&mut bus) {
             cpu::StepResult::Trap(code) => {
                 println!("Trap! (code: {code})");
                 break;
             },
            _ => (),
        }
    }

    println!("Final state:");
    println!("PC: {:#x}", cpu.pc);
    for (i, reg) in cpu.regs.iter().enumerate() {
        if *reg != 0 {
            println!("x{:02}: {:#010x}", i, reg);
        }
    }
}
