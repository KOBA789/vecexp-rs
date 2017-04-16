use std::fs;
use std::io;
use std::path::PathBuf;
use indexer::Indexer;
use index_file::IndexFile;
use filebuffer::FileBuffer;
use vm::{VM, IteratorScanner};
use ::{Morpheme, MORPHEME_SIZE};
use itertools::Itertools;

pub struct Workspace {
    path: PathBuf,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Workspace {
        Workspace { path: path }
    }

    pub fn create_index(&self, source_path: PathBuf) -> io::Result<()> {
        fs::create_dir(&self.path)?;

        let indexer = Indexer::new(&self);

        indexer.execute(source_path)?;

        Ok(())
    }

    pub fn search(&self, query: Vec<String>) -> io::Result<()> {
        let mut vm = VM::parse(query);
        let body_buf = FileBuffer::open(&self.body_path())?;
        let index_data = self.index_file().load();

        let size: usize = body_buf.len() / MORPHEME_SIZE;
        let ptr: *const Morpheme = body_buf.as_ptr() as *const Morpheme;
        let morphemes: &[Morpheme] = unsafe { ::std::slice::from_raw_parts(ptr, size) };

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

        Ok(())
    }

    pub fn lookup(&self, column: usize, pat: Vec<u8>) -> Option<usize> {
        let index_data = self.index_file().load();
        let features = &index_data.feature_indices[column];
        for (i, feat) in features.into_iter().enumerate() {
            if *feat == pat {
                return Some(i + 1);
            }
        }

        None
    }

    pub fn index_path(&self) -> PathBuf {
        self.path.join("index.bin")
    }

    pub fn body_path(&self) -> PathBuf {
        self.path.join("body.bin")
    }

    pub fn index_file(&self) -> IndexFile {
        IndexFile::new(self.index_path())
    }
}
