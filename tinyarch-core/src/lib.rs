//! # TinyArch v2
//! TinyArch v2 is an emulation first, idealistic, simple ISA.
//! 
//! This documentation is very work in progress.
//! 
//! ### Example of using tinyarch as a library:
//!
//! ```ignore
//!use tinyarch::{assembler, devices, machine};
//!
//!fn main() {
//!    // Create new machine instance with 1kb RAM.
//!    let mut machine = machine::Machine::new(1024);
//!
//!    // Assemble a program into RAM.
//!    let mut writer = assembler::SimpleExecutableWriter::new(&mut machine, 0);
//!    if let Err(msg) = assembler::assemble(include_str!("program.ta"), &mut writer) {
//!        panic!("{}", msg);
//!    }
//!    
//!    // Connect to terminal output
//!    devices::text_output::connect_device(&mut machine);
//!
//!    // Run the machine until it halts
//!    while machine.step() { }
//!
//!    // Print some of its state, output its memory
//!    machine.state().debug_print();
//!    machine.write_memory_to_file("RAM.txt").expect("could not dump vm memory"); 
//!}
//! ```

/// The emulator itself, and means to interact with it, instructions, memory mapped IO, etc.
pub mod machine;
/// Some simple IO devices for the emulator to use.
pub mod devices;
/// Means to assemble and parse instructions into machine code, means to implement linkers.
pub mod assembler;