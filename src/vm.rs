use index::{BodyTable, FeatId, IndexData};

use std::io;

#[derive(Debug)]
pub enum InstCode {
    Expect(usize, FeatId),
    Match,
    Jump(usize),
    Next,
    Split(usize, usize),
    Noop,
}

pub struct VM<'a> {
    inst_seq: &'a [InstCode],
    input: BodyTable<'a>,
    index_data: &'a IndexData<'a>,
}

impl<'a> VM<'a> {
    pub fn new(inst_seq: &'a [InstCode],
               input: BodyTable<'a>,
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
                "Match" => InstCode::Match,
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

    pub fn exec(&self, writer: &mut io::Write, limit: Option<usize>) -> Option<()> {
        let mut result_size = 0;

        'outer: for &(begin, end) in self.index_data.sentence_index.iter() {
            let sentence = &self.input.slice(begin as usize, end as usize);
            let mut context: Option<Vec<&[u8]>> = None;

            for sp in 0..sentence.len() {
                let ret = self.int_exec(sentence, 0, sp);
                if let Some(end_sp) = ret {
                    if context.is_none() {
                        let mut surface_list = Vec::<&[u8]>::with_capacity(sentence.len());
                        for &m in sentence.columns[0] {
                            surface_list.push(&self.index_data.features_per_column[0][m as usize]);
                        }

                        context = Some(surface_list);
                    }
                    let context = &context.as_ref().unwrap();
                    for &feat in &context[..sp] {
                        writer.write_all(feat).unwrap();
                    }
                    writer.write_all(b"\t").unwrap();
                    for (i, &column) in sentence.columns.iter().enumerate() {
                        for &m in column {
                            writer.write_all(&self.index_data.features_per_column[i][m as usize]).unwrap();
                            writer.write_all(b"\t").unwrap();
                        }
                    }
                    for &feat in &context[end_sp..] {
                        writer.write_all(feat).unwrap();
                    }
                    writer.write_all(b"\n").unwrap();

                    result_size += 1;
                    if let Some(actual_limit) = limit {
                        if actual_limit <= result_size {
                            break 'outer;
                        }
                    }
                }
            }
        }
        writer.flush().unwrap();
        None
    }

    fn int_exec(&self, sentence: &BodyTable, pc: usize, sp: usize) -> Option<usize> {
        let mut pc = pc;
        let mut sp = sp;

        while sp < sentence.len() && pc < self.inst_seq.len() {
            match self.inst_seq[pc] {
                InstCode::Expect(col, feat) => {
                    if sentence.columns[col][sp] == feat {
                        pc += 1;
                    } else {
                        return None;
                    }
                }
                InstCode::Next => {
                    sp += 1;
                    pc += 1;
                }
                InstCode::Jump(next_pc) => {
                    pc = next_pc;
                }
                InstCode::Split(x, y) => {
                    return self.int_exec(sentence, x, sp)
                        .or_else(|| self.int_exec(sentence, y, sp));
                }
                InstCode::Match => {
                    return Some(sp);
                }
                InstCode::Noop => {
                    pc += 1;
                }
            };
        }

        return None;
    }
}
