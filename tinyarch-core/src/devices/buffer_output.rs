use crate::machine::{Machine, SharedMMIO};

const MMIO_ID: usize = 0x1;
const BUFFER_LEN: usize = 512;

/// A synchronous, thread-free replacement for `text_output` suitable for WASM.
///
/// The MMIO protocol is identical to `text_output`: slot 0 holds the character
/// count, slots 1..N hold ASCII values. Writing 0 back to slot 0 acknowledges
/// the write and unblocks programs that spin-wait on it (e.g. prime.ta).
pub struct BufferOutputDevice {
    mmio: SharedMMIO,
}

impl BufferOutputDevice {
    pub fn connect(machine: &mut Machine) -> Self {
        let mmio = machine.map_new_mmio(MMIO_ID, BUFFER_LEN);
        BufferOutputDevice { mmio }
    }

    /// Flush any characters the program has written to the MMIO buffer.
    /// Returns the text as a String and acknowledges the write (clears slot 0).
    /// Call this after every `step()` to avoid deadlocking programs that
    /// spin-wait on the acknowledgment.
    pub fn drain(&self) -> String {
        let count = self.mmio.read(0) as usize;
        if count == 0 {
            return String::new();
        }
        let mut text = String::with_capacity(count);
        for i in 0..count {
            text.push(self.mmio.read((i + 1) % BUFFER_LEN) as u8 as char);
        }
        self.mmio.write(0, 0);
        text
    }
}
