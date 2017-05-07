use index::{self, IndexFileBundle};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time;
use vm::VM;

pub struct Workspace {
    path: PathBuf,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Workspace {
        Workspace { path: path }
    }

    pub fn create_index(&self, source_path: PathBuf) -> io::Result<()> {
        fs::create_dir(&self.path)?;

        let indexer = index::Indexer::new(self);

        indexer.execute(source_path)?;

        Ok(())
    }

    pub fn search(&mut self, query: Vec<String>, limit: Option<usize>) -> io::Result<()> {
        let inst = VM::parse(query);

        let mut bufs = vec![];
        let body = self.body_table(&mut bufs);

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

    pub fn lookup(&mut self, column: usize, pat: String) -> io::Result<Option<usize>> {
        let pat_bytes = pat.as_bytes();
        self.features_file(column).lookup(pat_bytes)
    }
}

impl index::IndexFileBundle for Workspace {
    fn body_path(&self, column: usize) -> PathBuf {
        self.path.join(format!("body_{}.bin", column))
    }

    fn features_path(&self, column: usize) -> PathBuf {
        self.path.join(format!("features_{}.bin", column))
    }

    fn sentence_index_path(&self) -> PathBuf {
        self.path.join("sentence_index.bin")
    }
}
