
pub static DEFAULT_ASSEMBLER_MACROS: [(&'static str, fn(&str, &mut Vec<String>) -> Result<(), String>); 1] = [
    ("STRING", asm_string_macro)
];

fn asm_string_macro(line: &str, instr: &mut Vec<String>) -> Result<(), String> {
    if let (Some(start), Some(end)) = (line.find('"'), line.rfind('"'))
    {
        if start < end {
            let content = &line[start + 1..end];
            for c in content.chars() {
                instr.push(format!("DAT 0x{:02X}", c as u32));
            }
            return Ok(());
        }
    }
    return Err(format!("Malformed STRING macro: {}", line));
}