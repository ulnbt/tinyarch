use std::{thread, time::Duration};
use crate::machine;

pub fn connect_device(vm: &mut machine::Machine) {
    const BUFFER_LEN: usize = 512;
    let mem = vm.map_new_mmio(0x1, BUFFER_LEN); // use hw id 0x1
    //let int = vm.get_interrupt_channel(); // for input later
    thread::spawn(move || {
        let mut text = String::new();
        loop {
            thread::sleep(Duration::from_millis(5)); // simple polling for now
            let next_read = mem.read(0) as usize;
            if next_read != 0 {
                text.clear();
                for i in 0..next_read {
                    text.push(mem.read((i + 1) % BUFFER_LEN) as u8 as char);
                }
                mem.write(0, 0);
                print!("{}", text);
            }
        }
    });
}