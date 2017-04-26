use ::{COLS, FeatureList};
use features_file::FeaturesFile;
use sentence_index_file::{SentenceIndex, SentenceIndexFile};
use std::path::PathBuf;

pub struct IndexData {
    path: PathBuf,
    //pub features_per_column: [FeatureList<'a>; COLS],
    //pub sentence_index: SentenceIndex,
}

impl IndexData {
    pub fn new(path: PathBuf) -> IndexData {
        IndexData {
            path: path,
            //features_per_column: init_array!(FeatureList<'a>, COLS, FeatureList::new()),
            //sentence_index: SentenceIndex::new(),
        }
    }

    // pub fn index_data(&mut self, features_pool: &'a mut [Vec<u8>; COLS]) -> IndexData<'a> {
    // let features_per_column: [FeatureList; COLS] = init_array_fn!(FeatureList, COLS, |i| {
    // self.features_file(i).load(&mut features_pool[i]).unwrap()
    // });
    //
    // let sentence_index = self.sentence_index_file().load().unwrap();
    //
    // let index_data = IndexData {
    // features_per_column: features_per_column,
    // sentence_index: sentence_index,
    // };
    //
    // index_data
    // }
    //
}
