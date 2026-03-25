//access.rs
use super::Machine;
use super::State;

impl Machine {
    // Access
    #[inline(always)]
    pub fn state(&mut self) -> &mut State {
        &mut self.states[self.current_ring]
    } 
    #[inline(always)]
    pub fn r_read(&mut self, r: u8) -> u32 {
        self.state().reg[r as usize]
    }
    #[inline(always)]
    pub fn r_rix_read(&mut self, r_rix: u8) -> u32 {
        self.r_rix_m_read(r_rix, 1)
    }
    #[inline(always)]
    pub fn r_rix_m_read(&mut self, r_rix: u8, m: u8) -> u32 {
        let r = r_rix >> 4;
        let rix = r_rix & 0xF;
        match rix {
            0 => self.state().reg[r as usize],
            _ => {
                let r = self.state().reg[r as usize];
                let rix = self.state().reg[rix as usize] * m as u32;
                self.read_address((r + rix) as usize)
            }
        }
    }

    #[inline(always)]
    pub fn r_write(&mut self, r: u8, val: u32) {
        self.state().reg[r as usize] = val
    }
    #[inline(always)]
    pub fn r_rix_write(&mut self, r_rix: u8, val: u32) {
        self.r_rix_m_write(r_rix, 1, val)
    }
    #[inline(always)]
    pub fn r_rix_m_write(&mut self, r_rix: u8, m: u8, val: u32) {
        let r = r_rix >> 4;
        let rix = r_rix & 0xF;
        match rix {
            0 => self.state().reg[r as usize] = val,
            _ => {
                let r = self.state().reg[r as usize];
                let rix = self.state().reg[rix as usize] * m as u32;
                let addr = (r + rix) as usize;
                self.write_address(addr, val);
            }
        }
    }
}