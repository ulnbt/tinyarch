// CLI use of the VM written by claude

use tinyarch::{assembler, machine, devices};

use std::{env, fs, thread};
use std::path::Path;
use std::time::{Duration, Instant};

const DEFAULT_RAM: usize = 128 * 1024;

fn print_help() {
    println!("\
Usage: tinyarch <file> [OPTIONS]

Arguments:
  <file>                   Path to the assembly file to execute

General:
  -?, --help               Show this help message and exit

Registers:
  -rN <value>              Set integer register N (hex 1-F) before execution.
                           <value> may be a 32-bit unsigned integer (positive)
                           or a 32-bit signed integer (negative). Zero is not
                           allowed. Examples: -r1 42  -rA 4294967295  -rF -1

  -frN <value>             Set float register N (hex 1-F) before execution.
                           <value> must be a 32-bit float.
                           Examples: -fr1 3.14  -frB -0.5  -frF 1e10

Output:
  -s                       Print the full machine state after execution.
                           This is shown automatically when no -p flags are set.

  -p rN                    Print a single register after execution in the form
                           'rN=<value>'. Accepts any register r1-rF. Multiple
                           -p flags may be combined. When any -p is present,
                           the full state (-s) is suppressed unless -s is also
                           explicitly set.
                           Examples: -p r1  -p r1 -p r2 -p rF

Debugging:
  -t, --time               Print execution time, CPU cycle count, and MHz rate
                           after the program finishes.

  -m, --memory <file>      Dump the full VM memory to <file> after execution.
                           Example: -m memory.txt  --memory dump.bin
");
}

struct Config {
    file_path: Option<String>,
    registers: Vec<(usize, u32)>, // (reg_index 1-15, raw bits)
    show_time: bool,
    memory_file: Option<String>,
    show_state: bool,
    print_regs: Vec<usize>, // reg indices to print at end
}

/// Parse a single hex digit '1'-'F'/'f' into a register index 1–15.
fn parse_reg_index(s: &str) -> Option<usize> {
    if s.len() != 1 {
        return None;
    }
    let val = s.chars().next()?.to_digit(16)? as usize;
    if val >= 1 { Some(val) } else { None } // 0 is excluded, 1-15 valid
}

/// Parse a 32-bit integer value. Negative → i32 bit-cast to u32. Zero is rejected.
fn parse_int_value(s: &str, flag: &str) -> u32 {
    if s.starts_with('-') {
        match s.parse::<i32>() {
            Ok(0) => {
                eprintln!("Error: value for {} cannot be zero", flag);
                std::process::exit(1);
            }
            Ok(v) => v as u32,
            Err(_) => {
                eprintln!("Error: '{}' is not a valid 32-bit signed integer for {}", s, flag);
                std::process::exit(1);
            }
        }
    } else {
        match s.parse::<u32>() {
            Ok(0) => {
                eprintln!("Error: value for {} cannot be zero", flag);
                std::process::exit(1);
            }
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: '{}' is not a valid 32-bit unsigned integer for {}", s, flag);
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    let mut config = Config {
        file_path: None,
        registers: Vec::new(),
        show_time: false,
        memory_file: None,
        show_state: false,
        print_regs: Vec::new(),
    };

    let mut i = 0;
    while i < raw_args.len() {
        let arg = &raw_args[i];

        // ── Boolean flags ────────────────────────────────────────────────────
        // At the top of the while loop, before all other branches:
        if arg == "--help" || arg == "-?" {
            print_help();
            std::process::exit(0);
        } else if arg == "--time" || arg == "-t" {
            config.show_time = true;

        } else if arg == "-s" {
            config.show_state = true;

        // ── Memory dump: -m <file> / --memory <file> ─────────────────────────
        } else if arg == "-m" || arg == "--memory" {
            i += 1;
            let next = raw_args.get(i).unwrap_or_else(|| {
                eprintln!("Error: {} requires a file name", arg);
                std::process::exit(1);
            });
            config.memory_file = Some(next.clone());

        // ── Print register: -p rN ─────────────────────────────────────────────
        } else if arg == "-p" {
            i += 1;
            let next = raw_args.get(i).unwrap_or_else(|| {
                eprintln!("Error: -p requires a register argument (e.g. r1, rA, rF)");
                std::process::exit(1);
            });
            let lower = next.to_lowercase();
            if !lower.starts_with('r') {
                eprintln!("Error: -p argument must be a register like r1–rF, got '{}'", next);
                std::process::exit(1);
            }
            match parse_reg_index(&lower[1..]) {
                Some(idx) => config.print_regs.push(idx),
                None => {
                    eprintln!("Error: invalid register '{}' for -p — must be r1–rF", next);
                    std::process::exit(1);
                }
            }

        // ── Float register: -frN <f32> ────────────────────────────────────────
        } else if arg.len() >= 4 && arg.to_lowercase().starts_with("-fr") {
            let hex_part = arg[3..].to_lowercase();
            let reg_idx = match parse_reg_index(&hex_part) {
                Some(idx) => idx,
                None => {
                    eprintln!("Error: invalid register in '{}' — must be -fr1 to -frF", arg);
                    std::process::exit(1);
                }
            };
            i += 1;
            let val_str = raw_args.get(i).unwrap_or_else(|| {
                eprintln!("Error: {} requires a 32-bit float value", arg);
                std::process::exit(1);
            });
            let fval: f32 = val_str.parse().unwrap_or_else(|_| {
                eprintln!("Error: '{}' is not a valid 32-bit float for {}", val_str, arg);
                std::process::exit(1);
            });
            config.registers.push((reg_idx, fval.to_bits()));

        // ── Integer register: -rN <u32/i32> ──────────────────────────────────
        } else if arg.len() >= 3 && arg.to_lowercase().starts_with("-r") {
            let hex_part = arg[2..].to_lowercase();
            let reg_idx = match parse_reg_index(&hex_part) {
                Some(idx) => idx,
                None => {
                    eprintln!("Error: invalid register in '{}' — must be -r1 to -rF", arg);
                    std::process::exit(1);
                }
            };
            i += 1;
            let val_str = raw_args.get(i).unwrap_or_else(|| {
                eprintln!("Error: {} requires a 32-bit integer value", arg);
                std::process::exit(1);
            });
            let value = parse_int_value(val_str, arg);
            config.registers.push((reg_idx, value));

        // ── Positional: file path ─────────────────────────────────────────────
        } else if !arg.starts_with('-') {
            if config.file_path.is_some() {
                eprintln!("Error: unexpected extra argument '{}'", arg);
                std::process::exit(1);
            }
            config.file_path = Some(arg.clone());

        } else {
            eprintln!("Error: unknown argument '{}'", arg);
            std::process::exit(1);
        }

        i += 1;
    }

    let file_path = config.file_path.clone().unwrap_or_else(|| {
        eprintln!("Error: expecting a file to execute");
        std::process::exit(1);
    });

    let path = Path::new(&file_path);
    if !path.exists() {
        eprintln!("Error: file '{}' does not exist", file_path);
        std::process::exit(1);
    }

    if let Err(err) = run_file(path, &config) {
        eprintln!("Error processing file '{}': {}", file_path, err);
        std::process::exit(1);
    }
}

fn run_file(path: &Path, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Read and assemble
    let assembly = fs::read_to_string(path)?;
    let mut machine = machine::Machine::new(DEFAULT_RAM);
    let mut writer = assembler::SimpleExecutableWriter::new(&mut machine, 0);
    assembler::assemble(&assembly, &mut writer)?;

    // Write CLI-supplied register values before execution
    for &(reg_idx, bits) in &config.registers {
        machine.state().reg[reg_idx] = bits;
    }

    // Connect device(s)
    devices::text_output::connect_device(&mut machine);

    // Execute
    let start = Instant::now();
    let mut cycles: usize = 0;
    while machine.step() { cycles += 1; }
    let duration = start.elapsed();
    thread::sleep(Duration::from_millis(10)); // let terminal thread flush output

    // ── Timing ───────────────────────────────────────────────────────────────
    if config.show_time {
        println!("execution time: {:?}", duration);
        println!("cpu cycles: {}", cycles);
        println!("cpu rate: {:.2}MHz", cycles as f64 / (duration.as_secs_f64() * 1_000_000f64));
    }

    // ── Memory dump ───────────────────────────────────────────────────────────
    if let Some(ref mem_file) = config.memory_file {
        machine.write_memory_to_file(mem_file).expect("could not dump VM memory");
    }

    // ── State / register output ───────────────────────────────────────────────
    // Show full state if: -s was given, OR no -p flags were provided at all.
    if config.show_state || config.print_regs.is_empty() {
        println!("final state:");
        machine.state().debug_print();
    }

    // Print individually requested registers (can coexist with -s).
    for &reg_idx in &config.print_regs {
        let label = char::from_digit(reg_idx as u32, 16).unwrap(); // '1'-'f'
        println!("r{}={}", label, machine.state().reg[reg_idx]);
    }

    Ok(())
}