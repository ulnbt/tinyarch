
pub struct State {
    pub reg: [u32; 16],
    pub pc: u32,
    pub flags: u32,
    pub stack: u32,
    pub sparam: u8,
}

impl State {
    pub fn new() -> State {
        State {
            reg: [0; 16],
            pc: 0,
            flags: 0,
            stack: 0,
            sparam: 0,
        }
    }
    pub fn debug_print(&self) {
        println!("PC:{:x}, ST: {:x} ({:x}), REG: {:x?}", self.pc, self.stack, self.sparam, self.reg);
    }
}