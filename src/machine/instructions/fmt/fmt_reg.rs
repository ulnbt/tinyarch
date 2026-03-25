
pub fn reg_name(r: u8) -> String {
    match r {
        0 => "0".to_string(),
        r => format!("r{:X}", r),
    }
}

pub fn r_rix_name(r_rix: u8, m: u8) -> String {
    let r   = r_rix >> 4;
    let rix = r_rix & 0xF;
    match (r, rix, m) {
        (0, 0, _) => "0".to_string(),
        (r, 0, _) => format!("r{:X}", r),
        (0, rix, 1) => format!("@r{:X}", rix),
        (0, rix, m) => format!("@r{:X}*{}", rix, m),
        (r, rix, 1) => format!("r{:X}[r{:X}]", r, rix),
        (r, rix, m) => format!("r{:X}[r{:X}*{}]", r, rix, m),
    }
}
