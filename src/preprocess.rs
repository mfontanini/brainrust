use crate::command::{Command, RawCommand};
use thiserror::Error;

#[derive(Default)]
pub struct Preprocessor {
    commands: Vec<Command>,
    open_loop_indexes: Vec<usize>,
    current_command: Option<Command>,
}

impl Preprocessor {
    pub fn process(&mut self, command: &RawCommand) -> Result<(), Error> {
        match command {
            RawCommand::IncrementPointer => self.increment_pointer(),
            RawCommand::DecrementPointer => self.decrement_pointer(),
            RawCommand::IncrementData => self.increment_data(),
            RawCommand::DecrementData => self.decrement_data(),
            RawCommand::Output => self.output(),
            RawCommand::Input => self.input(),
            RawCommand::LoopStart => self.loop_start(),
            RawCommand::LoopEnd => self.loop_end()?,
            RawCommand::Noop => (),
        };
        Ok(())
    }

    pub fn program(mut self) -> Result<Vec<Command>, Error> {
        if !self.open_loop_indexes.is_empty() {
            return Err(Error::OpenLoops(self.open_loop_indexes.len()));
        }
        self.store_current();
        Ok(self.commands)
    }

    fn increment_pointer(&mut self) {
        self.alter_pointer(1);
    }

    fn decrement_pointer(&mut self) {
        self.alter_pointer(-1);
    }

    fn alter_pointer(&mut self, step: i32) {
        let offset = match self.current_command.take() {
            Some(Command::MovePointer { offset }) => offset + step,
            None => step,
            Some(command) => {
                self.commands.push(command);
                step
            }
        };
        self.current_command = Some(Command::MovePointer { offset });
    }

    fn increment_data(&mut self) {
        self.alter_data(1);
    }

    fn decrement_data(&mut self) {
        self.alter_data(-1);
    }

    fn alter_data(&mut self, step: i32) {
        let modifier = match self.current_command.take() {
            Some(Command::ModifyData { modifier }) => modifier + step,
            None => step,
            Some(command) => {
                self.commands.push(command);
                step
            }
        };
        self.current_command = Some(Command::ModifyData { modifier });
    }

    fn output(&mut self) {
        self.handle_generic(Command::Output);
    }

    fn input(&mut self) {
        self.handle_generic(Command::Input);
    }

    fn loop_start(&mut self) {
        self.store_current();
        self.open_loop_indexes.push(self.commands.len());
        // We'll come back and fill this up later
        self.commands.push(Command::LoopStart { end_index: 0 });
        self.current_command = None;
    }

    fn loop_end(&mut self) -> Result<(), Error> {
        self.store_current();
        let start_index = match self.open_loop_indexes.pop() {
            Some(index) => index,
            None => return Err(Error::NoLoopStart),
        };
        let end_index = self.commands.len();
        self.commands.push(Command::LoopEnd { start_index });
        self.commands[start_index] = Command::LoopStart { end_index };
        self.current_command = None;
        Ok(())
    }

    fn handle_generic(&mut self, expected: Command) {
        self.store_current();
        self.current_command = Some(expected);
    }

