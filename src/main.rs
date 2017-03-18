// mod vm;
mod index_file;
mod indexer;
mod workspace;

extern crate filebuffer;
extern crate byteorder;
extern crate rustc_serialize;
extern crate bincode;

use std::env;
use std::path::PathBuf;
use workspace::Workspace;

pub const COLS: usize = 10;
type Morpheme<'a> = [u32; COLS];

fn main() {
    let args: Vec<_> = env::args().skip(1).take(2).collect();
    let workspace_path = PathBuf::from(args[0].clone());
    let source_path = PathBuf::from(args[1].clone());

    let workspace = Workspace::new(workspace_path);

    println!("indexing...");

    workspace.create_index(source_path).unwrap();

    println!("fully indexed.");
}
