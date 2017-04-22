use ::{FeatId, Morpheme};
use index_file::IndexData;
use std::io::{self, Write};

type ResultCode = u32;

#[derive(Debug)]
pub enum InstCode {
    Expect(usize, FeatId),
    Match(ResultCode),
    Jump(usize),
    Next,
    Split(usize, usize),
    Noop,
}

pub struct VM<'a> {
    inst_seq: Vec<InstCode>,
    input: &'a [Morpheme],
    index_data: &'a IndexData,
}

impl<'a> VM<'a> {
    pub fn new(inst_seq: Vec<InstCode>,
               input: &'a [Morpheme],
               index_data: &'a IndexData)
               -> VM<'a> {
        VM {
            inst_seq: inst_seq,
            input: input,
            index_data: index_data,
        }
    }

    pub fn parse(input: Vec<String>) -> Vec<InstCode> {
        let mut inst_seq: Vec<InstCode> = vec![];

        for op_str in input {
            let opcode_operand: Vec<&str> = op_str.split(":").collect();
            let operands = &opcode_operand[1..];
            inst_seq.push(match &opcode_operand[0][..] {
                "Match" => InstCode::Match(operands[0].parse::<ResultCode>().unwrap()),
                "Jump" => InstCode::Jump(operands[0].parse::<usize>().unwrap()),
                "Expect" => {
                    InstCode::Expect(operands[0].parse::<usize>().unwrap(),
                                     operands[1].parse::<FeatId>().unwrap())
                }
                "Split" => {
                    InstCode::Split(operands[0].parse::<usize>().unwrap(),
                                    operands[1].parse::<usize>().unwrap())
                }
                "Next" => InstCode::Next,
                "Noop" => InstCode::Noop,
                _ => panic!("unsupported opcode"),
            });
        }

        inst_seq
    }

    pub fn exec(&self) -> Option<ResultCode> {
        for &(begin, end) in self.index_data.sentence_index.iter() {
            let sentence = &self.input[begin..end + 1];
            let mut context: Option<Vec<u8>> = None;
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            for sp in 0..sentence.len() {
                let ret = self.int_exec(sentence, 0, sp);
                if ret {
                    if context.is_none() {
                        let mut surface_list = Vec::<&[u8]>::with_capacity(sentence.len());
                        let (_, size) = sentence.into_iter().fold((&mut surface_list, 0), |(list, size), m| {
                            let surface = &self.index_data.features_per_column[0][m.feature_ids[0] as usize].as_slice();
                            list.push(surface);
                            (list, size + surface.len())
                        });
                        let mut whole_surface = Vec::<u8>::with_capacity(size);
                        for surface in &mut surface_list {
                            whole_surface.write_all(surface).unwrap();
                        }
                        context = Some(whole_surface);
                    }
                    handle.write_all(context.as_ref().unwrap()).unwrap();
                    handle.write_all(b"\n").unwrap();
                }
            }
        }
        None
    }

    fn int_exec(&self, sentence: &[Morpheme], pc: usize, sp: usize) -> bool {
        let mut pc = pc;
        let mut sp = sp;

        while pc < sentence.len() && sp < sentence.len() {
            match self.inst_seq[pc] {
                InstCode::Expect(col, feat) => {
                    if sentence[sp].feature_ids[col] == feat {
                        pc += 1;
                    } else {
                        return false;
                    }
                }
                InstCode::Match(_) => {
                    return true;
                }
                InstCode::Jump(next_pc) => {
                    pc = next_pc;
                }
                InstCode::Next => {
                    sp += 1;
                    pc += 1;
                }
                InstCode::Noop => {
                    pc += 1;
                }
                InstCode::Split(x, y) => {
                    return self.int_exec(sentence, x, sp) || self.int_exec(sentence, y, sp);
                }
            };
        }

        return false;
    }
}
