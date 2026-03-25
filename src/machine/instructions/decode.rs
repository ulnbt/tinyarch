use super::{BinaryOperator, Condition, Instruction};

impl Instruction {
    pub fn decode(instruction: u32) -> Instruction {
        match (instruction >> 30) & 0b11 {
            0b00 => { // special
                match (instruction >> 28) & 0b11 {
                    0b00 => match instruction & 0xff { // UNFINISHED
                        0 => Instruction::SpHalt,
                        1 => Instruction::SpDebug,
                        2 => Instruction::SpStackPos { val: instruction >> 8 },
                        _ => unimplemented!()
                    },
                    0b10 => unimplemented!(),
                    0b01 => unimplemented!(),
                    0b11 => { // STACK INSTRUCTION!
                        let esc = (instruction >> 27) & 0b1 != 0;
                        let push = (instruction >> 26) & 0b1 == 0;
                        let i = (instruction >> 19) & 0b1 == 0;
                        let stype = (instruction >> 24) & 0b11;
                        match (push, stype, i) {
                            (true, 0b00, _) => Instruction::StPushRg
                                {esc, src: (instruction >> 16) as u8, src_val: (instruction >> 8) as u8 },
                            (true, 0b01, _) => Instruction::StPushRgImm 
                                { esc, src: (instruction >> 16) as u8, val: instruction & 0xfffff },
                            (true, 0b10, false) => Instruction::StPushPcI
                                { esc, sparam: (instruction >> 16) as u8, val: instruction & 0x7ffff },
                            (true, 0b10, true) => Instruction::StPushPcR
                                { esc, sparam: (instruction >> 16) as u8, dst: (instruction >> 11) as u8 },
                            (true, 0b11, false) => Instruction::StPushStI
                                { esc, sparam: (instruction >> 16) as u8, val: instruction & 0x7ffff },
                            (true, 0b11, true) => Instruction::StPushStR
                                { esc, sparam: (instruction >> 16) as u8, dst: (instruction >> 11) as u8 },
                            
                            (false, 0b00, _) => Instruction::StPeek
                                {esc, dst: (instruction >> 16) as u8, offset_rg: (instruction >> 8) as u8, offset: instruction as u8},
                            (false, 0b01, _) => Instruction::StPopRg 
                                { esc, dst: (instruction >> 16) as u8 },
                            (false, 0b10, _) => Instruction::StPopPc
                                { esc, sparam: (instruction >> 16) as u8 },
                            (false, 0b11, _) => Instruction::StPopSt
                                { esc, sparam: (instruction >> 16) as u8 },
                            _ => unreachable!()
                        }
                    }
                    _ => unreachable!(),
                }

            },
            0b01 => { // jumps
                if (instruction >> 29) & 0b1 == 0 { // compare
                    if (instruction >> 28) & 0b1 == 0 { // Immediate destination
                        Instruction::BrCoIm { 
                            co: Condition::decode((instruction >> 24) as u8 & 0b1111),
                            lhs: (instruction >> 20) as u8 & 0xf,
                            rhs: (instruction >> 16) as u8 & 0xf,
                            val: instruction & 0xffff,
                        }
                    } else {
                        Instruction::BrCoRg { // Register destination
                            co: Condition::decode((instruction >> 24) as u8 & 0b1111),
                            lhs: (instruction >> 16) as u8,
                            rhs: (instruction >> 8) as u8,
                            dst: instruction as u8,
                        }
                    }
                } else {
                    if (instruction >> 28) & 0b1 == 1 {
                        todo!()
                    } else {
                        Instruction::BrUnconditional { dst: instruction & 0xfff_ffff }
                    }
                }
            },
            0b10 => { // binop
                if (instruction >> 29) & 0b1 == 0 {
                    Instruction::OpIm { 
                        op: BinaryOperator::decode((instruction >> 24) as u8 & 0b11111),
                        lhs: (instruction >> 20) as u8 & 0xf,
                        dst: (instruction >> 16) as u8 & 0xf,
                        rhs: instruction & 0xffff,
                    }
                } else {
                    Instruction::OpRg { 
                        op: BinaryOperator::decode((instruction >> 24) as u8 & 0b11111),
                        lhs: (instruction >> 16) as u8,
                        rhs: (instruction >> 8) as u8,
                        dst: instruction as u8,
                    }
                }
            },
            0b11 => { // move
                match (instruction >> 28) & 0b11 {
                    0b00 => {
                        let n = (instruction & 0xf) + 1;
                        let i = ((instruction >> 4) & 0xf) + 1;
                        let j = ((instruction >> 8) & 0xf) + 1;
                        Instruction::MvRg { 
                            src: (instruction >> 20) as u8,
                            dst: (instruction >> 12) as u8, 
                            src_m: (n * i) as u8, 
                            dst_m: (n * j) as u8, 
                        }
                    },
                    0b01 => {
                        Instruction::MvRd { 
                            dst: (instruction >> 20) as u8,
                            val: instruction & 0xffff 
                        }
                    },
                    0b10 => {
                        Instruction::MvLd { 
                            dst: (instruction >> 20) as u8,
                            val: instruction & 0xffff 
                        }
                    },
                    0b11 => {
                        Instruction::MvLf { 
                            dst: ((instruction >> 24) & 0xf) as u8,
                            val: ((instruction >> 23 & 0b1) << 31) |  instruction & 0x7fffff
                        }
                    }
                    _ => unreachable!(),
                }
            },
            _ => unreachable!()
        }
    } 
} 