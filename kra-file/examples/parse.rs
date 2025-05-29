use std::{env::args, path::PathBuf};

use kra_file::{layer::Node, parse::ParsingConfiguration, KraFile};

//print all nodes, recursively
fn tree(node: &Node, depth: usize) {
    println!(
        "{:>width$}{1}",
        " ",
        node.uuid().unwrap(),
        width = depth * 4
    );
    if node.is_group_layer() {
        for i in node.as_group_layer().unwrap().layers() {
            tree(i, depth + 1)
        }
    }
}

fn main() {
    let path: PathBuf = args().nth(1).expect("Expected path to file").into();
    match KraFile::read(path, ParsingConfiguration::default()) {
        Ok(file) => {
            for i in file.layers() {
                tree(i, 0)
            }
        }
        Err(what) => println!("{}", what),
    }
}
