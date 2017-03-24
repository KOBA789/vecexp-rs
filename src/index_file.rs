use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};
use std::fs::File;
use std::path::PathBuf;
use ::{Feat, FeatId, FeatIdSize, Cols};

pub struct IndexFile {
    path: PathBuf,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct IndexData {
    pub feature_indices: [Vec<Feat>; Cols],
    pub sentence_index: Vec<(usize, usize)>,
}

impl IndexFile {
    pub fn new(path: PathBuf) -> IndexFile {
        IndexFile { path: path }
    }

    pub fn save(&self, index: IndexData) {
        let mut file = File::create(&self.path).unwrap();
        encode_into(&index, &mut file, SizeLimit::Infinite).unwrap();
    }

    pub fn load(&self) -> IndexData {
        let mut file = File::open(&self.path).unwrap();
        let index: IndexData = decode_from(&mut file, SizeLimit::Infinite).unwrap();
        index
    }
}
