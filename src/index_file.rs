use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};
use std::fs::File;
use std::path::PathBuf;

pub struct IndexFile {
    path: PathBuf,
}

pub struct IndexFileBody {
    feature_indices: [Vec<String>; ::COLS],
    sentence_index: Vec<(u32, u32)>,
}

impl IndexFile {
    pub fn new(path: PathBuf) -> IndexFile {
        IndexFile { path: path }
    }

    pub fn save(&self, index: [Vec<&[u8]>; ::COLS]) {
        let mut file = File::create(&self.path).unwrap();
        encode_into(&index, &mut file, SizeLimit::Infinite).unwrap();
    }

    pub fn load(&self) -> [Vec<String>; ::COLS] {
        let mut file = File::open(&self.path).unwrap();
        let indices: [Vec<String>; ::COLS] = decode_from(&mut file, SizeLimit::Infinite).unwrap();
        indices
    }
}
