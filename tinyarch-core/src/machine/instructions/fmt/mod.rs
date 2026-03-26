// implements fmt for Instruction enum

mod fmt_binop;
mod fmt_cond;
mod fmt_reg;

use std::fmt;
use super::{BinaryOperator, Condition, Instruction};
use fmt_reg::{reg_name, r_rix_name};


impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::RawData(val) => write!(f, "DAT {val:#x}"),

            Instruction::MvRg { src, dst, src_m, dst_m } =>
                write!(f, "{} -> {}", r_rix_name(*src, *src_m), r_rix_name(*dst, *dst_m)),

            Instruction::MvRd { dst, val } =>
                write!(f, "@{:#x} -> {}", val, r_rix_name(*dst, 1)),

            Instruction::MvLd { dst, val } =>
                write!(f, "{:#x} -> {}", val, r_rix_name(*dst, 1)),

            // (behaviour might change!)
            Instruction::MvLf { dst, val } =>
                write!(f, "{} -> r{:X}", f32::from_bits(*val), dst),

            Instruction::OpIm { op, lhs, dst, rhs } =>
                write!(f, "{} {} {:#x} -> {}", reg_name(*lhs), op, rhs, reg_name(*dst)),

            Instruction::OpRg { op, lhs, rhs, dst } =>
                write!(f, "{} {} {} -> {}",
                    r_rix_name(*lhs, 1), op, r_rix_name(*rhs, 1), r_rix_name(*dst, 1)),

            Instruction::BrCoIm { co, lhs, rhs, val } =>
                write!(f, "if {} {} {} jmp {:#x}", reg_name(*lhs), co, reg_name(*rhs), val),

            // Register: all three are r_rix bytes.
            Instruction::BrCoRg { co, lhs, rhs, dst } =>
                write!(f, "if {} {} {} jmp {}",
                    r_rix_name(*lhs, 1), co, r_rix_name(*rhs, 1), r_rix_name(*dst, 1)),

            // Flag-based branches (NOT finalized)
            Instruction::BrFlagIm { flag, val } =>
                write!(f, "if flag({}) jmp {:#x}", flag, val),
            Instruction::BrFlagRg { flat, dst } =>
                write!(f, "if flag({}) jmp {}", flat, reg_name(*dst)),
            
            Instruction::BrUnconditional { dst } =>
                write!(f, "jmp {:#x}", *dst),

            // Stack
            Instruction::SpStackPos { val } => write!(f, "stack {val}"),

            Instruction::StPushRg { esc, src, src_val } => 
                if *src_val == 0 { write!(f, "push {}", r_rix_name(*src, 1)) }
                else { write!(f, "push {} <- {}", r_rix_name(*src, 1), r_rix_name(*src_val, 1)) },
            Instruction::StPushRgImm { esc, src, val } =>
                write!(f, "push {} <- {val:#x}", r_rix_name(*src, 1)),
            Instruction::StPushPcI { esc, sparam, val } => 
                if *sparam == 0 { write!(f, "sjmp {val:#x}") } else { write!(f, "sjmp {val:#x} ({sparam})") },
            Instruction::StPushPcR { esc, sparam, dst } =>
                if *sparam == 0 { write!(f, "sjmp {}", r_rix_name(*dst, 1)) } else { write!(f, "sjmp {} ({sparam})", r_rix_name(*dst, 1)) },
            Instruction::StPushStI { esc, sparam, val } => 
                if *sparam == 0 { write!(f, "call {val:#x}") } else { write!(f, "call {val:#x} ({sparam})") },
            Instruction::StPushStR { esc, sparam, dst } =>
                if *sparam == 0 { write!(f, "call {}", r_rix_name(*dst, 1)) } else { write!(f, "call {} ({sparam})", r_rix_name(*dst, 1)) },

            Instruction::StPeek { esc, dst, offset_rg, offset } => {
                if *offset_rg == 0 && *offset == 0 { 
                    write!(f, "peek {} + {} -> {}", r_rix_name(*offset_rg, 1), *offset, r_rix_name(*dst, 1))  
                } else if *offset == 0 { 
                    write!(f, "peek {} -> {}", r_rix_name(*offset_rg, 1), r_rix_name(*dst, 1))
                } else if *offset_rg == 0 {
                    write!(f, "peek {} -> {}", *offset, r_rix_name(*dst, 1))
                } else {
                    write!(f, "peek {}", r_rix_name(*dst, 1))
                }
            }
            Instruction::StPopRg { esc, dst } => write!(f, "pop {}", r_rix_name(*dst, 1)),
            Instruction::StPopPc { esc, sparam } => if *sparam == 0 {write!(f, "sret")} else {write!(f, "sret ({sparam})")},
            Instruction::StPopSt { esc, sparam } => if *sparam == 0 {write!(f, "ret")} else {write!(f, "ret ({sparam})")},

            // other special
            Instruction::SpHalt          => write!(f, "halt"),
            Instruction::SpNop           => write!(f, "nop"),

            Instruction::SpIntRet        => write!(f, "intret"),
            Instruction::SpInt           => write!(f, "int"),
            Instruction::SpVec           => write!(f, "vec"),
            Instruction::SpWrith         => write!(f, "writh"),
            Instruction::SpReath         => write!(f, "reath"),
            Instruction::SpInith         => write!(f, "inith"),
            Instruction::SpUmem          => write!(f, "umem"),
            Instruction::SpUser          => write!(f, "user"),
            Instruction::SpDebug         => write!(f, "debug"),
        }
    }
}