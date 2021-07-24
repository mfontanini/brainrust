use std::io::{self, Read};

pub trait InputOutput {
    fn read(&mut self) -> io::Result<u8>;
    fn write(&mut self, data: u8) -> io::Result<()>;
}

#[derive(Default)]
pub struct DefaultInputOutput {}

impl InputOutput for DefaultInputOutput {
    fn read(&mut self) -> io::Result<u8> {
        let mut buffer = [0; 1];
        io::stdin().read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    fn write(&mut self, data: u8) -> io::Result<()> {
        let output_char = char::from_u32(data as u32).unwrap_or('?');
        print!("{}", output_char);
        Ok(())
    }
}
