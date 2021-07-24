use brainrust::{command::parse, *};
use std::{
    env::args,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
    process::exit,
};

fn read_program<R>(mut processor: Preprocessor, input: R) -> Result<Vec<Command>, Box<dyn Error>>
where
    R: Read,
{
    let input = BufReader::new(input);
    for line in input.lines() {
        let line = line?;
        for character in line.chars() {
            let command = parse(&character);
            processor.process(&command)?;
        }
    }
    Ok(processor.program()?)
}

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <program-path>", args[0]);
        exit(1);
    }
    let file = match File::open(&args[1]) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            exit(1);
        }
    };
    let program = match read_program(Preprocessor::default(), file) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error reading program: {}", e);
            exit(1);
        }
    };
    let mut interpreter = Interpreter::with_cells(30000);
    if let Err(e) = interpreter.execute(&program) {
        println!("Error executing program: {:?}", e);
    }
}
