use rv32imc::bus::default_bus::DefaultBus;
use rv32imc::cpu::Cpu;
use std::env;
use std::fs;
use std::path::Path;
use rv32imc::cpu;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <binary_file_or_directory>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);

    if path.is_dir() {
        run_all_tests(path);
    } else {
        match run_test(path) {
            Ok(success) => {
                if success {
                    println!("Result: SUCCESS");
                } else {
                    println!("Result: FAILED");
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn run_test(path: &Path) -> Result<bool, String> {
    let mut cpu = Cpu::new(0x0);
    let mut bus = DefaultBus::new(1024 * 1024); // 1MB

    bus.load_bin(path.to_str().unwrap(), 0)
        .map_err(|e| format!("Error loading binary: {}", e))?;

    let mut steps = 0;
    let max_steps = 1000000;

    loop {
        match cpu.step(&mut bus) {
            cpu::StepResult::Trap(_code) => {
                break;
            }
            _ => (),
        }
        steps += 1;
        if steps > max_steps {
            return Err("Timeout".to_string());
        }
    }

    let success = cpu.regs[3] == 1;

    if !success {
        if let Some(filename) = path.file_name() {
            println!("\nFinal state for {:?}:", filename);
        } else {
            println!("\nFinal state for {:?}:", path);
        }
        cpu.dump_registers();
    }

    Ok(success)
}

fn run_all_tests(dir: &Path) {
    let entries = fs::read_dir(dir).expect("Failed to read directory");
    let mut tests = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("bin") {
            tests.push(path);
        }
    }

    tests.sort();

    let mut success_count = 0;
    let total_count = tests.len();

    for test_path in &tests {
        print!("Running {:<40} ... ", test_path.file_name().unwrap().to_str().unwrap());
        match run_test(test_path) {
            Ok(true) => {
                println!("SUCCESS");
                success_count += 1;
            }
            Ok(false) => {
                println!("FAILED");
            }
            Err(e) => {
                println!("ERROR ({})", e);
            }
        }
    }

    println!("\nSummary: {}/{} tests passed", success_count, total_count);
}
