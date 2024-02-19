use std::{env::args, path::PathBuf};

use kra::{
    layer::{Node, NodeType},
    KraFile,
};

//print all nodes, recursively
fn tree(node: &Node, depth: usize) {
    println!("{:>width$}{1}", " ", node, width = depth * 4);
    if let NodeType::GroupLayer(props) = node.props().node_type() {
        for i in props.layers() {
            tree(i, depth + 1)
        }
    }
}

fn main() {
    let path: PathBuf = args().nth(1).expect("Expected path to file").into();
    match KraFile::read(path) {
        Ok(file) => {
            for i in file.layers() {
                tree(i, 0)
            }
        }
        Err(what) => println!("{}", what),
    }
}
