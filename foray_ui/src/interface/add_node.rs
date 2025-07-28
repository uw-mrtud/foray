use crate::node_instance::ForayNodeTemplate;
use crate::project::{NodeTree, Project};
use crate::style::container::rounded_box;
use crate::{app::Message, style};
use iced::padding::left;
use iced::*;
use itertools::Itertools;
use widget::{column, *};

const ROW_HEIGHT: f32 = 25.0;

/// Given a list of node trees,
/// Render a nested list, that is expanded to `selected_tree_path`
pub fn add_node_tree_panel<'b>(
    projects: &[Project],
    selected_tree_path: &[String],
) -> Element<'b, Message> {
    let node_list = column(
        projects
            .iter()
            .flat_map(|prj| &prj.node_tree)
            .sorted_by(|a, b| a.partial_cmp(b).unwrap())
            .map(|tree| node_tree(tree, &[], selected_tree_path)),
    );

    container(column![
        // Header
        container(text("Add Node").size(16.))
            .center_x(Fill)
            .padding(5.),
        horizontal_rule(3.0),
        // Contents
        container(scrollable(node_list).spacing(2.)).padding(2.0)
    ])
    .style(rounded_box)
    .width(300.)
    .height(Shrink)
    .into()
}

/// Recursively build a nested list
pub fn node_tree<'b>(
    node: &NodeTree<ForayNodeTemplate>,
    tree_path: &[String],
    selected_tree_path: &[String],
) -> Element<'b, Message> {
    let tree_row = |name, message| {
        row![
            // Vertical bar seperators
            row((0..tree_path.len()).map(|_| row![vertical_rule(10.0)].height(ROW_HEIGHT).into())),
            button(container(name).padding(left(4.0)))
                .padding(0.)
                .on_press(message)
                .width(Fill)
                .style(style::button::list)
        ]
    };
    match node {
        // Base case, File Row
        NodeTree::Leaf(node) => tree_row(text(node.name()), Message::AddNode(node.clone()))
            .align_y(Center)
            .into(),
        // Folder Row
        NodeTree::Group(name, node_trees) => {
            let next_path = &[tree_path, &[name.to_owned()]].concat();
            let folder_row = tree_row(
                text(name.to_string()).style(text::primary),
                Message::SelectNodeGroup(next_path.to_vec()),
            );
            // Is this folder selected?
            match selected_tree_path.starts_with(next_path) {
                true => column![
                    folder_row,
                    // Recurse
                    column(
                        node_trees
                            .iter()
                            .map(|n| node_tree(n, next_path, selected_tree_path)),
                    )
                ]
                .into(),
                false => folder_row.into(),
            }
        }
    }
}
