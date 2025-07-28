use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use foray_graph::{node_instance::ForayNodeTemplate, rust_node::RustNodeTemplate};
use relative_path::PathExt;
use strum::IntoEnumIterator;

use foray_py::py_node::PyNodeTemplate;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum NodeTree<T> {
    Group(String, Vec<NodeTree<T>>),
    Leaf(T),
}

impl<D> NodeTree<D> {
    pub fn new() -> Self {
        NodeTree::Group("".into(), vec![])
    }
}

impl<D> NodeTree<D>
where
    D: Clone + PartialOrd,
{
    pub fn sort(&self) -> Self {
        match self {
            NodeTree::Group(v, node_trees) => {
                let mut sorted = node_trees.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

                NodeTree::Group(v.clone(), sorted)
            }
            NodeTree::Leaf(v) => NodeTree::Leaf(v.clone()),
        }
    }
}

impl<D> Default for NodeTree<D> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Project {
    pub absolute_path: PathBuf,
    pub node_tree: Vec<NodeTree<ForayNodeTemplate>>,
}

pub fn python_project(absolute_path: &Path) -> Project {
    let node_tree = python_tree(absolute_path.to_path_buf(), |dir| {
        (dir.path().extension().map(|os| os.to_str()) == Some(Some("py")) || dir.path().is_dir())
            && not_hidden(dir)
    });
    Project {
        absolute_path: absolute_path.to_path_buf(),
        node_tree,
    }
}
pub fn rust_project() -> Project {
    let rust_nodes = RustNodeTemplate::iter()
        .map(|n| NodeTree::Leaf(ForayNodeTemplate::RustNode(n)))
        .collect();
    let node_tree = NodeTree::Group("rust".to_string(), rust_nodes);
    Project {
        absolute_path: Default::default(),
        node_tree: vec![node_tree],
    }
}

pub fn not_hidden(entry: &DirEntry) -> bool {
    !entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn python_tree<F>(project_path: PathBuf, filter_entries: F) -> Vec<NodeTree<ForayNodeTemplate>>
where
    F: Fn(&DirEntry) -> bool + Copy,
{
    fn inner<F>(
        project_path: PathBuf,
        path: PathBuf,
        filter_entries: F,
    ) -> Vec<NodeTree<ForayNodeTemplate>>
    where
        F: Fn(&DirEntry) -> bool + Copy,
    {
        let mut result = fs::read_dir(&path)
            .unwrap_or_else(|e| {
                panic!("Directory should exist:{e:?}, and user should have permission")
            })
            .filter_map(|e| e.ok())
            .filter(filter_entries)
            .fold(vec![], move |mut root, entry| {
                let dir = entry.metadata().unwrap();
                if dir.is_dir() {
                    let entries = inner(project_path.clone(), entry.path(), filter_entries);
                    if !entries.is_empty() {
                        root.push(NodeTree::Group(
                            entry
                                .path()
                                .file_stem()
                                .map(|os| os.to_str().unwrap_or(""))
                                .unwrap_or("COULD NOT PARSE NODE NAME")
                                .to_string(),
                            entries,
                        ))
                    }
                } else {
                    root.push(NodeTree::Leaf(ForayNodeTemplate::PyNode(
                        PyNodeTemplate::new(
                            entry.path(),
                            entry
                                .path()
                                .relative_to(project_path.clone())
                                .expect("node is subpath of project dir"),
                        ),
                    )));
                }
                root
            });
        result.sort_by(|a, b| a.partial_cmp(b).expect("nodes should be comparable"));
        result
    }

    inner(project_path.clone(), project_path, filter_entries)
}
