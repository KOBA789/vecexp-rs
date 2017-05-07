use std::fs;
use std::io::{self, Read, Write};
use std::path;

pub struct SentenceIndexFile {
    path: path::PathBuf,
}

pub type SentenceIndex = Vec<(u32, u32)>;

impl SentenceIndexFile {
    pub fn new(path: path::PathBuf) -> SentenceIndexFile {
        SentenceIndexFile { path: path }
    }

    pub fn save(&self, sentence_index: SentenceIndex) -> io::Result<()> {
        let file = fs::File::create(&self.path)?;
        let mut writer = io::BufWriter::new(file);
        for (begin, end) in sentence_index {
            let bp = &begin as *const u32 as *const u8;
            let ep = &end as *const u32 as *const u8;
            writer.write_all(unsafe { ::std::slice::from_raw_parts(bp, 4) })?;
            writer.write_all(unsafe { ::std::slice::from_raw_parts(ep, 4) })?;
        }

        Ok(())
    }

    pub fn load(&self) -> io::Result<SentenceIndex> {
        let metadata = fs::metadata(&self.path)?;
        let file_len = metadata.len() as usize;
        let mut sentence_index = SentenceIndex::with_capacity(file_len / 8);

        let mut file = fs::File::open(&self.path)?;
        let mut buf: [u8; 8] = [0; 8];
        while let Ok(()) = file.read_exact(&mut buf) {
            let begin: &u32 = unsafe { ::std::mem::transmute(&buf[..4] as *const [u8] as *const u8) };
            let end: &u32 = unsafe { ::std::mem::transmute(&buf[4..] as *const [u8] as *const u8) };
            sentence_index.push((*begin, *end));
        }

        Ok(sentence_index)
    }
}
