#[derive(Clone, Debug, PartialEq)]
pub enum RawCommand {
    IncrementPointer,
    DecrementPointer,
    IncrementData,
    DecrementData,
    Output,
    Input,
    LoopStart,
    LoopEnd,
    Noop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    MovePointer { offset: i32 },
    ModifyData { modifier: i32 },
    Output,
    Input,
    LoopStart { end_index: usize },
    LoopEnd { start_index: usize },
}

impl From<char> for RawCommand {
    fn from(input: char) -> Self {
        match input {
            '>' => Self::IncrementPointer,
            '<' => Self::DecrementPointer,
            '+' => Self::IncrementData,
            '-' => Self::DecrementData,
            '.' => Self::Output,
            ',' => Self::Input,
            '[' => Self::LoopStart,
            ']' => Self::LoopEnd,
            _ => Self::Noop,
        }
    }
}
