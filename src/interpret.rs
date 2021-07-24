use crate::{Command, InputOutput};
use thiserror::Error;

type ExecuteResult = Result<SideEffect, Error>;

pub struct Interpreter<I: InputOutput> {
    cells: Vec<u8>,
    pointer: usize,
    io: I,
}

impl<I: InputOutput> Interpreter<I> {
    pub fn new(io: I, cells: usize) -> Self {
        let cells = vec![0; cells];
        let pointer = 0;
        Self { cells, pointer, io }
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
        self.io.write(self.cells[self.pointer])?;
        Ok(SideEffect::default())
    }

    fn input(&mut self) -> ExecuteResult {
        self.cells[self.pointer] = self.io.read()?;
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

    pub fn current_data(&self) -> u8 {
        self.cells[self.pointer]
    }

    pub fn current_pointer(&self) -> usize {
        self.pointer
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
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate};

    mock! {
        InputOutput {

        }

        impl InputOutput for InputOutput {
            fn read(&mut self) -> std::io::Result<u8>;
            fn write(&mut self, data: u8) -> std::io::Result<()>;
        }
    }

    #[test]
    fn move_pointer() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 10);
        interpreter
            .execute(&[
                Command::MovePointer { offset: 5 },
                Command::MovePointer { offset: -3 },
            ])
            .expect("Failed to execute");
        assert_eq!(2, interpreter.current_pointer());
        assert_eq!(0, interpreter.current_data());
    }

    #[test]
    fn move_past_tape_end() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 1);
        let result = interpreter.execute(&[Command::MovePointer { offset: 1 }]);
        assert!(matches!(result, Err(Error::OutOfBounds)));
    }

    #[test]
    fn move_before_tape_beginning() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 1);
        let result = interpreter.execute(&[Command::MovePointer { offset: -1 }]);
        assert!(matches!(result, Err(Error::OutOfBounds)));
    }

    #[test]
    fn modify_data() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 10);
        interpreter
            .execute(&[
                Command::ModifyData { modifier: 5 },
                Command::ModifyData { modifier: -3 },
            ])
            .expect("Failed to execute");
        assert_eq!(0, interpreter.current_pointer());
        assert_eq!(2, interpreter.current_data());
    }

    #[test]
    fn modify_data_loop_around() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 10);
        interpreter
            .execute(&[Command::ModifyData { modifier: 567 }])
            .expect("Failed to execute");
        assert_eq!(55, interpreter.current_data());
    }

    #[test]
    fn modify_data_loop_around_negative() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 10);
        interpreter
            .execute(&[Command::ModifyData { modifier: -567 }])
            .expect("Failed to execute");
        assert_eq!(201, interpreter.current_data());
    }

    #[test]
    fn input() {
        let mut io = MockInputOutput::new();
        io.expect_read().once().returning(|| Ok(42));
        let mut interpreter = Interpreter::new(io, 10);
        interpreter
            .execute(&[Command::Input])
            .expect("Failed to execute");
        assert_eq!(0, interpreter.current_pointer());
        assert_eq!(42, interpreter.current_data());
    }

    #[test]
    fn output() {
        let mut io = MockInputOutput::new();
        io.expect_write()
            .once()
            .with(predicate::eq(42))
            .returning(|_| Ok(()));
        let mut interpreter = Interpreter::new(io, 10);
        interpreter
            .execute(&[Command::ModifyData { modifier: 42 }, Command::Output])
            .expect("Failed to execute");
        assert_eq!(0, interpreter.current_pointer());
        assert_eq!(42, interpreter.current_data());
    }

    #[test]
    fn loop_skips_if_zero() {
        let mut interpreter = Interpreter::new(MockInputOutput::new(), 10);
        interpreter
            .execute(&[
                Command::LoopStart { end_index: 2 },
                Command::ModifyData { modifier: 15 },
                Command::LoopEnd { start_index: 0 },
            ])
            .expect("Failed to execute");
        assert_eq!(0, interpreter.current_data());
        assert_eq!(0, interpreter.current_pointer());
    }

    #[test]
    fn loop_executes_if_non_zero() {
        let mut io = MockInputOutput::new();
        io.expect_write()
            .once()
            .with(predicate::eq(2))
            .returning(|_| Ok(()));
        io.expect_write()
            .once()
            .with(predicate::eq(1))
            .returning(|_| Ok(()));
        let mut interpreter = Interpreter::new(io, 10);
        interpreter
            .execute(&[
                Command::ModifyData { modifier: 2 },
                Command::LoopStart { end_index: 3 },
                Command::Output,
                Command::ModifyData { modifier: -1 },
                Command::LoopEnd { start_index: 1 },
            ])
            .expect("Failed to execute");
        assert_eq!(0, interpreter.current_data());
        assert_eq!(0, interpreter.current_pointer());
    }
}
