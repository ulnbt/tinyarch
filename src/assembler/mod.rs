
//! ### TinyArch v2: Assembler
//! 
//! 


mod parse;

/// Parses a line of assembly into a `machine::Instruction`
pub use parse::parse_instruction;

/// Possible errors when parsing instruction with `parse_instruction`
pub use parse::ParseError;

mod default_macros;
use default_macros::DEFAULT_ASSEMBLER_MACROS;

mod executable_writer;
/// Trait used to implement custom assembly macros, file writing and linking.
pub use executable_writer::ExecutableWriter;

mod simple_writer;
/// A simple implementation of `ExecutableWriter` that writes directly to a `Machine`'s memory.
pub use simple_writer::SimpleExecutableWriter;


/// Parses a multiline program of assembly into an `ExecutableWriter`
pub fn assemble<T: ExecutableWriter>(src: &str, mut ew: &mut T) -> Result<(), String> {
    let mut instructions: Vec<String> = Vec::new();
    let ew_macros = T::macros();

    // pass 1: strip comments, find labels, trim whitespace, apply macros
    for line in src.lines() {
        // Strip comments
        let line = line.split(';').next().unwrap().trim();
        if line.is_empty() { continue; }

        // Find labels
        let instr: &str = if let Some(pos) = line.find(':') {
            let label = line[..pos].trim();
            if !label.is_empty() && label.chars().all(|c| c.is_alphanumeric() || c == '_') {
                ew.add_label(label, instructions.len() as u32)?;
                line[pos + 1..].trim()
            } else { line }
        } else { line };

        if instr.is_empty() { continue; }

        // Apply default macros
        for (n, f) in DEFAULT_ASSEMBLER_MACROS {
            if instr.starts_with(n) { 
                f(n, &mut instructions)?;
                continue;
            }
        }

        // Apply executable writer's macros
        for (n, f) in ew_macros {
            if instr.starts_with(n) { 
                f(&mut ew, n, &mut instructions)?;
                continue;
            }
        }

        // Otherwise push instruction as is
        instructions.push(instr.to_string());
    }

    // pass 2: replace labels with numerical addresses in instructions

    // get keys and sort by length (as to not lazily match label substrings)
    let mut labels = ew.get_labels()?;    
    labels.sort_by_key(|a| std::cmp::Reverse(a.len()));

    for label in labels {
        let value = format!("{}", ew.get_label_pos(label)?);
        let at_label = format!("@{}", label);
        let at_value = format!("@{}", value);

        for line in instructions.iter_mut() {
            // The find/replace needs to be much more robust I feel.
            // Am I sure that the word boundary is not needed for at_label's?
            *line = line.replace(&at_label, &at_value);
            if line.contains(label) {
                *line = replace_whole_word(line, label, &value); // attempts to replace it
            }
        }
    }

    // pass 3: encode into writter
    for (pos, instr) in instructions.iter().enumerate() {
        println!("{instr}");
        match parse::parse_instruction(instr) {
            Ok(n) => { 
                println!("==> {}", n);
                ew.write_instruction(n.encode(), pos)?; 
            }
            Err(e) => { return Err(format!( "Assembly error at instruction: {e}" )) }
        }
    }

    Ok(())
}

fn replace_whole_word(s: &str, from: &str, to: &str) -> String {
    if from.is_empty() {
        return s.to_string();
    }

    let mut result = String::with_capacity(s.len());
    let mut last_end = 0;
    for (start, matched) in s.match_indices(from) {
        let end = start + matched.len();

        let left_ok = s[..start].chars().next_back()
            .map_or(true, |c| !c.is_alphanumeric() && c != '_');
        let right_ok = s[end..].chars().next()
            .map_or(true, |c| !c.is_alphanumeric() && c != '_');

        result.push_str(&s[last_end..start]);

        if left_ok && right_ok { result.push_str(to); } 
        else { result.push_str(matched); }

        last_end = end;
    }

    result.push_str(&s[last_end..]);
    result
}