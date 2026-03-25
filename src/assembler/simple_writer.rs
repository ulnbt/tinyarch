use super::ExecutableWriter;
use crate::machine::Machine;
use std::collections::HashMap;

pub struct SimpleExecutableWriter<'a> {
    machine: &'a mut Machine,
    labels: HashMap<String, u32>,
    offset: usize,
}

impl<'a> SimpleExecutableWriter<'a> {
    pub fn new(machine: &'_ mut Machine, offset: usize) -> SimpleExecutableWriter<'_> {
        SimpleExecutableWriter { machine, labels: HashMap::new(), offset }
    }
}

impl<'a> ExecutableWriter for SimpleExecutableWriter<'a> {
    fn write_instruction(&mut self, instr: u32, pos: usize) -> Result<(), String> {
        self.machine.memory[pos + self.offset] = instr;
        Ok(())
    }
    
    fn add_label(&mut self, label: &str, pos: u32) -> Result<(), String> {
        if self.labels.insert(label.to_string(), pos + self.offset as u32).is_some() {
            Err(format!("Label {} defined twice?", label))
        } else { Ok(()) }
    }
    
    fn get_labels(&self) -> Result<Vec<&str>, String> {
        Ok(self.labels.keys().map(|s| &s[..]).collect())
    }
    
    fn get_label_pos(&self, label: &str) -> Result<u32, String> {
        // the labels are taken directly from get_labels so it should be fine.
        Ok(*self.labels.get(label).expect("Executable writer: label not found!")) 
    }


}