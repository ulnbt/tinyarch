use super::Instruction;
use super::Machine;


impl Machine {
    pub fn exec(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::RawData(_) => panic!("Attempting to execute invalid instruction! (decoded as data??)"),
            Instruction::MvRg { src, dst, src_m, dst_m } => {
                if dst == 0 {
                    let a = src & 0xf;
                    let b = (src >> 4) & 0xf;
                    if a != 0 && b != 0 {
                        let val_a = self.r_read(a);
                        let val_b = self.r_read(b);
                        self.r_write(a, val_b);
                        self.r_write(b, val_a);
                    }
                } else {
                    let v = self.r_rix_m_read(src, src_m);
                    self.r_rix_m_write(dst, dst_m, v);
                }
            },
            Instruction::MvRd { dst, val } => {
                let val = self.read_address(val as usize);
                self.r_rix_write(dst, val)
            },
            Instruction::MvLd { dst, val } |
            Instruction::MvLf { dst, val } => self.r_rix_write(dst, val),
            Instruction::OpIm { op, lhs, dst, rhs } => {
                if dst != 0 {
                    let lhs = self.r_read(lhs);
                    self.r_write(dst, op.compute(lhs, rhs));
                }
            },
            Instruction::OpRg { op, lhs, rhs, dst } => {
                if dst != 0 {
                    let lhs = self.r_rix_read(lhs);
                    let rhs = self.r_rix_read(rhs);
                    self.r_rix_write(dst, op.compute(lhs, rhs))
                }
            },
            Instruction::BrCoIm { co, lhs, rhs, val } => {
                let lhs = self.r_read(lhs);
                let rhs = self.r_read(rhs);
                if co.compare(lhs, rhs) {
                    self.state().pc = val;
                }
            },
            Instruction::BrCoRg { co, lhs, rhs, dst } => {
                let lhs = self.r_rix_read(lhs);
                let rhs = self.r_rix_read(rhs);
                if co.compare(lhs, rhs) {
                    self.state().pc = self.r_rix_read(dst);
                }
            },
            Instruction::BrFlagIm { flag, val } => unimplemented!(),
            Instruction::BrFlagRg { flat, dst } => unimplemented!(),
            Instruction::BrUnconditional { dst } => {
                self.state().pc = dst;
            },

            // stack!
            Instruction::StPushRg { esc, src, src_val } => todo!(),
            Instruction::StPushRgImm { esc, src, val } => todo!(),
            Instruction::StPushPcI { esc, sparam, val } => todo!(),
            Instruction::StPushPcR { esc, sparam, dst } => todo!(),
            Instruction::StPushStI { esc, sparam, val } => todo!(),
            Instruction::StPushStR { esc, sparam, dst } => todo!(),
            Instruction::StPeek { esc, dst, offset_rg, offset } => todo!(),
            Instruction::StPopRg { esc, dst } => todo!(),
            Instruction::StPopPc { esc, sparam } => todo!(),
            Instruction::StPopSt { esc, sparam } => todo!(),
            Instruction::SpStackPos { val } => self.state().stack = val,
            

            Instruction::SpHalt => {
                self.halted = true;
            },
            Instruction::SpNop => todo!(),
            Instruction::SpInt => todo!(),
            Instruction::SpIntRet => todo!(),
            Instruction::SpVec => todo!(),
            Instruction::SpWrith => todo!(),
            Instruction::SpReath => todo!(),
            Instruction::SpInith => todo!(),
            Instruction::SpUmem => todo!(),
            Instruction::SpUser => todo!(),
            Instruction::SpDebug => {
                println!("DEBUG!");
            },
            
            
            
        }
    }
}