    fn store_current(&mut self) {
        if let Some(command) = self.current_command.take() {
            self.commands.push(command);
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0} open loops on program end")]
    OpenLoops(usize),

    #[error("Unmatched loop ending")]
    NoLoopStart,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process_program(
        mut processor: Preprocessor,
        program: &[RawCommand],
    ) -> Result<Vec<Command>, Error> {
        for command in program {
            processor.process(command)?;
        }
        processor.program()
    }

    fn run_test(program: &[RawCommand], expected: &[Command]) {
        let processor = Preprocessor::default();
        let result = process_program(processor, program).expect("Optimization failed");
        assert_eq!(expected, result);
    }

    fn expect_failure(program: &[RawCommand], error: Error) {
        let processor = Preprocessor::default();
        let result = process_program(processor, program);
        assert_eq!(Err(error), result);
    }

    #[test]
    fn pointer_moves() {
        let program = &[
            RawCommand::IncrementPointer,
            RawCommand::IncrementPointer,
            RawCommand::DecrementPointer,
            RawCommand::IncrementPointer,
        ];
        let expected = &[Command::MovePointer { offset: 2 }];
        run_test(program, expected);
    }

    #[test]
    fn pointer_moves_backwards_first() {
        let program = &[
            RawCommand::DecrementPointer,
            RawCommand::DecrementPointer,
            RawCommand::IncrementPointer,
        ];
        let expected = &[Command::MovePointer { offset: -1 }];
        run_test(program, expected);
    }

    #[test]
    fn pointer_moves_negative_offset() {
        let program = &[
            RawCommand::IncrementPointer,
            RawCommand::DecrementPointer,
            RawCommand::DecrementPointer,
        ];
        let expected = &[Command::MovePointer { offset: -1 }];
        run_test(program, expected);
    }

    #[test]
    fn modify_data() {
        let program = &[
            RawCommand::IncrementData,
            RawCommand::IncrementData,
            RawCommand::DecrementData,
            RawCommand::IncrementData,
        ];
        let expected = &[Command::ModifyData { modifier: 2 }];
        run_test(program, expected);
    }

    #[test]
    fn modify_data_negative() {
        let program = &[
            RawCommand::DecrementData,
            RawCommand::IncrementData,
            RawCommand::DecrementData,
        ];
        let expected = &[Command::ModifyData { modifier: -1 }];
        run_test(program, expected);
    }

    #[test]
    fn io() {
        let program = &[RawCommand::Input, RawCommand::Output, RawCommand::Input];
        let expected = &[Command::Input, Command::Output, Command::Input];
        run_test(program, expected);
    }

    #[test]
    fn broken_loops() {
        expect_failure(&[RawCommand::LoopStart], Error::OpenLoops(1));
        expect_failure(
            &[RawCommand::LoopStart, RawCommand::LoopStart],
            Error::OpenLoops(2),
        );
        expect_failure(&[RawCommand::LoopEnd], Error::NoLoopStart);
    }

    #[test]
    fn empty_loop() {
        let program = &[RawCommand::LoopStart, RawCommand::LoopEnd];
        let expected = &[
            Command::LoopStart { end_index: 1 },
            Command::LoopEnd { start_index: 0 },
        ];
        run_test(program, expected);
    }

    #[test]
    fn non_empty_loop() {
        let program = &[
            RawCommand::LoopStart,
            RawCommand::Input,
            RawCommand::LoopEnd,
        ];
        let expected = &[
            Command::LoopStart { end_index: 2 },
            Command::Input,
            Command::LoopEnd { start_index: 0 },
        ];
        run_test(program, expected);
    }

    #[test]
    fn empty_nested_loops() {
        let program = &[
            RawCommand::LoopStart,
            RawCommand::LoopStart,
            RawCommand::LoopEnd,
            RawCommand::LoopEnd,
        ];
        let expected = &[
            Command::LoopStart { end_index: 3 },
            Command::LoopStart { end_index: 2 },
            Command::LoopEnd { start_index: 1 },
            Command::LoopEnd { start_index: 0 },
        ];
        run_test(program, expected);
    }

    #[test]
    fn non_empty_nested_loops() {
        let program = &[
            RawCommand::LoopStart,
            RawCommand::Input,
            RawCommand::LoopStart,
            RawCommand::Output,
            RawCommand::LoopEnd,
            RawCommand::LoopEnd,
        ];
        let expected = &[
            Command::LoopStart { end_index: 5 },
            Command::Input,
            Command::LoopStart { end_index: 4 },
            Command::Output,
            Command::LoopEnd { start_index: 2 },
            Command::LoopEnd { start_index: 0 },
        ];
        run_test(program, expected);
    }

    #[test]
    fn noops_are_ignored() {
        let program = &[
            RawCommand::Input,
            RawCommand::Noop,
            RawCommand::Noop,
            RawCommand::Output,
        ];
        let expected = &[Command::Input, Command::Output];
        run_test(program, expected);
    }
}
