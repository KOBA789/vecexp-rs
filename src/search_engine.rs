use index_file::IndexData;
use ::Morpheme;
use vm::{VM, IteratorScanner};
use itertools::Itertools;
use std::time::Instant;

macro_rules! stderr {
    ($($arg:tt)*) => (
        use std::io::Write;
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr (file handle closed?): {}", x),
        }
    )
}

pub struct SearchEngine<'a> {
  index_data: &'a IndexData,
  morphemes: &'a[Morpheme],
}

impl<'a> SearchEngine<'a> {
  pub fn new(index_data: &'a IndexData, morphemes: &'a [Morpheme]) -> SearchEngine<'a> {
    SearchEngine { index_data: index_data, morphemes: morphemes }
  }

  pub fn search(&self, query: Vec<String>) {
    let mut vm = VM::parse(query);
    let morphemes = self.morphemes;
    let index_data = &self.index_data;

    let now = Instant::now();
    for row_id in 0..morphemes.len() {
        let rest = morphemes[row_id..].into_iter();
        let mut scanner = IteratorScanner::new(rest);
        if let Some(ret) = vm.exec(&mut scanner) {
            print!("{}: ", ret);
            let head = &morphemes[row_id];
            let (begin, end) = index_data.sentence_index[head.sentence_id as usize];
            let context = &morphemes[begin..end+1].into_iter().map(|m| {
                ::std::str::from_utf8(
                    index_data.feature_indices[0][
                        (m.feature_ids[0] - 1) as usize
                    ].as_slice()
                ).unwrap()
            }).join("");
            println!("{}", context);
        }
        vm.reset();
    }
    let elapsed = now.elapsed();
    let ms = elapsed.as_secs() * 1_000 + (elapsed.subsec_nanos() / 1_000_000) as u64;
    stderr!("completed in {} ms", ms);
  }
}