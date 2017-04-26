use std::fs::{self, File};
use std::io::{BufWriter, Read, Result, Write};
use std::path::PathBuf;

pub struct SentenceIndexFile {
    path: PathBuf,
}

pub type SentenceIndex = Vec<(u32, u32)>;

impl SentenceIndexFile {
    pub fn new(path: PathBuf) -> SentenceIndexFile {
        SentenceIndexFile { path: path }
    }

    pub fn save(&self, sentence_index: SentenceIndex) -> Result<()> {
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);
        for (begin, end) in sentence_index {
            let bp = &begin as *const u32 as *const u8;
            let ep = &end as *const u32 as *const u8;
            writer.write_all(unsafe { ::std::slice::from_raw_parts(bp, 4) })?;
            writer.write_all(unsafe { ::std::slice::from_raw_parts(ep, 4) })?;
        }

        Ok(())
    }

    pub fn load(&self) -> Result<SentenceIndex> {
        let metadata = fs::metadata(&self.path)?;
        let file_len = metadata.len() as usize;
        let mut sentence_index = SentenceIndex::with_capacity(file_len / 8);

        let mut file = File::open(&self.path)?;
        let mut buf: [u8; 8] = [0; 8];
        while let Ok(()) = file.read_exact(&mut buf) {
            let begin: &u32 = unsafe { ::std::mem::transmute(&buf[..4] as *const [u8] as *const u8) };
            let end: &u32 = unsafe { ::std::mem::transmute(&buf[4..] as *const [u8] as *const u8) };
            sentence_index.push((*begin, *end));
        }

        Ok(sentence_index)
    }
}
