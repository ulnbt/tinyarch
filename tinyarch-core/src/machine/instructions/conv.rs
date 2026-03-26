// Raw bit conversions of types. Used in binop.rs, and conditions.rs

pub fn i_i32(v: u32) -> i32 {
    i32::from_ne_bytes(v.to_ne_bytes())
}
pub fn f_i32(v: i32) -> u32 {
    u32::from_ne_bytes(v.to_ne_bytes())
}
pub fn i_f32(v: u32) -> f32 {
    f32::from_ne_bytes(v.to_ne_bytes())
}
pub fn f_f32(v: f32) -> u32 {
    u32::from_ne_bytes(v.to_ne_bytes())
}