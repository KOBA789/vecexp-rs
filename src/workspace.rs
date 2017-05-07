use ::{Feat, FeatureList, COLS, FeatId, BodyTable};
use features_file::FeaturesFile;
use sentence_index_file::{SentenceIndex, SentenceIndexFile};
use indexer::Indexer;
use vm::VM;

use filebuffer::FileBuffer;

use std::fs;
use std::io;
use std::path;
use std::time;

pub struct Workspace {
    path: path::PathBuf,
}

pub struct IndexData<'a> {
    pub features_per_column: [FeatureList<'a>; COLS],
    pub sentence_index: SentenceIndex,
}

impl Workspace {
    pub fn new(path: path::PathBuf) -> Workspace {
        Workspace {
            path: path,
        }
    }

    pub fn create_index(&self, source_path: path::PathBuf) -> io::Result<()> {
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

    pub fn search(&mut self, query: Vec<String>, limit: Option<usize>) -> io::Result<()> {
        let inst = VM::parse(query);

        let mut bufs = vec![];
        let body = BodyTable {
            columns: [
                self.load_column(&mut bufs, 0),
                self.load_column(&mut bufs, 1),
                self.load_column(&mut bufs, 2),
                self.load_column(&mut bufs, 3),
                self.load_column(&mut bufs, 4),
                self.load_column(&mut bufs, 5),
                self.load_column(&mut bufs, 6),
                self.load_column(&mut bufs, 7),
                self.load_column(&mut bufs, 8),
                self.load_column(&mut bufs, 9),
            ]
        };

        let mut pools = vec![];
        let index_data = self.index_data(&mut pools);

        let stdout = io::stdout();
        let handle = stdout.lock();
        let mut buffered = io::BufWriter::with_capacity(1024 * 1024, handle);

        let vm = VM::new(inst.as_slice(), body, &index_data);

        let now = time::Instant::now();

        vm.exec(&mut buffered, limit);

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

    fn load_column<'a>(&self, bufs: &mut Vec<FileBuffer>, column: usize) -> &'a [FeatId] {
        let buf = FileBuffer::open(self.body_path(column)).unwrap();
        let size: usize = buf.len() / ::std::mem::size_of::<FeatId>();
        let ptr: *const FeatId = buf.as_ptr() as *const FeatId;
        let feat_list: &[FeatId] = unsafe { ::std::slice::from_raw_parts(ptr, size) };
        bufs.push(buf);
        feat_list
    }

    pub fn body_path(&self, column: usize) -> path::PathBuf {
        self.path.join(format!("body_{}.bin", column))
    }

    pub fn features_path(&self, column: usize) -> path::PathBuf {
        self.path.join(format!("features_{}.bin", column))
    }

    pub fn sentence_index_path(&self) -> path::PathBuf {
        self.path.join("sentence_index.bin")
    }

    pub fn features_file(&self, column: usize) -> FeaturesFile {
        FeaturesFile::new(self.features_path(column))
    }

    pub fn sentence_index_file(&self) -> SentenceIndexFile {
        SentenceIndexFile::new(self.sentence_index_path())
    }
}
