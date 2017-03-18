use std::fs;
use std::io;
use std::path::PathBuf;
use indexer::Indexer;
use index_file::IndexFile;

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
