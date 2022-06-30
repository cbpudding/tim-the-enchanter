use std::{collections::VecDeque, error::Error, fmt};

pub struct Interpreter {
    code: Vec<Symbol>,
    counter: usize,
    memory: [u8; 256],
    output: Vec<char>,
    pointer: u8
}

#[derive(Debug)]
pub enum InterpreterError {
    UnbalancedBrackets,
    TickLimitReached
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl Error for InterpreterError {}

enum Symbol {
    Right, // >
    Left, // <
    Increment, // +
    Decrement, // -
    Output, // .
    Input, // ,
    BranchZero(usize), // [
    BranchNonzero(usize) // ]
}

impl Interpreter {
    pub fn output(&self) -> String {
        self.output.iter().cloned().collect()
    }

    pub fn run(&mut self, len: usize) -> Result<String, InterpreterError> {
        let mut lcv = 0;
        while lcv < len && self.counter < self.code.len() {
            self.tick();
            lcv += 1;
        }
        if lcv < len {
            Ok(self.output())
        } else {
            Err(InterpreterError::TickLimitReached)
        }
    }

    pub fn tick(&mut self) {
        if self.counter < self.code.len() {
            match self.code[self.counter] {
                Symbol::Right => unsafe { self.pointer = self.pointer.unchecked_add(1) },
                Symbol::Left => unsafe { self.pointer = self.pointer.unchecked_sub(1) },
                Symbol::Increment => unsafe { self.memory[self.pointer as usize] = self.memory[self.pointer as usize].unchecked_add(1) },
                Symbol::Decrement => unsafe { self.memory[self.pointer as usize] = self.memory[self.pointer as usize].unchecked_sub(1) },
                Symbol::Output => self.output.push(self.memory[self.pointer as usize] as char),
                Symbol::Input => self.memory[self.pointer as usize] = 0,
                Symbol::BranchZero(branch) => if self.memory[self.pointer as usize] == 0 {
                    self.counter = branch;
                },
                Symbol::BranchNonzero(branch) => if self.memory[self.pointer as usize] != 0 {
                    self.counter = branch;
                }
            }
            self.counter += 1;
        }
    }
}

impl TryFrom<String> for Interpreter {
    type Error = InterpreterError;

    fn try_from(src: String) -> Result<Self, Self::Error> {
        let mut code = Vec::new();
        let mut stack = VecDeque::new();
        for c in src.chars() {
            match c {
                '>' => code.push(Symbol::Right),
                '<' => code.push(Symbol::Left),
                '+' => code.push(Symbol::Increment),
                '-' => code.push(Symbol::Decrement),
                '.' => code.push(Symbol::Output),
                ',' => code.push(Symbol::Input),
                '[' => {
                    stack.push_back(code.len());
                    code.push(Symbol::BranchZero(0));
                }
                ']' => {
                    if let Some(matching) = stack.pop_back() {
                        code[matching] = Symbol::BranchZero(code.len());
                        code.push(Symbol::BranchNonzero(matching));
                    } else {
                        return Err(InterpreterError::UnbalancedBrackets);
                    }
                }
                _ => {}
            }
        }
        if stack.len() > 0 {
            return Err(InterpreterError::UnbalancedBrackets);
        }
        Ok(Self {
            code,
            counter: 0,
            memory: [0; 256],
            output: Vec::new(),
            pointer: 0
        })
    }
}