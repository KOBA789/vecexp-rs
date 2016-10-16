use std::io;
use std::io::prelude::*;

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

        println!("out of bound");
        None
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
}

impl<'a> OnMemoryScanner<'a> {
    fn new(input: &'a [Morpheme]) -> OnMemoryScanner<'a> {
        OnMemoryScanner { input: input, position: 0 }
    }

    fn is_eos(&self) -> bool {
        self.input.len() <= self.position
    }

    fn consume(&mut self) -> &[Morpheme] {
        let ret = &self.input[..self.position+1];
        self.input = &self.input[self.position+1..];
        self.position = 0;

        ret
    }

    fn step(&mut self) {
        self.input = &self.input[1..];
        self.position = 0;
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
            let is_period = word[1] == "句点";
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
            //println!("{:?}", scanner.input);

            self.vm.reset();
            let result = self.vm.exec(&mut scanner);

            match result {
                Some(_) => {
                    let matched = scanner.consume().to_vec();
                    //self.sentence;

                    if matched.len() < 4 {
                        print!("*\t*\t*\t*\t*\t*\t*\t*\t*\t*\t");
                    }
                    for line in matched.iter().map(|m| m.join("\t")) {
                        print!("{}\t", line);
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

/*
検証,名詞,サ変接続,,,,,検証,ケンショウ,ケンショー
する,動詞,自立,,,サ変・スル,基本形,する,スル,スル
。,記号,句点,,,,,。,。,。
EOS

=> 現在形
*/

/*
検証,名詞,サ変接続,,,,,検証,ケンショウ,ケンショー
し,動詞,自立,,,サ変・スル,連用形,する,シ,シ
た,助動詞,,,,特殊・タ,基本形,た,タ,タ
。,記号,句点,,,,,。,。,。
EOS

=> 過去形
 */

fn main() {
    let stdin = io::stdin();
    let locked = stdin.lock();

    let scanner = StdinScanner::new(locked);
    let vm = VM::new(vec![
        OpCode::Jump(2),
        OpCode::Fail,
        OpCode::Expect(1, "名詞", 1),
        OpCode::Next,
        OpCode::Expect(1, "名詞", 7),
        OpCode::Expect(2, "接尾", 1),
        OpCode::Next,
        OpCode::Expect(1, "助詞", 1),
        OpCode::Next,
        OpCode::Expect(1, "動詞", 11), //9
        OpCode::Match("OK1"),
        OpCode::Expect(1, "名詞", 1),
        OpCode::Expect(3, "サ変接続", 1),
        OpCode::Match("OK2"),
        OpCode::Noop,
    ]);

    let mut finder = Finder::new(scanner, vm);
    finder.find();

    //println!("{}", vm.exec(&mut scanner).unwrap());
}
