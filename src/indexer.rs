use byteorder::{ByteOrder, LittleEndian};
use filebuffer::FileBuffer;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::str;
use workspace::Workspace;

macro_rules! init_array(
    ($ty:ty, $len:expr, $val:expr) => (
        {
            let mut array: [$ty; $len] = unsafe { ::std::mem::uninitialized() };
            for i in array.iter_mut() {
                unsafe { ::std::ptr::write(i, $val); }
            }
            array
        }
    )
);

pub struct Indexer<'a> {
    workspace: &'a Workspace,
}

impl<'a> Indexer<'a> {
    pub fn new(workspace: &'a Workspace) -> Indexer<'a> {
        Indexer { workspace: workspace }
    }

    pub fn execute(&self, source_path: PathBuf) -> ::std::io::Result<()> {
        let body_path = self.workspace.body_path();

        let orig_buf = FileBuffer::open(&source_path)?;
        let mut out_file = BufWriter::new(File::create(body_path)?);

        let mut indices = init_array!(HashMap<&[u8], u32>, ::COLS, HashMap::new());

        let perline = orig_buf.split(|&c| c == b'\n').filter(|r| r.len() > 0);
        for line in perline {
            let mut row_bytes = [0; ::COLS * 4];
            for (i, col) in line.split(|&c| c == b',').enumerate() {
                let ref mut index = indices[i];
                let id = match index.get(col) {
                    Some(&id) => id,
                    None => {
                        let id = index.len() as u32 + 1;
                        index.insert(col, id);
                        id
                    }
                };
                LittleEndian::write_u32(&mut row_bytes[i * 4..i * 4 + 4], id);
            }
            out_file.write(&row_bytes)?;
        }

        let mut sorted_indices = init_array!(Vec<&[u8]>, ::COLS, vec![]);
        for (i, index) in indices.iter().enumerate() {
            let mut index_entries: Vec<_> = index.iter().collect();
            index_entries.sort_by_key(|&(_, id)| id);
            sorted_indices[i] = index_entries.iter().map(|&(&label, _)| label).collect();
            println!("  #{}: {} unique words", i, index.len());
        }

        {
            let index_file = self.workspace.index_file();
            index_file.save(sorted_indices);
        }

        Ok(())
    }
}
