
mod binop;
pub use binop::BinaryOperator;

mod condition;
pub use condition::Condition;

mod decode;
mod encode;
mod fmt;

mod conv;

#[derive(PartialEq)]
pub enum Instruction {
    RawData(u32),
    
    MvRg{src: u8, dst: u8, src_m: u8, dst_m: u8},
    MvRd{dst: u8, val: u32},
    MvLd{dst: u8, val: u32},
    MvLf{dst: u8, val: u32},
    
    OpIm{op: BinaryOperator, lhs: u8, dst: u8, rhs: u32},
    OpRg{op: BinaryOperator, lhs: u8, rhs: u8, dst: u8},

    BrCoIm{co: Condition, lhs: u8, rhs: u8, val: u32},
    BrCoRg{co: Condition, lhs: u8, rhs: u8, dst: u8},
    BrUnconditional{dst: u32},
    
    // Flag branching is not finalized:
    BrFlagIm{flag: u8, val: u32},
    BrFlagRg{flat: u8, dst: u8},

    // Unique
    SpHalt,
    SpNop,

    // Stack
    StPushRg { esc: bool, src: u8, src_val: u8 },
    StPushRgImm { esc: bool, src: u8, val: u32 },
    StPushPcI { esc: bool, sparam: u8, val: u32 },
    StPushPcR { esc: bool, sparam: u8, dst: u8 },
    StPushStI { esc: bool, sparam: u8, val: u32 },
    StPushStR { esc: bool, sparam: u8, dst: u8 },

    StPeek { esc: bool, dst: u8, offset_rg: u8, offset: u8 },
    StPopRg { esc: bool, dst: u8 },
    StPopPc { esc: bool, sparam: u8 },
    StPopSt { esc: bool, sparam: u8 },

    SpStackPos { val: u32 },


    //// To implement


    // Interupts
    SpInt,
    SpVec,
    SpIntRet, 

    // State manipulation 
    SpWrith,
    SpReath,
    SpInith,

    // 
    SpUmem,
    SpUser,


    // Debug
    SpDebug,
}
