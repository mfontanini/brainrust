use brainrust::{interpret::Error, *};
use std::{
    env::args,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    process::exit,
};

fn read_program<R>(
    mut processor: Preprocessor,
    input: R,
) -> Result<Vec<Command>, Box<dyn std::error::Error>>
where
    R: Read,
{
    let input = BufReader::new(input);
    for line in input.lines() {
        let line = line?;
        for character in line.chars() {
            let command = character.into();
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
    println!("> Executing program containing {} commands", program.len());
    let mut interpreter = Interpreter::new(DefaultInputOutput::default(), 30000);
    match interpreter.execute(&program) {
        Ok(_) => (),
        Err(Error::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => (),
        Err(e) => println!("Error executing program: {:?}", e),
    }
}
