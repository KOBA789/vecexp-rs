#[macro_use]
mod macros;
mod vm;
mod index;
mod workspace;

extern crate filebuffer;
#[macro_use]
extern crate clap;
extern crate linked_hash_map;


use std::path;
use std::process;
use workspace::Workspace;

type FeatId = u32;
type Feat<'a> = &'a [u8];
type FeatureList<'a> = Vec<Feat<'a>>;
// TODO: use `std::mem::size_of::<FeatId>()`
pub const FEAT_ID_SIZE: usize = 4;
pub const COLS: usize = 10;

#[derive(Debug)]
#[repr(packed)]
pub struct Morpheme {
    feature_ids: [FeatId; COLS],
}

pub struct BodyTable<'a> {
    columns: [&'a [FeatId]; COLS],
}

impl<'a> BodyTable<'a> {
    #[inline]
    fn len(&self) -> usize {
        return self.columns[0].len();
    }

    #[inline]
    fn slice(&self, begin: usize, end: usize) -> BodyTable<'a> {
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

// TODO: use `std::mem::size_of::<Morpheme>()`
pub const MORPHEME_SIZE: usize = FEAT_ID_SIZE * COLS;

impl<'a> Morpheme {
    pub fn from_slice(slice: &'a [u8]) -> &'a Morpheme {
        let ptr: *const Self = (slice as *const [u8]) as *const Self;
        let value: &Morpheme = unsafe { &*ptr };
        value
    }

    pub fn as_slice(&self) -> &'a [u8] {
        let ptr: *const u8 = (self as *const Self) as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, MORPHEME_SIZE) }
    }

    pub fn new() -> Morpheme {
        Morpheme { feature_ids: [0; COLS] }
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
            (@arg limit: -n --limit +takes_value "Limits the number of results")
            (@arg instseq: +multiple "InstSeq")
        )
        (@subcommand lookup =>
            (about: "lookup feature id")
            (@arg column: "Column")
            (@arg feature: "Feature")
        )
    )
        .get_matches();

    let workspace_path = path::PathBuf::from(matches.value_of("workspace").unwrap());
    let mut workspace = Workspace::new(workspace_path);

    if let Some(matches) = matches.subcommand_matches("index") {
        let source_path = path::PathBuf::from(matches.value_of("source").unwrap());
        println!("indexing...");
        match workspace.create_index(source_path) {
            Ok(()) => println!("fully indexed."),
            Err(err) => {
                println!("Error: {}", err);
                process::exit(1);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("query") {
        let opcodes: Vec<_> =
            matches.values_of("instseq").unwrap().map(|s| s.to_string()).collect();
        let limit: Option<usize> = matches.value_of("limit").map({
            |v| v.parse::<usize>().unwrap()
        });
        workspace.search(opcodes, limit).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("lookup") {
        let column: usize = matches.value_of("column").unwrap().parse::<usize>().unwrap();
        let feature = matches.value_of("feature").unwrap().as_bytes();
        match workspace.lookup(column, feature) {
            Some(feat_id) => println!("{}", feat_id),
            None => println!("not found."),
        }
    }
}
