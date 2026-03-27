use tinyarch::{assembler, machine, devices};

#[test]
fn hw_ta_produces_hello_world() {
    let source = include_str!("../examples/hw.ta");
    let mut m = machine::Machine::new(64 * 1024);
    let mut writer = assembler::SimpleExecutableWriter::new(&mut m, 0);
    assembler::assemble(source, &mut writer).expect("assembly failed");

    let dev = devices::buffer_output::BufferOutputDevice::connect(&mut m);

    let mut output = String::new();
    while m.step() {
        output.push_str(&dev.drain());
    }
    output.push_str(&dev.drain()); // final drain after halt

    assert_eq!(output, "Hello World!\n");
}
