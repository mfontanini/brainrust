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

pub fn parse(input: &char) -> RawCommand {
    match input {
        '>' => RawCommand::IncrementPointer,
        '<' => RawCommand::DecrementPointer,
        '+' => RawCommand::IncrementData,
        '-' => RawCommand::DecrementData,
        '.' => RawCommand::Output,
        ',' => RawCommand::Input,
        '[' => RawCommand::LoopStart,
        ']' => RawCommand::LoopEnd,
        _ => RawCommand::Noop,
    }
}
