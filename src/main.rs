use std::io;
use std::io::prelude::*;

enum OpCode<'a> {
    Expect(usize, &'a str, usize),
    Fail,
    Match(&'a str),
    Jump(usize),
    Next,
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

    fn exec<'b>(&'b mut self, scanner: &mut Scanner) -> Option<&str> {
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

trait Scanner {
    fn expect(&self, col: usize, pat: &str) -> Option<bool>;
    fn next(&mut self) -> bool;
}

struct OnMemoryScanner<'a> {
    input: &'a Vec<[&'a str; 10]>,
    position: usize,
}

impl<'a> OnMemoryScanner<'a> {
    fn new(input: &'a Vec<[&'a str; 10]>) -> OnMemoryScanner<'a> {
        OnMemoryScanner { input: input, position: 0 }
    }

    fn is_eos(&self) -> bool {
        self.input.len() <= self.position
    }
}

impl<'a> Scanner for OnMemoryScanner<'a> {
    fn expect(&self, col: usize, pat: &str) -> Option<bool> {
        if self.is_eos() {
            return None;
        }

        Some(self.input[self.position][col] == pat)
    }

    fn next<'b>(&'b mut self) -> bool {
        if self.is_eos() {
            return false;
        }

        self.position += 1;

        !self.is_eos()
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

    fn next<'b>(&'b mut self) -> bool {
        let mut buffer = String::new();
        match self.stdin.read_line(&mut buffer) {
            Ok(n) => {
                if n <= 0 {
                    return false;
                }

                let trim = buffer.trim();

                if trim == "EOS" {
                    return false;
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

    let mut scanner = StdinScanner::new(locked);
    let mut vm = VM::new(vec![
        OpCode::Jump(2),
        OpCode::Fail,
        OpCode::Next,
        OpCode::Expect(1, "名詞", 1),
        OpCode::Expect(2, "サ変接続", 1),
        OpCode::Next,
        OpCode::Expect(1, "動詞", 1),
        OpCode::Expect(5, "サ変・スル", 1),
        OpCode::Expect(6, "基本形", 13),
        OpCode::Next,
        OpCode::Expect(1, "記号", 1),
        OpCode::Expect(2, "句点", 1),
        OpCode::Match("現在形"),
        OpCode::Expect(6, "連用形", 1),
        OpCode::Next,
        OpCode::Expect(1, "助動詞", 1),
        OpCode::Expect(5, "特殊・タ", 1),
        OpCode::Expect(6, "基本形", 1),
        OpCode::Next,
        OpCode::Expect(1, "記号", 1),
        OpCode::Expect(2, "句点", 1),
        OpCode::Match("過去形"),
    ]);
    println!("{}", vm.exec(&mut scanner).unwrap());
}
