use ::{FeatureList};
use std::fs::{self, File};
use std::io::{Read, Result, Write};
use std::path::PathBuf;

pub struct FeaturesFile {
    path: PathBuf,
}

impl FeaturesFile {
    pub fn new(path: PathBuf) -> FeaturesFile {
        FeaturesFile { path: path }
    }

    pub fn load<'a>(&self, mut pool: &'a mut Vec<u8>) -> Result<FeatureList<'a>> {
        let metadata = fs::metadata(&self.path)?;
        let file_len = metadata.len() as usize;

        let mut file = File::open(&self.path)?;

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

    pub fn save(&self, features: FeatureList) -> Result<()> {
        let mut file = File::create(&self.path)?;
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
