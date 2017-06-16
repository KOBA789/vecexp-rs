#[macro_use]
mod macros;
mod vm;
mod index;
mod compiler;
mod workspace;

extern crate filebuffer;
#[macro_use]
extern crate clap;
extern crate linked_hash_map;
extern crate combine;

use std::path::PathBuf;
use std::process;
use workspace::Workspace;

fn main() {
    let matches = clap_app!(vecexp =>
        (author: "KOBA789 <kobahide789@gmail.com>")
        (about: "Text mining tool by using RegExp-like query")
        (@arg workspace: +required "Sets workspace path")
        (@subcommand index =>
            (about: "create workspace & index")
            (@arg source: +required "Sets source file")
        )
        (@subcommand query_iseq =>
            (about: "query-iseq")
            (@arg limit: -n --limit +takes_value "Limits the number of results")
            (@arg instseq: +multiple "InstSeq")
        )
        (@subcommand query =>
            (about: "query")
            (@arg limit: -n --limit +takes_value "Limits the number of results")
            (@arg query: "Query")
        )
        (@subcommand lookup =>
            (about: "lookup feature id")
            (@arg column: "Column")
            (@arg feature: "Feature")
        )
        (@subcommand decode =>
            (about: "decode feature id to feature surface")
            (@arg column: "Column")
            (@arg feature_id: "Feature ID")
        )
    ).get_matches();

    let workspace_path = PathBuf::from(matches.value_of("workspace").unwrap());
    let mut workspace = Workspace::new(workspace_path);

    if let Some(matches) = matches.subcommand_matches("index") {
        let source_path = PathBuf::from(matches.value_of("source").unwrap());
        println_stderr!("indexing...");
        try!(workspace.create_index(source_path));
        println_stderr!("fully indexed.")
    } else if let Some(matches) = matches.subcommand_matches("query_iseq") {
        let opcodes: Vec<_> =
            matches.values_of("instseq").unwrap().map(|s| s.to_string()).collect();
        let limit: Option<usize> = matches.value_of("limit").map({
            |v| v.parse::<usize>().unwrap()
        });
        try!(workspace.search2(opcodes, limit));
    } else if let Some(matches) = matches.subcommand_matches("query") {
        let query = matches.value_of("query").unwrap();
        let limit: Option<usize> = matches.value_of("limit").map({
            |v| v.parse::<usize>().unwrap()
        });
        let tree = compiler::parse(query);
        let iseq = compiler::compile(tree);
        try!(workspace.search(iseq, limit));
    } else if let Some(matches) = matches.subcommand_matches("lookup") {
        let column: usize = matches.value_of("column").unwrap().parse::<usize>().unwrap();
        let feature = String::from(matches.value_of("feature").unwrap());
        match try!(workspace.lookup(column, feature)) {
            Some(feat_id) => println!("{}", feat_id),
            None => println_stderr!("not found."),
        }
    } else if let Some(matches) = matches.subcommand_matches("decode") {
        let column: usize = matches.value_of("column").unwrap().parse::<usize>().unwrap();
        let feature = matches.value_of("feature_id").unwrap().parse::<u32>().unwrap();
        let feat = try!(workspace.decode(column, feature));
        println!("{}", feat);
    }
}
