use filebuffer::FileBuffer;
use linked_hash_map::LinkedHashMap;

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

pub type FeatId = u32;
pub type Feat<'a> = &'a [u8];
pub type FeatList<'a> = Vec<Feat<'a>>;
pub const COLS: usize = 10;

type BorrowFeat<'a> = &'a [u8];

pub trait IndexFileBundle {
    fn body_path(&self, usize) -> PathBuf;
    fn features_path(&self, usize) -> PathBuf;
    fn sentence_index_path(&self) -> PathBuf;

    fn features_file(&self, column: usize) -> FeaturesFile {
        FeaturesFile::new(self.features_path(column))
    }

    fn sentence_index_file(&self) -> SentenceIndexFile {
        SentenceIndexFile::new(self.sentence_index_path())
    }

    fn index_data<'a>(&self, pools: &'a mut Vec<Vec<u8>>) -> IndexData<'a> {
        *pools = vec![Vec::new(); 10];
        let mut features_per_column = init_array!(FeatList, COLS, FeatList::new());
        for (column, (mut pool, mut features)) in
            pools.iter_mut().zip(&mut features_per_column).enumerate() {
            *features = self.features_file(column).load(pool).unwrap();
        }

        let sentence_index = self.sentence_index_file().load().unwrap();

        IndexData {
            features_per_column: features_per_column,
            sentence_index: sentence_index,
        }
    }

    unsafe fn load_column<'a>(&self, mut bufs: &mut Vec<FileBuffer>, column: usize) -> &'a [FeatId] {
        let buf = FileBuffer::open(self.body_path(column)).unwrap();
        let size: usize = buf.len() / ::std::mem::size_of::<FeatId>();
        let ptr: *const FeatId = buf.as_ptr() as *const FeatId;
        let feat_list: &'a [FeatId] = ::std::slice::from_raw_parts(ptr, size);
        bufs.push(buf);
        feat_list
    }

    fn body_table<'a>(&self, mut bufs: &'a mut Vec<FileBuffer>) -> BodyTable<'a> {
        unsafe {
            BodyTable {
                columns: [
                    self.load_column(bufs, 0),
                    self.load_column(bufs, 1),
                    self.load_column(bufs, 2),
                    self.load_column(bufs, 3),
                    self.load_column(bufs, 4),
                    self.load_column(bufs, 5),
                    self.load_column(bufs, 6),
                    self.load_column(bufs, 7),
                    self.load_column(bufs, 8),
                    self.load_column(bufs, 9)
                ],
            }
        }
    }
}

pub struct Indexer<'a> {
    bundle: &'a IndexFileBundle,
}

impl<'a> Indexer<'a> {
    pub fn new(bundle: &'a IndexFileBundle) -> Indexer<'a> {
        Indexer { bundle: bundle }
    }

    fn open_column_file(&self, column: usize) -> io::Result<io::BufWriter<fs::File>> {
        let path = self.bundle.body_path(column);
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
                let features_file = self.bundle.features_file(column);
                features_file.save(features)?;
            }

            let sentence_index_file = self.bundle.sentence_index_file();
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

    pub fn load<'a>(&self, mut pool: &'a mut Vec<u8>) -> io::Result<FeatList<'a>> {
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

    pub fn save(&self, features: FeatList) -> io::Result<()> {
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

    pub fn lookup(&self, pat: Feat) -> io::Result<Option<usize>> {
        let mut pool = Vec::new();
        let features = self.load(&mut pool)?;
        for (i, feat) in features.into_iter().enumerate() {
            if feat == pat {
                return Ok(Some(i));
            }
        }
        Ok(None)
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

pub struct IndexData<'a> {
    pub features_per_column: [FeatList<'a>; COLS],
    pub sentence_index: SentenceIndex,
}

pub struct BodyTable<'a> {
    pub columns: [&'a [FeatId]; COLS],
}

impl<'a> BodyTable<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        return self.columns[0].len();
    }

    #[inline]
    pub fn slice(&self, begin: usize, end: usize) -> BodyTable<'a> {
        BodyTable {
            columns: [&self.columns[0][begin..end],
                      &self.columns[1][begin..end],
                      &self.columns[2][begin..end],
                      &self.columns[3][begin..end],
                      &self.columns[4][begin..end],
                      &self.columns[5][begin..end],
                      &self.columns[6][begin..end],
                      &self.columns[7][begin..end],
                      &self.columns[8][begin..end],
                      &self.columns[9][begin..end]],
        }
    }
}
