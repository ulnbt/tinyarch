use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicU32, Ordering};
use super::Machine;

/// Represents a slice of memory shared between CPU and Devices
#[derive(Clone)]
pub struct SharedMMIO {
    pub data: Arc<[AtomicU32]>,
}

impl SharedMMIO {
    pub fn new(size: usize) -> Self {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(AtomicU32::new(0));
        }
        Self { data: v.into() }
    }

    pub fn read(&self, addr: usize) -> u32 {
        self.data[addr].load(Ordering::Relaxed)
    }

    pub fn write(&self, addr: usize, val: u32) {
        self.data[addr].store(val, Ordering::Relaxed);
    }
}

impl Machine {
    pub fn map_new_mmio(&mut self, id: usize, size: usize) -> SharedMMIO {
        assert!(id < 0xf && id > 0, "Hardware MMIO ID must be 1-15");
        assert!(self.mmio_devices[id].is_none(), "Hardware MMIO ID is already in use!");
        let mmio = SharedMMIO::new(size);
        self.mmio_devices[id] = Some(mmio.clone());
        mmio
    }

    pub fn get_interrupt_channel(&mut self) -> mpsc::Sender<u32> {
        self.internal_interrupt.clone()
    }
}