use std::{collections::HashMap, iter::once, path::PathBuf};

use foray_py::{
    discover::{self, RawNodePackageInfo},
    py_node::PyNodeTemplate,
};

use crate::node_instance::ForayNodeTemplate;

#[derive(Debug)]
pub struct NodeTree<T> {
    pub name: String,
    pub children: HashMap<String, NodeTree<T>>,
    pub data: Option<T>,
}

impl<D> NodeTree<D> {
    pub fn new(root_name: String) -> Self {
        Self {
            name: root_name,
            children: HashMap::new(),
            data: None,
        }
    }
    pub fn insert(&mut self, key: Vec<&str>, data: D) {
        let mut current_node = self;

        for k in key.iter() {
            current_node = current_node
                .children
                .entry(k.to_string())
                .or_insert_with(|| NodeTree {
                    name: k.to_string(),
                    children: Default::default(),
                    data: None,
                });
        }
        current_node.data = Some(data);
    }
}

/// Get all python projects (node collections) from the current python environment
pub fn read_python_projects() -> Vec<crate::project::Project> {
    let raw = discover::get_foray_py_packages();
    raw.into_iter().map(python_project).collect()
}

#[derive(Debug)]
pub struct Project {
    pub absolute_path: PathBuf,
    pub node_tree: NodeTree<ForayNodeTemplate>,
}

pub fn python_project(package_info: RawNodePackageInfo) -> Project {
    let mut tree = NodeTree::new(package_info.package_name.clone());

    let entry_point_modules: Vec<&str> = package_info.entry_point.split(".").collect();

    package_info.node_py_paths.iter().for_each(|py_path| {
        // Format the python path for display in a tree selection,
        // Always start with the package name, and omit the entry_point path
        let display_path: Vec<&str> = once(package_info.package_name.as_str())
            .chain(
                py_path
                    .split(".")
                    .filter(|module| !entry_point_modules.iter().any(|em| em == module)),
            )
            .collect();

        tree.insert(
            display_path.clone(),
            ForayNodeTemplate::PyNode(PyNodeTemplate::new(py_path.to_string())),
        )
    });

    Project {
        absolute_path: package_info.abs_path,
        node_tree: tree,
    }
}
