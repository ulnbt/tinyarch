
// These are somd old debug console things, they might be added to the console IO later.

fn input_r1(m: &mut machine::Machine) {
    println!("Set input register:");
    let mut s = String::new();
    m.state().reg[1] = loop {
        s.clear();
        print!("r1="); 
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::io::stdin().read_line(&mut s).unwrap();
        if let Ok(x) = s.trim().parse() { break x; } 
        println!("Invalid u32!");
    };
}



fn set_memory_loop(m: &mut machine::Machine) {
    let mut s=String::new();
    let mut addr = 0;
    loop {
        print!("::");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        s.clear();
        std::io::stdin().read_line(&mut s).unwrap();
        if s.trim() == "" { break; }
        match assembler::parse_instruction(s.trim()) {
            Ok(n)=>{ 
                m.memory[addr] = n.encode();
                println!("[{:x}]={}",addr, n); 
                addr += 1;
            },
            Err(e)=>println!("Error: {e}")
        }
    }
}