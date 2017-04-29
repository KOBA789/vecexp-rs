use ::{Feat, FeatureList, MORPHEME_SIZE, Morpheme, COLS};
use filebuffer::FileBuffer;
use features_file::FeaturesFile;
use sentence_index_file::{SentenceIndex, SentenceIndexFile};
use indexer::Indexer;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Instant;
use vm::VM;

pub struct Workspace {
    path: PathBuf,
}

pub struct IndexData<'a> {
    pub features_per_column: [FeatureList<'a>; COLS],
    pub sentence_index: SentenceIndex,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Workspace {
        Workspace {
            path: path,
        }
    }

    pub fn create_index(&self, source_path: PathBuf) -> io::Result<()> {
        fs::create_dir(&self.path)?;

        let indexer = Indexer::new(&self);

        indexer.execute(source_path)?;

        Ok(())
    }

    fn index_data<'a>(&self, pools: &'a mut Vec<Vec<u8>>) -> IndexData<'a> {
        *pools = vec![Vec::new(); 10];
        let mut features_per_column = init_array!(FeatureList, COLS, FeatureList::new());
        for (column, (mut pool, mut features)) in pools.iter_mut().zip(&mut features_per_column).enumerate() {
            *features = self.features_file(column).load(pool).unwrap();
        }

        let sentence_index = self.sentence_index_file().load().unwrap();

        IndexData {
            features_per_column: features_per_column,
            sentence_index: sentence_index,
        }
    }

    pub fn search(&mut self, query: Vec<String>) -> io::Result<()> {
        let inst = VM::parse(query);
        let body_buf = FileBuffer::open(&self.body_path())?;

        let mut pools = vec![];
        let index_data = self.index_data(&mut pools);

        let size: usize = body_buf.len() / MORPHEME_SIZE;
        let ptr: *const Morpheme = body_buf.as_ptr() as *const Morpheme;
        let morphemes: &[Morpheme] = unsafe { ::std::slice::from_raw_parts(ptr, size) };

        let vm = VM::new(inst.as_slice(), morphemes, &index_data);

        let now = Instant::now();

        vm.exec();

        let elapsed = now.elapsed();
        let ms = elapsed.as_secs() * 1_000 + (elapsed.subsec_nanos() / 1_000_000) as u64;
        println_stderr!("completed in {} ms", ms);
        Ok(())
    }

    pub fn lookup(&mut self, column: usize, pat: Feat) -> Option<usize> {
        let mut pool = Vec::new();
        let features = self.features_file(column).load(&mut pool).unwrap();
        for (i, feat) in features.into_iter().enumerate() {
            if feat == pat {
                return Some(i);
            }
        }

        None
    }

    pub fn body_path(&self) -> PathBuf {
        self.path.join("body.bin")
    }

    pub fn features_path(&self, column: usize) -> PathBuf {
        self.path.join(format!("features_{}.bin", column))
    }

    pub fn sentence_index_path(&self) -> PathBuf {
        self.path.join("sentence_index.bin")
    }

    pub fn features_file(&self, column: usize) -> FeaturesFile {
        FeaturesFile::new(self.features_path(column))
    }

    pub fn sentence_index_file(&self) -> SentenceIndexFile {
        SentenceIndexFile::new(self.sentence_index_path())
    }
}
