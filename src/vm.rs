use std::iter::Peekable;
use ::{FeatId, Morpheme};

type ResultCode = u32;

#[derive(Debug)]
pub enum OpCode {
    Expect(usize, FeatId, usize),
    Fail,
    Match(ResultCode),
    Jump(usize),
    Next,
    Noop,
}

pub struct VM {
    pc: usize,
    code: Vec<OpCode>,
}

enum State {
    Done(Option<ResultCode>),
    Going(usize)
}

impl<'a> VM {
    pub fn new(code: Vec<OpCode>) -> VM {
        VM { pc: 1, code: code }
    }

    pub fn parse(input: Vec<String>) -> VM {
        let mut code: Vec<OpCode> =  vec![];

        for op_str in input {
            let opcode_operand: Vec<&str> = op_str.split(":").collect();
            let operands = &opcode_operand[1..];
            code.push(
                match &opcode_operand[0][..] {
                    "Fail" => OpCode::Fail,
                    "Match" => OpCode::Match(operands[0].parse::<ResultCode>().unwrap()),
                    "Jump" => OpCode::Jump(operands[0].parse::<usize>().unwrap()),
                    "Expect" => OpCode::Expect(
                        operands[0].parse::<usize>().unwrap(),
                        operands[1].parse::<FeatId>().unwrap(),
                        operands[2].parse::<usize>().unwrap()),
                    "Next" => OpCode::Next,
                    "Noop" => OpCode::Noop,
                    _ => panic!("unsupported opcode")
                }
            );
        }

        VM::new(code)
    }

    pub fn reset(&mut self) {
        self.pc = 1;
    }

    pub fn exec(&mut self, scanner: &mut Scanner) -> Option<ResultCode> {
        while self.pc < self.code.len() {
            let ref op = self.code[self.pc];
            let ref next_state = match *op {
                OpCode::Fail => State::Done(None),
                OpCode::Match(ret) => State::Done(Some(ret)),
                OpCode::Jump(pc) => State::Going(pc),
                OpCode::Expect(col, pat, pc) => match scanner.expect(col, pat) {
                    Some(ret) => State::Going(if ret { self.pc + 1 } else { pc }),
                    None => State::Done(None),
                },
                OpCode::Next => match scanner.next() {
                    true => State::Going(self.pc + 1),
                    false => State::Done(None),
                },
                OpCode::Noop => State::Going(self.pc + 1)
            };

            match *next_state {
                State::Done(ret) => return ret,
                State::Going(pc) => self.pc = pc,
            };
        }

        panic!("out of bound");
    }
}

pub trait Scanner {
    fn expect(&mut self, col: usize, pat: FeatId) -> Option<bool>;
    fn next(&mut self) -> bool;
}

pub struct IteratorScanner<'a, T> where T: Iterator<Item = &'a Morpheme> {
    input: Peekable<T>,
    sentence_id: u32,
}

impl<'a, T> IteratorScanner<'a, T> where T: Iterator<Item = &'a Morpheme> {
    pub fn new(input: T) -> IteratorScanner<'a, T> {
        let mut peekable = input.peekable();
        let sentence_id = peekable.peek().unwrap().sentence_id;
        IteratorScanner { input: peekable, sentence_id: sentence_id }
    }

    fn peek(&mut self) -> Option<&Morpheme> {
        if let Some(morpheme) = self.input.peek() {
            if morpheme.sentence_id == self.sentence_id {
                return Some(morpheme);
            }
        }
        None
    }
}

impl<'a, T> Scanner for IteratorScanner<'a, T> where T: Iterator<Item = &'a Morpheme> {
    fn expect(&mut self, col: usize, feat_id: FeatId) -> Option<bool> {
        match self.peek() {
            Some(morpheme) => Some(morpheme.feature_ids[col] == feat_id),
            None => None,
        }
    }

    fn next(&mut self) -> bool {
        self.input.next();
        match self.peek() {
            Some(_) => true,
            None => false,
        }
    }
}
