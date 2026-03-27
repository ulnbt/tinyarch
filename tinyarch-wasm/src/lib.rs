use wasm_bindgen::prelude::*;
use tinyarch::{assembler, machine::Machine};
use tinyarch::assembler::SimpleExecutableWriter;
use tinyarch::devices::buffer_output::BufferOutputDevice;

#[wasm_bindgen]
pub struct TinyArchWasm {
    machine: Machine,
    output: BufferOutputDevice,
    output_buf: String,
    cycle_count: u64,
    max_cycles: u64,
}

#[wasm_bindgen]
impl TinyArchWasm {
    /// Create a new machine with `ram_words` of RAM and a cycle budget of `max_cycles`.
    #[wasm_bindgen(constructor)]
    pub fn new(ram_words: usize, max_cycles: u64) -> TinyArchWasm {
        console_error_panic_hook::set_once();
        let mut machine = Machine::new(ram_words);
        let output = BufferOutputDevice::connect(&mut machine);
        TinyArchWasm {
            machine,
            output,
            output_buf: String::new(),
            cycle_count: 0,
            max_cycles,
        }
    }

    /// Assemble TinyArch source text into RAM, resetting all machine state.
    /// Returns an error string if assembly fails.
    pub fn assemble(&mut self, source: &str) -> Result<(), JsValue> {
        let ram_size = self.machine.memory.len();
        self.machine = Machine::new(ram_size);
        self.output = BufferOutputDevice::connect(&mut self.machine);
        self.output_buf.clear();
        self.cycle_count = 0;

        let mut writer = SimpleExecutableWriter::new(&mut self.machine, 0);
        assembler::assemble(source, &mut writer).map_err(|e| JsValue::from_str(&e))
        // writer drops here, releasing the mutable borrow on self.machine
    }

    /// Execute up to `n` instructions.
    ///
    /// MMIO output is drained after every step and accumulated internally —
    /// this is required for programs that spin-wait on MMIO acknowledgment
    /// (e.g. prime.ta). Retrieve accumulated text with `drain_output()`.
    ///
    /// Returns `true` if the machine is still running, `false` if it halted
    /// or the cycle budget was exceeded.
    pub fn step_n(&mut self, n: u32) -> bool {
        for _ in 0..n {
            if self.cycle_count >= self.max_cycles {
                return false;
            }
            let running = self.machine.step();
            self.cycle_count += 1;
            let chunk = self.output.drain();
            self.output_buf.push_str(&chunk);
            if !running {
                return false;
            }
        }
        true
    }

    /// Return and clear all buffered output text since the last call.
    pub fn drain_output(&mut self) -> String {
        std::mem::take(&mut self.output_buf)
    }

    /// True if the machine has executed a HALT instruction.
    pub fn is_halted(&self) -> bool {
        self.machine.halted
    }

    /// Total instruction cycles executed so far.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Current program counter.
    pub fn pc(&self) -> u32 {
        self.machine.states[self.machine.current_ring].pc
    }

    /// Flags register.
    pub fn flags(&self) -> u32 {
        self.machine.states[self.machine.current_ring].flags
    }

    /// Stack pointer.
    pub fn stack_ptr(&self) -> u32 {
        self.machine.states[self.machine.current_ring].stack
    }

    /// All 16 registers as a flat Vec<u32> (JS receives a Uint32Array).
    pub fn registers(&self) -> Vec<u32> {
        self.machine.states[self.machine.current_ring].reg.to_vec()
    }

    /// Read `len` words of RAM starting at word address `start`.
    /// Returns an empty vec if `start` is out of range.
    pub fn memory_range(&self, start: usize, len: usize) -> Vec<u32> {
        let mem_len = self.machine.memory.len();
        if start >= mem_len {
            return Vec::new();
        }
        let end = start.saturating_add(len).min(mem_len);
        self.machine.memory[start..end].to_vec()
    }
}
