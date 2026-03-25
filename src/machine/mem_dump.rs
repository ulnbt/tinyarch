use super::Machine;

use std::fs::File;
use std::io::{Write, BufWriter};

fn write_vec_to_file(data: &[u32], filename: &str) -> std::io::Result<()> {
    let f = File::create(filename)?;
    let mut writer = BufWriter::new(f);

    // Calculate padding based on the highest index in hex
    let max_idx = if data.is_empty() { 0 } else { data.len() - 1 };
    let width = format!("{:X}", max_idx).len();

    let mut i = 0;
    while i < data.len() {
        if data[i] == 0 {
            // Find how many consecutive zeros we have
            let start = i;
            while i < data.len() && data[i] == 0 {
                i += 1;
            }
            let end = i - 1;

            if start == end {
                writeln!(writer, "[{:>width$X}]: 0", start, width = width)?;
            } else {
                writeln!(writer, "[{:>width$X}-{:>width$X}]: 0", start, end, width = width)?;
            }
        } else {
            // Standard hex output for non-zero values
            writeln!(writer, "[{:>width$X}]: {:08X}", i, data[i], width = width)?;
            i += 1;
        }
    }

    Ok(())
}

impl Machine {
    pub fn write_memory_to_file(&self, filename: &str) -> std::io::Result<()> {
        write_vec_to_file(&self.memory, filename)
    }
}