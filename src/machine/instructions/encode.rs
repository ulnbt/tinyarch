use super::Instruction;

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

impl Instruction {
    pub fn encode(&self) -> u32 {
        match self {
            Instruction::RawData(val) => *val,
            Instruction::SpHalt => 0,
            Instruction::SpDebug => 1,
            Instruction::BrCoIm { co, lhs, rhs, val } => {
                (0b01 << 30)
                    | (0b0 << 29)  // compare
                    | (0b0 << 28)  // immediate destination
                    | ((*co as u32 & 0b1111) << 24)
                    | ((*lhs as u32 & 0xf) << 20)
                    | ((*rhs as u32 & 0xf) << 16)
                    | (val & 0xffff)
            }
            Instruction::BrCoRg { co, lhs, rhs, dst } => {
                (0b01 << 30)
                    | (0b0 << 29)  // compare
                    | (0b1 << 28)  // register destination
                    | ((*co as u32 & 0b1111) << 24)
                    | ((*lhs as u32) << 16)
                    | ((*rhs as u32) << 8)
                    | (*dst as u32)
            },
            Instruction::BrUnconditional {dst} => (0b01 << 30) | (*dst & 0xfff_ffff),
            Instruction::OpIm { op, lhs, dst, rhs } => {
                (0b10 << 30)
                    | (0b0 << 29)
                    | ((*op as u32 & 0b11111) << 24)
                    | ((*lhs as u32 & 0xf) << 20)
                    | ((*dst as u32 & 0xf) << 16)
                    | (rhs & 0xffff)
            }
            Instruction::OpRg { op, lhs, rhs, dst } => {
                (0b10 << 30)
                    | (0b1 << 29)
                    | ((*op as u32 & 0b11111) << 24)
                    | ((*lhs as u32) << 16)
                    | ((*rhs as u32) << 8)
                    | (*dst as u32)
            }
            Instruction::MvRg { src, dst, src_m, dst_m } => {
                let src_m = *src_m as u32;
                let dst_m = *dst_m as u32;
                let n = gcd(src_m, dst_m);
                let i = src_m / n;
                let j = dst_m / n;
                assert!(n >= 1 && n <= 16, "n={} out of nibble range", n);
                assert!(i >= 1 && i <= 16, "i={} out of nibble range", i);
                assert!(j >= 1 && j <= 16, "j={} out of nibble range", j);
                (0b11 << 30)
                    | (0b00 << 28)
                    | ((*src as u32) << 20)
                    | ((*dst as u32) << 12)
                    | ((j - 1) << 8)
                    | ((i - 1) << 4)
                    | (n - 1)
            }
            Instruction::MvRd { dst, val } => {
                (0b11 << 30)
                    | (0b01 << 28)
                    | ((*dst as u32) << 20)
                    | (val & 0xffff)
            }
            Instruction::MvLd { dst, val } => {
                (0b11 << 30)
                    | (0b10 << 28)
                    | ((*dst as u32) << 20)
                    | (val & 0xffff)
            }
            Instruction::MvLf { dst, val } => {
                let sign_bit = (val >> 31) & 0b1;
                (0b11 << 30)
                    | (0b11 << 28)
                    | ((*dst as u32 & 0xf) << 24)
                    | (sign_bit << 23)
                    | (val & 0x7fffff)
            }

            Instruction::StPushRg { esc, src, src_val } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (0b00 << 24)              
                | ((*src as u32) << 16)
                | ((*src_val as u32) << 8),

            Instruction::StPushRgImm { esc, src, val } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (0b01 << 24)              
                | ((*src as u32) << 16)
                | (val & 0xfffff),

            Instruction::StPushPcI { esc, sparam, val } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (0b10 << 24)              
                | ((*sparam as u32) << 16)
                | (1 << 19)
                | (val & 0x7ffff),

            Instruction::StPushPcR { esc, sparam, dst } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (0b10 << 24)              
                | ((*sparam as u32) << 16)
                | ((*dst as u32) << 11),

            Instruction::StPushStI { esc, sparam, val } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (0b11 << 24)              
                | ((*sparam as u32) << 16)
                | (1 << 19)
                | (val & 0x7ffff),

            Instruction::StPushStR { esc, sparam, dst } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (0b11 << 24)              
                | ((*sparam as u32) << 16)
                | ((*dst as u32) << 11),

            Instruction::StPeek { esc, dst, offset_rg, offset } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (1 << 26)
                | (0b00 << 24)              
                | ((*dst as u32) << 16)
                | ((*offset_rg as u32) << 8)
                | (*offset as u32),

            Instruction::StPopRg { esc, dst } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (1 << 26)                 
                | (0b01 << 24)              
                | ((*dst as u32) << 16),

            Instruction::StPopPc { esc, sparam } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (1 << 26)                 
                | (0b10 << 24)              
                | ((*sparam as u32) << 16),

            Instruction::StPopSt { esc, sparam } =>
                0x3000_0000
                | ((*esc as u32) << 27)
                | (1 << 26)                 
                | (0b11 << 24)              
                | ((*sparam as u32) << 16),

            Instruction::SpStackPos { val } => 2 | ((val & 0xfffff) << 8),

            //Instruction::SpInt => todo!(),
            //Instruction::SpVec => todo!(),
            //Instruction::SpIntRet => todo!(),

            //Instruction::BrFlagIm { flag, val } => todo!(),
            //Instruction::BrFlagRg { flat, dst } => todo!(),
            //Instruction::SpNop => todo!(),
            //Instruction::SpWrith => todo!(),
            //Instruction::SpReath => todo!(),
            //Instruction::SpInith => todo!(),
            //Instruction::SpUmem => todo!(),
            //Instruction::SpUser => todo!(),

            _ => unimplemented!("encode not implemented for this instruction variant"),
        }
    }
}