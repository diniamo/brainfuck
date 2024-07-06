use std::{env, fs};
use std::io::{self, Read, StdinLock, StdoutLock, Write};
use std::path::Path;

#[derive(PartialEq, Eq, Debug)]
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

struct Runtime<'a> {
    pub data: [u8; 30_000],
    pub data_pointer: usize,

    pub stdout: StdoutLock<'a>,
    pub stdin: StdinLock<'a>,
}

fn skip_loop(instructions: &[Instruction]) -> usize {
    let mut level = 0u32;

    for (i, instruction) in instructions.iter().enumerate() {
        match instruction {
            Instruction::LoopOpen => level += 1,
            Instruction::LoopClose => level -= 1,
            _ => (),
        }

        if level == 0 {
            return i + 1;
        }
    }

    unreachable!();
}

fn execute(instructions: &[Instruction], runtime: &mut Runtime) -> usize {
    let mut instruction_pointer = 0usize;

    loop {
        if instruction_pointer >= instructions.len() {
            break;
        }

        let instruction = &instructions[instruction_pointer];

        match instruction {
            Instruction::Right => {
                if runtime.data_pointer + 1 >= runtime.data.len() {
                    runtime.data_pointer = 0
                } else {
                    runtime.data_pointer += 1
                }

                instruction_pointer += 1;
            }
            Instruction::Left => {
                if runtime.data_pointer == 0 {
                    runtime.data_pointer = runtime.data.len() - 1;
                } else {
                    runtime.data_pointer -= 1;
                }

                instruction_pointer += 1;
            }
            Instruction::Increment => {
                runtime.data[runtime.data_pointer] = runtime.data[runtime.data_pointer]
                    .checked_add(1)
                    .unwrap_or(u8::MIN);

                instruction_pointer += 1;
            }
            Instruction::Decrement => {
                runtime.data[runtime.data_pointer] = runtime.data[runtime.data_pointer]
                    .checked_sub(1)
                    .unwrap_or(u8::MAX);

                instruction_pointer += 1;
            }
            Instruction::Output => {
                runtime
                    .stdout
                    .write_all(&[runtime.data[runtime.data_pointer]])
                    .expect("Failed to write to stdout");

                instruction_pointer += 1;
            }
            Instruction::Input => {
                let mut buf = [0u8; 1];
                runtime.data[runtime.data_pointer] = if runtime.stdin.read_exact(&mut buf).is_ok() {
                    buf[0]
                } else {
                    0
                };

                instruction_pointer += 1;
            }
            Instruction::LoopOpen => {
                if runtime.data[runtime.data_pointer] == 0 {
                    instruction_pointer += skip_loop(&instructions[instruction_pointer..]);
                } else {
                    instruction_pointer +=
                        execute(&instructions[(instruction_pointer + 1)..], runtime) + 1;
                }
            }
            Instruction::LoopClose => {
                if runtime.data[runtime.data_pointer] != 0 {
                    instruction_pointer = 0;
                } else {
                    return instruction_pointer + 1;
                }
            }
            Instruction::Invalid => unreachable!(),
        }
    }

    instruction_pointer
}

fn main() {
    let argument = env::args()
        .nth(1)
        .expect("Please provide a file or a program as an argument");

    let instructions = if let Ok(contents) = fs::read_to_string(Path::new(&argument)) {
        contents
    } else {
        argument
    }
    .chars()
    .filter_map(|c| match c.into() {
        Instruction::Invalid => None,
        instruction => Some(instruction),
    })
    .collect::<Box<[Instruction]>>();

    let mut runtime = Runtime {
        data: [0u8; 30_000],
        data_pointer: 0,

        stdout: io::stdout().lock(),
        stdin: io::stdin().lock(),
    };

    execute(&instructions, &mut runtime);
}
