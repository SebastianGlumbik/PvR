//! Run this file with `cargo test --test 05_brainfuck_interpreter`.

// (bonus): Create an interpreter for the [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) language.
// The Brainfuck program will be parsed out of a string and represented as a struct.
//
// Handle both parsing and execution errors using enums representing error conditions,
// see tests for details.
// A parsing error can be either an unknown instruction or an unpaired loop instruction.
// An execution error can be either that the program tries to read input, but there is no more
// input available, or when the program executes more than 10000 instructions (which probably
// signals an infinite loop).
//
// Hint: Put `#[derive(Debug, Eq, PartialEq)]` on top of `ParseError`, `ExecuteError` and `Program`
// (and any other custom types nested inside them) so that asserts in tests work.

#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
    UnknownInstruction { location: usize, instruction: char },
    UnmatchedLoop { location: usize },
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecuteError {
    NoInputLeft,
    InfiniteLoop,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Program {
    code: String,
    loops: Vec<(usize, usize)>,
}

impl Program {
    pub fn execute(&self, input: Vec<u8>, mut data: Vec<u8>) -> Result<String, ExecuteError> {
        let mut output = String::new();
        let mut index: usize = 0;
        let mut data_ptr: usize = 0;
        let mut input_ptr: usize = 0;
        let mut instruction_counter: usize = 0;
        while index < self.code.len() {
            if instruction_counter > 10000 {
                return Err(ExecuteError::InfiniteLoop);
            }
            let instruction = self.code.chars().nth(index).unwrap_or_default();
            instruction_counter += 1;
            match instruction {
                '>' if data_ptr + 1 < data.len() => {
                    data_ptr += 1;
                }
                '<' if data_ptr > 0 => {
                    data_ptr -= 1;
                }
                '+' => {
                    data[data_ptr] += 1;
                }
                '-' => {
                    data[data_ptr] -= 1;
                }
                '.' => output.push(char::from(data[data_ptr])),
                ',' => {
                    if let Some(byte) = input.get(input_ptr) {
                        data[data_ptr] += byte;
                        input_ptr += 1;
                    } else {
                        return Err(ExecuteError::NoInputLeft);
                    }
                }
                '[' if data[data_ptr] == 0 => {
                    index = self.loops[index].1 + 1;
                    continue;
                }
                ']' if data[data_ptr] != 0 => {
                    if let Some((position, _)) = self.loops.iter().find(|(_, end)| *end == index) {
                        index = position + 1;
                        continue;
                    }
                }
                _ => {}
            }
            index += 1;
        }

        Ok(output)
    }
}

pub fn parse_program(program: &str) -> Result<Program, ParseError> {
    let mut stack = Vec::<usize>::new();
    let mut loops = Vec::<(usize, usize)>::new();

    for (location, instruction) in program.chars().enumerate() {
        match instruction {
            '>' | '<' | '+' | '-' | '.' | ',' => continue,
            '[' => stack.push(location),
            ']' => {
                if let Some(start) = stack.pop() {
                    loops.push((start, location))
                } else {
                    return Err(ParseError::UnmatchedLoop { location });
                }
            }
            _ => {
                return Err(ParseError::UnknownInstruction {
                    location,
                    instruction,
                })
            }
        }
    }

    if stack.is_empty() {
        Ok(Program {
            code: program.to_string(),
            loops,
        })
    } else {
        Err(ParseError::UnmatchedLoop {
            location: stack.pop().unwrap_or_default(),
        })
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::{parse_program, ExecuteError, ParseError};

    #[test]
    fn parse_empty() {
        check_output("", "", "");
    }

    #[test]
    fn parse_unknown_instruction() {
        assert!(matches!(
            parse_program(">p"),
            Err(ParseError::UnknownInstruction {
                location: 1,
                instruction: 'p'
            })
        ));
    }

    #[test]
    fn parse_unmatched_loop_start() {
        assert_eq!(
            parse_program(">++[+>][++>"),
            Err(ParseError::UnmatchedLoop { location: 7 })
        );
    }

    #[test]
    fn parse_unmatched_loop_end() {
        assert_eq!(
            parse_program(">++[+>][++>]+]"),
            Err(ParseError::UnmatchedLoop { location: 13 })
        );
    }

    #[test]
    fn missing_input() {
        let program = parse_program(",").unwrap();
        let result = program.execute(vec![], vec![0; 30000]);
        assert_eq!(result, Err(ExecuteError::NoInputLeft));
    }

    #[test]
    fn infinite_loop() {
        let program = parse_program("+[]").unwrap();
        let result = program.execute(vec![], vec![0; 30000]);
        assert_eq!(result, Err(ExecuteError::InfiniteLoop));
    }

    #[test]
    fn copy_input() {
        check_output(",.>,.>,.>,.>,.", "hello", "hello");
    }

    #[test]
    fn output_exclamation_mark() {
        check_output("+++++++++++++++++++++++++++++++++.", "", "!");
    }

    #[test]
    fn three_exclamation_marks() {
        check_output(">+++++++++++++++++++++++++++++++++<+++[>.<-]", "", "!!!");
    }

    #[test]
    fn hello_world() {
        check_output("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.", "", "Hello World!\n");
    }

    fn check_output(program_text: &str, input: &str, expected_output: &str) {
        let program = parse_program(program_text);
        match program {
            Ok(program) => {
                let result = program
                    .execute(input.to_string().into_bytes(), vec![0; 30000])
                    .expect(&format!("Cannot execute program {program_text}"));
                assert_eq!(result, expected_output);
            }
            Err(error) => {
                panic!("Error occurred while parsing program {program_text}: {error:?}");
            }
        }
    }
}
