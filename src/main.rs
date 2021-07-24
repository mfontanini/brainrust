use brainrust::{command::parse, *};

fn main() {
    let raw_program = r"
        ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
    ";
    let commands = raw_program.chars().map(|c| parse(&c));
    let mut processor = Preprocessor::default();
    for command in commands {
        processor
            .process(&command)
            .expect("Error preprocessing command")
    }
    let program = processor.program().expect("Preprocessing failed");
    let mut interpreter = Interpreter::with_cells(30000);
    if let Err(e) = interpreter.execute(&program) {
        println!("Error: {:?}", e);
    }
}
