
pub trait ExecutableWriter {
    fn write_instruction(&mut self, instr: u32, pos: usize) -> Result<(), String>;
    fn add_label(&mut self, label: &str, pos: u32) -> Result<(), String>;
    fn get_labels(&self) -> Result<Vec<&str>, String>;
    fn get_label_pos(&self, label: &str) -> Result<u32, String>;
    fn macros<'a>() -> &'a [(&'static str, fn(&mut Self, &str, &mut Vec<String>) -> Result<(), String>)] { &[] }
}