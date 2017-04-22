use std::fs;
use std::io;
use std::path::PathBuf;
use indexer::Indexer;
use index_file::{IndexFile, IndexData};
use filebuffer::FileBuffer;
use ::{Morpheme, MORPHEME_SIZE};
use search_engine::SearchEngine;

pub struct Workspace {
    path: PathBuf,
    index_data_cache: Option<IndexData>,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Workspace {
        Workspace { path: path, index_data_cache: None }
    }

    pub fn create_index(&self, source_path: PathBuf) -> io::Result<()> {
        fs::create_dir(&self.path)?;

        let indexer = Indexer::new(&self);

        indexer.execute(source_path)?;

        Ok(())
    }

    pub fn search(&mut self, query: Vec<String>) -> io::Result<()> {
        let body_buf = FileBuffer::open(&self.body_path())?;
        let index_data = self.index_data();

        let size: usize = body_buf.len() / MORPHEME_SIZE;
        let ptr: *const Morpheme = body_buf.as_ptr() as *const Morpheme;
        let morphemes: &[Morpheme] = unsafe { ::std::slice::from_raw_parts(ptr, size) };

        let engine = SearchEngine::new(index_data, morphemes);
        engine.search(query);

        Ok(())
    }

    pub fn lookup(&mut self, column: usize, pat: Vec<u8>) -> Option<usize> {
        let index_data = self.index_data();
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

    pub fn index_data(&mut self) -> &IndexData {
        if self.index_data_cache.is_none() {
            let index_data = self.index_file().load();
            self.index_data_cache = Some(index_data);
        }
        return self.index_data_cache.as_ref().unwrap();
    }
}
