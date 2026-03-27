use std::sync::mpsc;

mod instructions;
pub use instructions::{BinaryOperator, Condition, Instruction};

mod state;
pub use state::State;

mod interupts;
pub use interupts::Interupt;

mod mmio;
pub use mmio::SharedMMIO;

mod access;
mod execute;

mod mem_dump;

pub struct Machine {
    pub memory: Vec<u32>,
    pub current_ring: usize,
    pub states: [State; 3],
    pub halted: bool,

    pub mmio_devices: [Option<SharedMMIO>; 16],
    pub interrupt: mpsc::Receiver<u32>,
    pub internal_interrupt: mpsc::Sender<u32>,
}

impl Machine {
    pub fn new(memory: usize) -> Machine {
        let (s, r) = mpsc::channel::<u32>();
        Machine { 
            memory: vec![0; memory], 
            current_ring: 0, 
            states: [State::new(), State::new(), State::new()], 
            halted: false, 

            mmio_devices: std::array::from_fn(|_| None),
            interrupt: r,
            internal_interrupt: s,
        }
    }

    pub fn read_address(&mut self, address: usize) -> u32 {
        let id = address >> 28;
        if id == 0 { self.memory[address] }
        else if let Some(mmio) = &self.mmio_devices[id] {
            mmio.read(address & 0xfff_ffff)
        } else { 0 }
    }
    pub fn write_address(&mut self, address: usize, val: u32) {
        let id = address >> 28;
        if id == 0 { self.memory[address] = val }
        else if let Some(mmio) = &self.mmio_devices[id] {
            mmio.write(address & 0xfff_ffff, val)
        } else { }
    }


    pub fn step(&mut self) -> bool {
        let addr = self.state().pc;
        self.state().pc += 1;
        let instr = Instruction::decode(self.memory[addr as usize]);
        self.exec(instr);
        !self.halted
    }

    /// I do not recommend using this.
    pub(crate) fn step_debug(&mut self, debug: &mut bool) -> bool {
        let addr = self.state().pc;
        self.state().pc += 1;
        let instr = Instruction::decode(self.memory[addr as usize]);
        if instr == Instruction::SpDebug { *debug = true; }
        if *debug {
            self.state().debug_print();
            print!("[{:x}] {}", addr, instr);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            std::io::Read::read(&mut std::io::stdin(), &mut [0, 0]).unwrap();
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        self.exec(instr);
        // Shouldn't ever happen but just double checking, i should make this physically impossible
        if self.state().reg[0] != 0 { println!("WOAH BOY HOL UP WHAT A BIG ERROR!!!") }
        !self.halted
    }
}

