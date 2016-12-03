use std::io;
use std::io::prelude::*;
use std::cmp;

#[derive(Debug)]
enum OpCode<'a> {
    Expect(usize, &'a str, usize),
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
                        operands[1],
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

type Morpheme = Vec<String>;

trait Scanner {
    fn expect(&self, col: usize, pat: &str) -> Option<bool>;
    fn next(&mut self) -> bool;
    fn peek(&self) -> &Morpheme;
}

struct OnMemoryScanner<'a> {
    input: &'a [Morpheme],
    position: usize,
    start: usize,
}

impl<'a> OnMemoryScanner<'a> {
    fn new(input: &'a [Morpheme]) -> OnMemoryScanner<'a> {
        OnMemoryScanner { input: input, position: 0, start: 0 }
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

impl<'a> Scanner for OnMemoryScanner<'a> {
    fn expect(&self, col: usize, pat: &str) -> Option<bool> {
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

struct Finder<'a, S> {
    scanner: S,
    vm: VM<'a>,
    sentence: Vec<Morpheme>,
    is_eos: bool,
}

impl<'a, S: Scanner> Finder<'a, S> {
    fn new(scanner: S, vm: VM<'a>) -> Finder<'a, S> {
        Finder {
            scanner: scanner,
            vm: vm,
            sentence: Vec::with_capacity(50),
            is_eos: false,
        }
    }

    fn read_sentence(&mut self) -> bool {
        if self.is_eos { return false; }
        self.sentence.truncate(0);

        while self.scanner.next() {
            let word: Vec<String> = self.scanner.peek().clone();
            let is_period = word[2] == "句点" ||
                word[0] == "◇" ||
                word[0] == "◆" ||
                word[0] == "▽" ||
                word[0] == "▼" ||
                word[0] == "△" ||
                word[0] == "▲" ||
                word[0] == "□" ||
                word[0] == "■" ||
                word[0] == "○" ||
                word[0] == "●" ||
                word[0] == "【" ||
                word[0] == "】";
            self.sentence.push(word);

            if is_period {
                return true;
            }
        }

        self.is_eos = true;
        return true;
    }

    fn next_sentence(&mut self) -> bool {
        let is_going = self.read_sentence();
        if !is_going {
            return false;
        }

        let mut scanner = OnMemoryScanner::new(&self.sentence.as_slice());
        while !scanner.is_eos() {
            self.vm.reset();
            let result = self.vm.exec(&mut scanner);

            match result {
                Some(_) => {
                    let win_size = 3;

                    let (pre_match, matched, post_match) = scanner.consume();

                    let pre_fixed_pos = if win_size > pre_match.len() {
                        0
                    } else {
                        pre_match.len() - win_size
                    };
                    let pre_fixed = &pre_match[pre_fixed_pos..];
                    let pre_outer = &pre_match[0..pre_fixed_pos];

                    let post_fixed_pos = cmp::min(win_size, post_match.len());
                    let post_fixed = &post_match[0..post_fixed_pos];
                    let post_outer = &post_match[post_fixed_pos..];

                    print!("|");
                    for pre in pre_outer.iter().map(|m| m[0].clone()) {
                        print!("{}|", pre);
                    }
                    print!("\t");
                    for _ in 0..(win_size - pre_fixed.len()) {
                        print!("\t");
                    }
                    for pre in pre_fixed.iter().map(|m| m[0].clone()) {
                        print!("{}\t", pre);
                    }

                    for line in matched.iter().map(|m| m.join("\t")) {
                        print!("{}\t", line);
                    }

                    for post in post_fixed.iter().map(|m| m[0].clone()) {
                        print!("{}\t", post);
                    }
                    for _ in 0..(win_size - post_fixed.len()) {
                        print!("\t");
                    }
                    print!("|");
                    for post in post_outer.iter().map(|m| m[0].clone()) {
                        print!("{}|", post);
                    }

                    println!("");
                },
                None => {
                    scanner.step();
                },
            }
        }

        return true;
    }

    fn find(&mut self) {
        while self.next_sentence() {
        }
    }
}

struct StdinScanner<'a> {
    stdin: io::StdinLock<'a>,
    row: Vec<String>,
}

impl<'a> StdinScanner<'a> {
    fn new(stdin: io::StdinLock<'a>) -> StdinScanner<'a>{
        let row = vec![];
        StdinScanner { stdin: stdin, row: row }
    }
}

impl<'a> Scanner for StdinScanner<'a> {
    fn expect(&self, col: usize, pat: &str) -> Option<bool> {
        Some(self.row[col].as_str() == pat)
    }

    fn peek(&self) -> &Vec<String> {
        &self.row
    }

    fn next(&mut self) -> bool {
        let mut buffer = String::new();
        match self.stdin.read_line(&mut buffer) {
            Ok(n) => {
                if n <= 0 {
                    return false;
                }

                let trim = buffer.trim();

                if trim == "EOS" {
                    self.row = vec![
                        "。".to_string(),
                        "記号".to_string(),
                        "句点".to_string(),
                        "*".to_string(),
                        "*".to_string(),
                        "*".to_string(),
                        "*".to_string(),
                        "。".to_string(),
                        "。".to_string(),
                        "。".to_string(),
                    ];
                    return true;
                }

                let split = trim.split(',');
                self.row = split.map(|s| s.to_string()).collect::<Vec<String>>();

                true
            }
            Err(error) => panic!(error),
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let locked = stdin.lock();

    let scanner = StdinScanner::new(locked);

    let args: Vec<String> = std::env::args().collect();

    let input = args.as_slice()[1..].to_vec();
    let vm = VM::parse(&input);

    let mut finder = Finder::new(scanner, vm);
    finder.find();
}
