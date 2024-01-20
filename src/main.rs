use bimap::BiHashMap;
use std::io::{Read, Write};
use std::path::Path;

#[derive(PartialEq, Eq)]
enum Instruction {
    Right,
    Left,
    Increment,
    Decrement,
    Output,
    Input,
    LoopOpen,
    LoopClose,
    Invalid,
}

impl From<char> for Instruction {
    fn from(value: char) -> Self {
        match value {
            '>' => Self::Right,
            '<' => Self::Left,
            '+' => Self::Increment,
            '-' => Self::Decrement,
            '.' => Self::Output,
            ',' => Self::Input,
            '[' => Self::LoopOpen,
            ']' => Self::LoopClose,
            _ => Self::Invalid,
        }
    }
}

fn find_closing_bracket(instructions: &[Instruction], index: usize) -> Option<usize> {
    let mut inner_count = 0u32;
    #[allow(clippy::needless_range_loop)]
    for i in (index + 1)..instructions.len() {
        let instruction = &instructions[i];
        match instruction {
            Instruction::LoopOpen => {
                inner_count += 1;
            }
            Instruction::LoopClose => {
                if inner_count == 0 {
                    return Some(i);
                } else {
                    inner_count -= 1;
                }
            }
            _ => {}
        }
    }

    None
}

fn main() {
    let argument = std::env::args()
        .nth(1)
        .expect("Please provide a file or a program as an argument");

    let instructions: Vec<Instruction> =
        if let Ok(contents) = std::fs::read_to_string(Path::new(&argument)) {
            contents
        } else {
            argument
        }
        .chars()
        .filter_map(|c| match c.into() {
            Instruction::Invalid => None,
            instruction => Some(instruction),
        })
        .collect();

    let mut loop_map: BiHashMap<usize, usize> = BiHashMap::new();
    for (i, instruction) in instructions.iter().enumerate() {
        if matches!(instruction, Instruction::LoopOpen) {
            loop_map.insert(
                i,
                find_closing_bracket(&instructions, i).unwrap_or_else(|| {
                    println!("Loop started at index {i} wasn't closed, terminating");
                    std::process::exit(1);
                }),
            );
        }
    }

    let mut data = [0u8; 30_000];
    let mut data_pointer = 0usize;
    let mut instruction_pointer = 0usize;

    let mut stdout = std::io::stdout().lock();
    let mut stdin = std::io::stdin().lock();

    loop {
        if instruction_pointer >= instructions.len() {
            break;
        }

        let instruction = &instructions[instruction_pointer];
        match instruction {
            Instruction::Right => {
                if data_pointer + 1 >= data.len() {
                    data_pointer = 0
                } else {
                    data_pointer += 1
                }
            }
            Instruction::Left => {
                if data_pointer == 0 {
                    data_pointer = data.len() - 1;
                } else {
                    data_pointer -= 1;
                }
            }
            Instruction::Increment => {
                data[data_pointer] = data[data_pointer].checked_add(1).unwrap_or(u8::MIN)
            }
            Instruction::Decrement => {
                data[data_pointer] = data[data_pointer].checked_sub(1).unwrap_or(u8::MAX)
            }
            Instruction::Output => {
                stdout
                    .write(&[data[data_pointer]])
                    .expect("Failed to write to stdout");
            }
            Instruction::Input => {
                let mut buf = [0u8; 1];
                if let Err(_) = stdin.read_exact(&mut buf) {
                    buf[0] = 0;
                }
                data[data_pointer] = buf[0];
            }
            Instruction::LoopOpen => {
                if data[data_pointer] == 0 {
                    instruction_pointer = *loop_map.get_by_left(&instruction_pointer).unwrap();
                } else {
                    instruction_pointer += 1;
                }
            }
            Instruction::LoopClose => {
                if data[data_pointer] != 0 {
                    instruction_pointer = *loop_map.get_by_right(&instruction_pointer).unwrap();
                } else {
                    instruction_pointer += 1;
                }
            }
            Instruction::Invalid => panic!("Invalid instruction in sanitized set"),
        }

        if !matches!(instruction, Instruction::LoopOpen | Instruction::LoopClose) {
            instruction_pointer += 1;
        }
    }
}
