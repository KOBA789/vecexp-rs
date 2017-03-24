mod vm;
mod index_file;
mod indexer;
mod workspace;

extern crate filebuffer;
extern crate byteorder;
extern crate rustc_serialize;
extern crate bincode;
#[macro_use] extern crate clap;
extern crate linked_hash_map;
#[macro_use] extern crate itertools;

use std::process;
use std::path::PathBuf;
use workspace::Workspace;

use linked_hash_map::LinkedHashMap;

type FeatId = u32;
type Feat = Vec<u8>;
// TODO: use `std::mem::size_of::<FeatId>()`
pub const FeatIdSize: usize = 4;
pub const Cols: usize = 10;

#[derive(Debug)]
#[repr(packed)]
pub struct Morpheme {
    sentence_id: u32,
    feature_ids: [FeatId; Cols],
}

// TODO: use `std::mem::size_of::<Morpheme>()`
pub const MorphemeSize: usize = FeatIdSize * Cols + 4;

impl<'a> Morpheme {
    pub fn from_slice(slice: &'a [u8]) -> &'a Morpheme {
        let ptr: *const Self = (slice as *const [u8]) as *const Self;
        let value: &Morpheme = unsafe { &*ptr };
        value
    }

    pub fn as_slice(&self) -> &'a [u8] {
        let ptr: *const u8 = (self as *const Self) as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, MorphemeSize) }
    }

    pub fn new() -> Morpheme {
        Morpheme { sentence_id: 0, feature_ids: [0; Cols] }
    }

    pub fn with_sentence_id(sentence_id: u32) -> Morpheme {
        Morpheme { sentence_id: sentence_id, feature_ids: [0; Cols] }
    }
}

fn main() {
    let matches = clap_app!(vecexp =>
        (author: "KOBA789 <kobahide789@gmail.com>")
        (about: "Text mining tool by using RegExp-like query")
        (@arg workspace: +required "Sets workspace path")
        (@subcommand index =>
            (about: "create workspace & index")
            (@arg source: +required "Sets source file")
        )
        (@subcommand query =>
            (about: "query")
            (@arg opcode: +multiple "OpCode")
        )
        (@subcommand lookup =>
            (about: "lookup feature id")
            (@arg column: "Column")
            (@arg feature: "Feature")
        )
    ).get_matches();

    let workspace_path = PathBuf::from(matches.value_of("workspace").unwrap());
    let workspace = Workspace::new(workspace_path);

    if let Some(matches) = matches.subcommand_matches("index") {
        let source_path = PathBuf::from(matches.value_of("source").unwrap());
        println!("indexing...");
        match workspace.create_index(source_path) {
            Ok(()) => println!("fully indexed."),
            Err(err) => {
                println!("Error: {}", err);
                process::exit(1);
            },
        }
    } else if let Some(matches) = matches.subcommand_matches("query") {
        let opcodes: Vec<_> = matches.values_of("opcode").unwrap().map(|s| s.to_string()).collect();
        workspace.search(opcodes).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("lookup") {
        let column: usize = matches.value_of("column").unwrap().parse::<usize>().unwrap();
        let feature = matches.value_of("feature").unwrap().as_bytes().to_vec();
        match workspace.lookup(column, feature) {
            Some(feat_id) => println!("{}", feat_id),
            None => println!("not found."),
        }
    }
}
