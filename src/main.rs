use tinyarch::{assembler, machine, devices};

use std::{env, fs, thread};
use std::path::Path;
use std::time::{Duration, Instant};

const DEFAULT_RAM: usize = 128 * 1024;

fn main() {
    let mut args = env::args().skip(1);

    if let Some(file_path) = args.next() {
        let path = Path::new(&file_path);

        if !path.exists() {
            eprintln!("Error: file '{}' does not exist", file_path);
            std::process::exit(1);
        }

        if let Err(err) = run_file(path) {
            eprintln!("Error processing file '{}': {}", file_path, err);
            std::process::exit(1);
        }
    } else {
        eprintln!("Expecting file to execute");
    }
}


fn run_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Read file
    let assembly = fs::read_to_string(path)?;

    // Initialize machine, load assembly 
    let mut machine = machine::Machine::new(DEFAULT_RAM);
    let mut writer = assembler::SimpleExecutableWriter::new(&mut machine, 0);
    assembler::assemble(&assembly, &mut writer)?;

    // Connect device(s)
    devices::text_output::connect_device(&mut machine);

    // Performance counter
    let start = Instant::now();
    let mut cycles: usize = 0;

    // Execute program
    while machine.step() { cycles += 1; }

    let duration = start.elapsed();
    thread::sleep(Duration::from_millis(10)); // wait a little for the terminal thread to output


    // Output performance metrics, final state.
    println!("execution time: {:?}", duration);
    println!("cpu cycles: {:?}", cycles);
    println!("cpu rate: {:.2?}MHz", cycles as f64 / (duration.as_secs_f64() * 1000_000f64));
    println!("final state:");
    machine.state().debug_print();
    
    // Uncomment this to write memory into text file
    //machine.write_memory_to_file("memory.txt").expect("could not dump vm memory"); 
    
    Ok(())
}
 