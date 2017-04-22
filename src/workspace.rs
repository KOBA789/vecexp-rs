use ::{MORPHEME_SIZE, Morpheme};
use filebuffer::FileBuffer;
use index_file::{IndexData, IndexFile};
use indexer::Indexer;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Instant;
use vm::VM;

pub struct Workspace {
    path: PathBuf,
    index_data_cache: Option<IndexData>,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Workspace {
        Workspace {
            path: path,
            index_data_cache: None,
        }
    }

    pub fn create_index(&self, source_path: PathBuf) -> io::Result<()> {
        fs::create_dir(&self.path)?;

        let indexer = Indexer::new(&self);

        indexer.execute(source_path)?;

        Ok(())
    }

    pub fn search(&mut self, query: Vec<String>) -> io::Result<()> {
        let inst = VM::parse(query);
        let body_buf = FileBuffer::open(&self.body_path())?;
        let index_data = self.index_data();

        let size: usize = body_buf.len() / MORPHEME_SIZE;
        let ptr: *const Morpheme = body_buf.as_ptr() as *const Morpheme;
        let morphemes: &[Morpheme] = unsafe { ::std::slice::from_raw_parts(ptr, size) };

        let vm = VM::new(inst, morphemes, index_data);

        let now = Instant::now();

        vm.exec();

        let elapsed = now.elapsed();
        let ms = elapsed.as_secs() * 1_000 + (elapsed.subsec_nanos() / 1_000_000) as u64;
        println_stderr!("completed in {} ms", ms);
        Ok(())
    }

    pub fn lookup(&mut self, column: usize, pat: Vec<u8>) -> Option<usize> {
        let index_data = self.index_data();
        let features = &index_data.features_per_column[column];
        println!("{:?}", features);
        for (i, feat) in features.into_iter().enumerate() {
            if *feat == pat {
                return Some(i);
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

    pub fn index_data(&mut self) -> &IndexData {
        if self.index_data_cache.is_none() {
            let index_data = self.index_file().load();
            self.index_data_cache = Some(index_data);
        }
        return self.index_data_cache.as_ref().unwrap();
    }
}
