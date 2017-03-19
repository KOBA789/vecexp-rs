mod vm;
mod index_file;
mod indexer;
mod workspace;

extern crate filebuffer;
extern crate byteorder;
extern crate rustc_serialize;
extern crate bincode;
#[macro_use]
extern crate clap;

use std::process;
use std::path::PathBuf;
use workspace::Workspace;

pub const COLS: usize = 10;

fn main() {
    let matches = clap_app!(vecexp =>
        (author: "KOBA789 <kobahide789@gmail.com>")
        (about: "Text mining tool by using RegExp-like query")
        (@arg workspace: +required "Sets workspace path")
        (@subcommand index =>
            (about: "create workspace & index")
            (@arg source: +required "Sets source file")
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
        // TODO: do search
    }
}
