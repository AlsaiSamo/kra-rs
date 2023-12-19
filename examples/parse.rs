use std::{env::args, path::PathBuf};

use kra::{
    layer::{Node, NodeType},
    KraFile,
};

//print all nodes, recursively
fn tree(node: &Node, depth: usize) {
    println!("{:>width$}{1}", " ", node, width = depth * 4);
    match node.props().node_type() {
        NodeType::GroupLayer(props) => {
            for i in props.layers() {
                tree(i, depth + 1)
            }
        }
        _ => {}
    }
}

fn main() {
    let path: PathBuf = args().skip(1).next().expect("Expected path to file").into();
    match KraFile::read(path) {
        Ok(file) => {
            for i in file.layers() {
                tree(i, 0)
            }
        }
        Err(what) => println!("{}", what),
    }
}
