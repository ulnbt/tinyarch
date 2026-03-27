use super::conv::*;

#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum Condition {
    Eq = 0, NEq = 1,
    Gt = 2, NGt = 3,
    GtU = 4, NGtU = 5,
    Mask = 6, NMask = 7,
    EqF = 8, NEqF = 9,
    GtF = 10, NGtF = 11,
    Par = 12, NPar = 13,
    Even = 14, NEven = 15,
}

impl Condition {
    pub fn compare(&self, rhs: u32, lhs: u32) -> bool {
        let r = match self {
            Condition::Eq | Condition::NEq => rhs == lhs,
            Condition::Gt | Condition::NGt => i_i32(rhs) > i_i32(lhs),
            Condition::GtU | Condition::NGtU => rhs >= lhs,
            Condition::Mask | Condition::NMask => (i_i32(rhs) & i_i32(lhs)) == 0,
            Condition::EqF | Condition::NEqF => i_f32(rhs) == i_f32(lhs),
            Condition::GtF | Condition::NGtF => i_f32(rhs) > i_f32(lhs),
            Condition::Par | Condition::NPar => rhs.count_ones() == lhs.count_ones(),
            Condition::Even | Condition::NEven => rhs % 2 == lhs % 2,
        };
        if (*self as u8) & 1 == 0 { r } else { !r }
    }
    pub fn decode(op: u8) -> Condition {
        if op <= 15 {
            unsafe { std::mem::transmute(op) }
        } else {
            panic!("Invalid condition!");
        }
    }
}