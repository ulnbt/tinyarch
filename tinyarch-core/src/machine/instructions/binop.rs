use super::conv::*;

#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum BinaryOperator {
    // Signed integer arithmetic
    Add = 0, Sub = 1, SubR = 2,
    Mul = 3, Div = 4, Mod = 5, 
    Min = 6, Max = 7,
    // Floating point arithmetic
    AddF = 8, SubF = 9, SubFR = 10,
    MulF = 11, DivF = 12, ModF = 13,
    MinF = 14, MaxF = 15,
    // Bitwise arithmetic
    And = 16, Or = 17, Xor = 18, Xnor = 19,
    // Unsigned integers
    MinU = 20, MaxU = 21, DivU = 22, ModU = 23,
    // Rots/Shifts
    Rol = 24, Ror = 25,
    Lsl = 26, Lsr = 27, Asr = 28,
    // Special
    Fpintexp = 29,
    Nop = 31,
}

impl BinaryOperator {
    pub fn compute(&self, lhs: u32, rhs: u32) -> u32 {
        match self {
            // Signed int arithmetic
            BinaryOperator::Add => f_i32(i_i32(lhs) + i_i32(rhs)),
            BinaryOperator::Sub => f_i32(i_i32(lhs) - i_i32(rhs)),
            BinaryOperator::SubR => f_i32(i_i32(rhs) - i_i32(lhs)),
            BinaryOperator::Mul => f_i32(i_i32(lhs) * i_i32(rhs)),
            BinaryOperator::Div => f_i32(i_i32(lhs) / i_i32(rhs)),
            BinaryOperator::Mod => f_i32(i_i32(lhs) % i_i32(rhs)),
            BinaryOperator::Min => f_i32(i_i32(lhs).min(i_i32(rhs))),
            BinaryOperator::Max => f_i32(i_i32(lhs).max(i_i32(rhs))),

            // Floating point math
            BinaryOperator::AddF => f_f32(i_f32(lhs) + i_f32(rhs)),
            BinaryOperator::SubF => f_f32(i_f32(lhs) - i_f32(rhs)),
            BinaryOperator::SubFR => f_f32(i_f32(rhs) - i_f32(lhs)),
            BinaryOperator::MulF => f_f32(i_f32(lhs) * i_f32(rhs)),
            BinaryOperator::DivF => f_f32(i_f32(lhs) / i_f32(rhs)),
            BinaryOperator::ModF => f_f32(i_f32(lhs) % i_f32(rhs)),
            BinaryOperator::MinF => f_f32(i_f32(lhs).min(i_f32(rhs))),
            BinaryOperator::MaxF => f_f32(i_f32(lhs).max(i_f32(rhs))),

            // Bitwise math
            BinaryOperator::And => lhs & rhs,
            BinaryOperator::Or => lhs | rhs,
            BinaryOperator::Xor => lhs ^ rhs,
            BinaryOperator::Xnor => !(lhs ^ rhs),
            BinaryOperator::MinU => lhs.min(rhs),
            BinaryOperator::MaxU => lhs.max(rhs),
            BinaryOperator::DivU => lhs / rhs,
            BinaryOperator::ModU => lhs % rhs,

            // Shifts/other
            BinaryOperator::Rol => lhs.rotate_left(rhs),
            BinaryOperator::Ror => lhs.rotate_right(rhs),
            BinaryOperator::Lsl => lhs >> rhs,
            BinaryOperator::Lsr => lhs << rhs,
            BinaryOperator::Asr => f_i32(i_i32(lhs) >> i_i32(rhs)),
            BinaryOperator::Fpintexp => f_f32(i_f32(lhs) * (2f32).powi(i_i32(rhs))),

            BinaryOperator::Nop => 0,
        }
    }

    pub fn decode(op: u8) -> BinaryOperator {
        if op <= 29 {
            unsafe { std::mem::transmute(op) }
        } else if op <= 31 {
            BinaryOperator::Nop
        } else {
            panic!("Invalid operator!");
        }
    }
}