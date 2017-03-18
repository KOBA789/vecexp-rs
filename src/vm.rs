use std::io;
use std::io::prelude::*;
use std::cmp;

type ValId = u32;
type Morpheme = [ValId];

#[derive(Debug)]
enum OpCode<'a> {
    Expect(usize, ValId, usize),
    Fail,
    Match(&'a str),
    Jump(usize),
    Next,
    Noop,
}

struct VM<'a> {
    pc: usize,
    code: Vec<OpCode<'a>>,
}

enum State<'a> {
    Done(Option<&'a str>),
    Going(usize)
}

impl<'a> VM<'a> {
    fn new(code: Vec<OpCode<'a>>) -> VM<'a> {
        VM { pc: 0, code: code }
    }

    fn parse(input: &'a Vec<String>) -> VM<'a> {
        let mut code: Vec<OpCode<'a>> =  vec![];

        for op_str in input {
            let opcode_operand: Vec<&str> = op_str.split(":").collect();
            let operands = &opcode_operand[1..];
            code.push(
                match &opcode_operand[0][..] {
                    "Fail" => OpCode::Fail,
                    "Match" => OpCode::Match(operands[0]),
                    "Jump" => OpCode::Jump(operands[0].parse::<usize>().unwrap()),
                    "Expect" => OpCode::Expect(
                        operands[0].parse::<usize>().unwrap(),
                        operands[1].parse::<ValId>().unwrap(),
                        operands[2].parse::<usize>().unwrap()),
                    "Next" => OpCode::Next,
                    "Noop" => OpCode::Noop,
                    _ => panic!("unsupported opcode")
                }
            );
        }

        VM::new(code)
    }

    fn reset(&mut self) {
        self.pc = 0;
    }

    fn exec(&mut self, scanner: &mut Scanner) -> Option<&str> {
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

trait Scanner {
    fn expect(&self, col: usize, pat: ValId) -> Option<bool>;
    fn next(&mut self) -> bool;
    fn peek(&self) -> &Morpheme;
}

struct SliceScanner<'a> {
    input: &'a [Morpheme],
    position: usize,
    start: usize,
}

impl<'a> SliceScanner<'a> {
    fn new(input: &'a [Morpheme]) -> SliceScanner<'a> {
        SliceScanner { input: input, position: 0, start: 0 }
    }

    fn is_eos(&self) -> bool {
        self.input.len() <= self.position
    }

    fn consume(&mut self) -> (&[Morpheme], &[Morpheme], &[Morpheme]) {
        let pre = &self.input[0..self.start];
        let post = &self.input[self.position+1..];
        let ret = &self.input[self.start..self.position+1];
        self.start = self.position;

        (pre, ret, post)
    }

    fn step(&mut self) {
        self.start += 1;
        self.position = self.start;
    }
}

impl<'a> Scanner for SliceScanner<'a> {
    fn expect(&self, col: usize, pat: ValId) -> Option<bool> {
        if self.is_eos() {
            return None;
        }

        Some(self.input[self.position][col] == pat)
    }

    fn next(&mut self) -> bool {
        if self.is_eos() {
            return false;
        }

        self.position += 1;

        !self.is_eos()
    }

    fn peek(&self) -> &Morpheme {
        &self.input[self.position]
    }
}
