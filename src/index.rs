use ::{COLS, FeatId, FeatureList};

use filebuffer::FileBuffer;
use linked_hash_map::LinkedHashMap;

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use workspace::Workspace;

type BorrowFeat<'a> = &'a [u8];

pub struct Indexer<'a> {
    workspace: &'a Workspace,
}

impl<'a> Indexer<'a> {
    pub fn new(workspace: &'a Workspace) -> Indexer<'a> {
        Indexer { workspace: workspace }
    }

    fn open_column_file(&self, column: usize) -> io::Result<io::BufWriter<fs::File>> {
        let path = self.workspace.body_path(column);
        Ok((io::BufWriter::new(fs::File::create(path)?)))
    }

    pub fn execute(&self, source_path: PathBuf) -> ::std::io::Result<()> {
        let orig_buf = FileBuffer::open(&source_path)?;

        let mut columns = Vec::with_capacity(COLS);
        for column in 0..COLS {
            columns.push(self.open_column_file(column)?);
        }

        let mut feature_id_map_bundle =
            init_array!(LinkedHashMap<BorrowFeat, FeatId>, COLS, LinkedHashMap::new());
        // FIXME: Hardcoded
        feature_id_map_bundle[0].insert("".as_bytes(), 0);
        feature_id_map_bundle[0].insert("。".as_bytes(), 1);
        feature_id_map_bundle[0].insert("◇".as_bytes(), 2);
        feature_id_map_bundle[0].insert("◆".as_bytes(), 3);
        feature_id_map_bundle[0].insert("▽".as_bytes(), 4);
        feature_id_map_bundle[0].insert("▼".as_bytes(), 5);
        feature_id_map_bundle[0].insert("△".as_bytes(), 6);
        feature_id_map_bundle[0].insert("▲".as_bytes(), 7);
        feature_id_map_bundle[0].insert("□".as_bytes(), 8);
        feature_id_map_bundle[0].insert("■".as_bytes(), 9);
        feature_id_map_bundle[0].insert("○".as_bytes(), 10);
        feature_id_map_bundle[0].insert("●".as_bytes(), 11);
        feature_id_map_bundle[0].insert("EOS".as_bytes(), 12);

        let mut current_sentence_head: u32 = 0;
        let mut sentence_index = Vec::<(u32, u32)>::new();

        let perline = orig_buf.split(|&c| c == b'\n').filter(|r| r.len() > 0);
        for (row_id, line) in perline.enumerate() {
            let row_id = row_id as u32;
            let cols = line.split(|&c| c == b',');
            let mut row: [FeatId; COLS] = [0; COLS];

            {
                let zipped = row.iter_mut().zip(cols.zip(feature_id_map_bundle.iter_mut()));
                for (mut column, (feat, mut feature_id_map)) in zipped {
                    let id = match feature_id_map.get(feat) {
                        Some(&id) => id,
                        None => {
                            let id = feature_id_map.len() as FeatId;
                            feature_id_map.insert(feat, id);
                            id
                        }
                    };
                    *column = id;
                }
            }

            {
                for (feat_id, mut column) in row.iter().zip(columns.iter_mut()) {
                    let ptr = (feat_id as *const u32) as *const u8;
                    column.write(unsafe { ::std::slice::from_raw_parts(ptr, 4) })?;
                }
                if row[0] <= 12 {
                    sentence_index.push((current_sentence_head, row_id + 1)); // +1 means exclusive range
                    current_sentence_head = row_id + 1;
                }
            }
        }

        {
            for (column, feature_id_map) in feature_id_map_bundle.into_iter().enumerate() {
                let features: Vec<&[u8]> = feature_id_map.keys().map(|&key| key).collect();
                let features_file = self.workspace.features_file(column);
                features_file.save(features)?;
            }

            let sentence_index_file = self.workspace.sentence_index_file();
            sentence_index_file.save(sentence_index)?;
        }
        Ok(())
    }
}

pub struct FeaturesFile {
    path: PathBuf,
}

impl FeaturesFile {
    pub fn new(path: PathBuf) -> FeaturesFile {
        FeaturesFile { path: path }
    }

    pub fn load<'a>(&self, mut pool: &'a mut Vec<u8>) -> io::Result<FeatureList<'a>> {
        let metadata = fs::metadata(&self.path)?;
        let file_len = metadata.len() as usize;

        let mut file = fs::File::open(&self.path)?;

        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let features_len = unsafe { ::std::mem::transmute::<_, u32>(len_buf) } as usize;

        let mut offsets = vec![0; features_len];
        file.read_exact(&mut offsets)?;

        *pool = vec![0; file_len - (4 + features_len)];
        file.read_exact(&mut pool)?;

        let mut features = Vec::<&[u8]>::with_capacity(features_len);

        let mut ptr: usize = 0;
        for offset in offsets {
            features.push(&pool[ptr..][..offset as usize]);
            ptr += offset as usize;
        }

        Ok(features)
    }

    pub fn save(&self, features: FeatureList) -> io::Result<()> {
        let mut file = fs::File::create(&self.path)?;
        let len_buf: [u8; 4] = unsafe { ::std::mem::transmute(features.len() as u32) };
        file.write_all(&len_buf)?;
        let mut offsets = Vec::<u8>::with_capacity(features.len());
        for feat in &features {
            offsets.push(feat.len() as u8);
        }
        file.write_all(&offsets)?;
        for feat in &features {
            file.write_all(feat)?;
        }
        file.flush()?;
        Ok(())
    }
}

pub struct SentenceIndexFile {
    path: PathBuf,
}

pub type SentenceIndex = Vec<(u32, u32)>;

impl SentenceIndexFile {
    pub fn new(path: PathBuf) -> SentenceIndexFile {
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
