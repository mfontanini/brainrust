use crate::Command;
use std::io::{self, Read};
use thiserror::Error;

type ExecuteResult = Result<SideEffect, Error>;

pub struct Interpreter {
    cells: Vec<u8>,
    pointer: usize,
}

impl Interpreter {
    pub fn with_cells(cells: usize) -> Self {
        let cells = vec![0; cells];
        let pointer = 0;
        Self { cells, pointer }
    }

    pub fn execute(&mut self, program: &[Command]) -> Result<(), Error> {
        let mut command_index = 0;
        while command_index < program.len() {
            let side_effect = self.execute_command(&program[command_index])?;
            match side_effect {
                SideEffect::Advance => command_index += 1,
                SideEffect::JumpTo(index) => command_index = index,
            }
        }
        Ok(())
    }

    fn execute_command(&mut self, command: &Command) -> ExecuteResult {
        match command {
            Command::MovePointer { offset } => self.move_pointer(*offset),
            Command::ModifyData { modifier } => self.modify_data(*modifier),
            Command::Output => self.output(),
            Command::Input => self.input(),
            Command::LoopStart { end_index } => self.loop_start(*end_index),
            Command::LoopEnd { start_index } => self.loop_end(*start_index),
        }
    }

    fn move_pointer(&mut self, offset: i32) -> ExecuteResult {
        let new_pointer = self.pointer as i32 + offset;
        if new_pointer < 0 || new_pointer as usize >= self.cells.len() {
            Err(Error::OutOfBounds)
        } else {
            self.pointer = new_pointer as usize;
            Ok(SideEffect::default())
        }
    }

    fn modify_data(&mut self, modifier: i32) -> ExecuteResult {
        let mut current = self.current_data() as i32;
        current += modifier;
        self.cells[self.pointer] = (current % 256) as u8;
        Ok(SideEffect::default())
    }

    fn output(&mut self) -> ExecuteResult {
        let output_char = char::from_u32(self.cells[self.pointer] as u32).unwrap_or('?');
        print!("{}", output_char);
        Ok(SideEffect::default())
    }

    fn input(&mut self) -> ExecuteResult {
        let mut buffer = [0; 1];
        io::stdin().read_exact(&mut buffer)?;
        self.cells[self.pointer] = buffer[0];
        Ok(SideEffect::default())
    }

    fn loop_start(&mut self, end_index: usize) -> ExecuteResult {
        if self.current_data() == 0 {
            Ok(SideEffect::JumpTo(end_index))
        } else {
            Ok(SideEffect::default())
        }
    }

    fn loop_end(&mut self, start_index: usize) -> ExecuteResult {
        if self.current_data() != 0 {
            Ok(SideEffect::JumpTo(start_index))
        } else {
            Ok(SideEffect::default())
        }
    }

    fn current_data(&self) -> u8 {
        self.cells[self.pointer]
    }
}

enum SideEffect {
    Advance,
    JumpTo(usize),
}

impl Default for SideEffect {
    fn default() -> Self {
        Self::Advance
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Out of bounds")]
    OutOfBounds,

    #[error("Loop ends before starting")]
    MalformedLoopEnd,

    #[error("No loop end found")]
    NoLoopEnd,

    #[error("io: {0}")]
    Io(#[from] io::Error),
}
