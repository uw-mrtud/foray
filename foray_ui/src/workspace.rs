use crate::file_watch::file_watch_subscription;
use crate::interface::add_node::add_node_tree_panel;
use crate::interface::node_canvas::camera::Camera;
use crate::interface::node_canvas::node_canvas;
use crate::interface::{side_bar::side_bar, SEPERATOR};
use crate::math::{Point, Vector};
use crate::network::Network;
use crate::node_instance::visualiztion::Visualization;
use crate::node_instance::{ForayNodeInstance, ForayNodeTemplate, NodeStatus};
use crate::project::{read_python_projects, rust_project, Project};
use crate::python_env;
use crate::style::theme::AppTheme;
use crate::user_data::UserData;

use foray_data_model::node::{Dict, PortData};
use foray_graph::graph::{ForayNodeError, Graph, PortRef, IO};

use foray_py::err::PyNodeConfigError;
use foray_py::py_node::{PyConfig, PyNodeTemplate};
use iced::event::listen_with;
use iced::keyboard::key::Named;
use iced::keyboard::{Event::KeyPressed, Key, Modifiers};
use iced::widget::{container, horizontal_space, mouse_area, row, stack, text, vertical_rule};
use iced::Event::Keyboard;
use iced::Length::Fill;
use iced::{mouse, window, Element, Renderer, Subscription, Task, Theme};
use itertools::Itertools;
use log::{error, info, trace, warn};
use std::fs::{self, read_to_string};
use std::iter::once;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, PartialEq)]
pub enum Action {
    #[default]
    InitialLoad,
    Idle,
    // initial camera position, initial screen space cursor posititon
    DragPan(Vector, Point),
    DragNode(Vec<(u32, Vector, Point)>),
    CreatingInputWire(PortRef),
    CreatingOutputWire(PortRef),
    AddingNode,
    LoadingNetwork,
    SavingNetwork,
}

pub struct Workspace {
    pub workspace_dir: PathBuf,

    /// Node, Wire and Shape data that is executed, and saved to disk
    pub network: Network,

    /// List of all known Node types, including system and user nodes
    pub projects: Vec<Project>,

    pub main_window_id: Option<window::Id>,
    pub user_data: UserData,
    //// UI
    pub action: Action,
    pub cursor_position: Point,
}

#[derive(Clone, Debug)]
pub enum WorkspaceMessage {
    //// Workspace
    OnMove(Point),
    UpdateCamera(Camera),

    //// Port
    PortPress(PortRef),
    PortMouseUp(PortRef),
    PortDelete(PortRef),

    //// Node
    OnCanvasDown(Option<u32>),
    OnCanvasUp,
    OpenAddNodeUi,
    AddNode(ForayNodeTemplate),
    SelectNodeGroup(Vec<String>),

    UpdateNodeTemplate(u32, ForayNodeTemplate),
    UpdateNodeParameter(u32, String, PortData),
    DeleteSelectedNodes,

    QueueCompute(u32),
    ComputeComplete(u32, Result<Dict<String, PortData>, ForayNodeError>),
    ComputeAll,

    //// Application
    AnimationTick,
    New,
    StartLoadNetwork,
    EndLoadNetwork(Result<(PathBuf, Arc<String>), FileError>),
    StartSaveNetwork,
    EndSaveNetwork(Option<PathBuf>),
    ReloadNodes,
    ResizeWindow(window::Id, iced::Size),

    Cancel,

    //// History
    Undo,
    Redo,
}

impl Workspace {
    pub fn update(
        &mut self,
        message: WorkspaceMessage,
        modifiers: Modifiers,
    ) -> Task<WorkspaceMessage> {
        match message {
            WorkspaceMessage::OnMove(_) => {}
            _ => info!("---Message--- {message:?} {:?}", Instant::now()),
        }
        match message {
            WorkspaceMessage::Cancel => self.action = Action::Idle,
            WorkspaceMessage::OnMove(cursor_position) => {
                self.cursor_position = cursor_position;

                // Update node position if currently dragging
                match &self.action {
                    Action::DragNode(offsets) => {
                        offsets
                            .iter()
                            .for_each(|(id, initial_position, initial_cursor)| {
                                *self
                                    .network
                                    .shapes
                                    .shape_positions
                                    .get_mut(id)
                                    .expect("Shape index must exist") = (*initial_position
                                    + (cursor_position - *initial_cursor)
                                        * (1.0 / self.network.shapes.camera.zoom))
                                    .to_point();
                            });
                    }
                    Action::DragPan(initial_camera, initial_cursor) => {
                        self.network.shapes.camera.position = *initial_camera
                            - (cursor_position - *initial_cursor)
                                * (1.0 / self.network.shapes.camera.zoom)
                    }
                    _ => (),
                }
            }

            WorkspaceMessage::UpdateCamera(camera) => self.network.shapes.camera = camera,

            WorkspaceMessage::PortPress(port) => match &self.action.clone() {
                Action::Idle => match port.io {
                    IO::In => self.action = Action::CreatingInputWire(port),
                    IO::Out => self.action = Action::CreatingOutputWire(port),
                },
                Action::CreatingInputWire(input) => match port.io {
                    IO::In => {}
                    IO::Out => {
                        if port.node != input.node {
                            self.network.add_edge(input, &port);
                            self.action = Action::Idle;
                            return Task::done(WorkspaceMessage::QueueCompute(input.node));
                        }
                    }
                },
                Action::CreatingOutputWire(output) => match port.io {
                    IO::Out => {}
                    IO::In => {
                        if port.node != output.node {
                            self.network.add_edge(&port, output);
                            self.action = Action::Idle;
                            return Task::done(WorkspaceMessage::QueueCompute(port.node));
                        }
                    }
                },
                _ => {}
            },
            // If the user clicks, and drags a port, make the connection on mouse up
            WorkspaceMessage::PortMouseUp(port) => match &self.action.clone() {
                Action::Idle => {}
                Action::CreatingInputWire(input) => {
                    if port.node != input.node {
                        match port.io {
                            IO::In => {}
                            IO::Out => {
                                self.network.add_edge(input, &port);
                                self.action = Action::Idle;
                                return Task::done(WorkspaceMessage::QueueCompute(input.node));
                            }
                        }
                    }
                }
                Action::CreatingOutputWire(output) => {
                    if port.node != output.node {
                        match port.io {
                            IO::Out => {}
                            IO::In => {
                                self.network.add_edge(&port, output);
                                self.action = Action::Idle;
                                return Task::done(WorkspaceMessage::QueueCompute(port.node));
                            }
                        }
                    }
                }
                _ => {}
            },
            WorkspaceMessage::PortDelete(port) => {
                self.network.remove_edge(port);
            }

            //// Node
            WorkspaceMessage::OnCanvasDown(clicked_id) => {
                //TODO: break this logic down into pure functions
                //// Clicked on a node
                if let Some(nx) = clicked_id {
                    self.action = self
                        .network
                        .select_node(nx, modifiers, self.cursor_position);
                    return Task::done(WorkspaceMessage::QueueCompute(nx));
                } else
                //// Clicked on the canvas background
                {
                    //// Clear selected shapes
                    self.network.selected_shapes = Default::default();

                    //// Start Pan
                    self.action =
                        Action::DragPan(self.network.shapes.camera.position, self.cursor_position);
                }
            }
            WorkspaceMessage::OnCanvasUp => {
                // TODO: push undo stack if shape has moved
                match self.action {
                    Action::DragNode(..) => self.action = Action::Idle,
                    Action::DragPan(_, _) => self.action = Action::Idle,
                    _ => (),
                }
            }

            WorkspaceMessage::UpdateNodeTemplate(id, new_template) => {
                //TODO: move into Network
                let old_template = &self.network.graph.get_node(id).template;
                if *old_template != new_template {
                    self.network.stash_state();
                    // Now we can aquire mutable reference
                    let old_template = &mut self.network.graph.get_mut_node(id).template;
                    *old_template = new_template;
                    return Task::done(WorkspaceMessage::QueueCompute(id));
                };
            }
            WorkspaceMessage::UpdateNodeParameter(id, name, updated_widget) => {
                //TODO: move into Network
                self.network.stash_state();
                let old_template = &mut self.network.graph.get_mut_node(id);
                old_template.parameters_values.insert(name, updated_widget);
                return Task::done(WorkspaceMessage::QueueCompute(id));
            }
            WorkspaceMessage::OpenAddNodeUi => self.action = Action::AddingNode,
            WorkspaceMessage::SelectNodeGroup(selected_tree_path) => match &self.action {
                Action::AddingNode => {
                    let current_path = self.user_data.get_new_node_path();
                    if current_path.starts_with(&selected_tree_path) {
                        self.user_data.set_new_node_path(
                            &selected_tree_path[0..selected_tree_path.len().saturating_sub(1)],
                        );
                    } else {
                        self.user_data.set_new_node_path(&selected_tree_path);
                    }
                }
                _ => error!(
                    "should not be able to select a nope group while Add Node UI is not active"
                ),
            },
            WorkspaceMessage::AddNode(template) => {
                //TODO: move into Network
                self.network.stash_state();
                let id = self.network.graph.node(template.into());
                self.network.selected_shapes = [id].into();

                let initial_position = self
                    .network
                    .shapes
                    .camera
                    .cursor_to_world(self.cursor_position);
                self.network
                    .shapes
                    .shape_positions
                    .insert_before(0, id, initial_position);
                self.action = Action::DragNode(vec![(
                    id,
                    initial_position.to_vector(),
                    self.cursor_position,
                )])
            }
            WorkspaceMessage::DeleteSelectedNodes => {
                //TODO: move into Network
                if !self.network.selected_shapes.is_empty() {
                    self.network.stash_state();
                    self.network.selected_shapes.iter().for_each(|id| {
                        self.network.graph.delete_node(*id);
                        self.network.shapes.shape_positions.swap_remove(id);
                    });
                    self.network.selected_shapes = [].into();
                    //PERF: ideally, we should only execute affected nodes
                    return Task::done(WorkspaceMessage::ComputeAll);
                }
            }

            WorkspaceMessage::AnimationTick => {}
            WorkspaceMessage::New => {
                //TODO: move into Network
                if self.network.unsaved_changes {
                    let result = rfd::MessageDialog::new()
                        .set_title("Unsaved Changes")
                        .set_level(rfd::MessageLevel::Warning)
                        .set_description("Network has unsaved changes, continue without saving?")
                        .set_buttons(rfd::MessageButtons::OkCancel)
                        .show();
                    match result {
                        rfd::MessageDialogResult::Ok => {}
                        rfd::MessageDialogResult::Cancel => return Task::none(),
                        _ => {}
                    }
                }
                self.network = Network::default();
                self.reload_nodes();
            }
            WorkspaceMessage::StartLoadNetwork => {
                if self.network.unsaved_changes {
                    // TODO: make this dialog async similar to `load_network_dialog`
                    let result = rfd::MessageDialog::new()
                        .set_title("Unsaved Changes")
                        .set_level(rfd::MessageLevel::Warning)
                        .set_description("Network has unsaved changes, continue without saving?")
                        .set_buttons(rfd::MessageButtons::OkCancel)
                        .show();
                    match result {
                        rfd::MessageDialogResult::Ok => {}
                        rfd::MessageDialogResult::Cancel => return Task::none(),
                        _ => {}
                    }
                }
                self.action = Action::LoadingNetwork;
                return Task::perform(
                    load_network_dialog(self.get_and_create_network_default_dir()),
                    WorkspaceMessage::EndLoadNetwork,
                );
            }

            WorkspaceMessage::EndLoadNetwork(result) => {
                self.action = Action::Idle;
                match result {
                    Ok((file, network)) => match ron::from_str(&network) {
                        Ok(network) => {
                            self.network = network;

                            self.network.file = Some(file.clone());
                            self.user_data.set_recent_network_file(Some(file));
                            self.reload_nodes();
                            return Task::done(WorkspaceMessage::ComputeAll);
                        }
                        Err(err) => {
                            error!("Error parsing network: {err}")
                        }
                    },
                    Err(FileError::DialogClosed) => {}
                    Err(FileError::FileReadFailed(err)) => {
                        error!("Error reading network file: {err}")
                    }
                }
            }
            WorkspaceMessage::StartSaveNetwork => match self.network.file.clone() {
                Some(file) => return Task::done(WorkspaceMessage::EndSaveNetwork(Some(file))),
                None => {
                    self.action = Action::SavingNetwork;
                    return Task::perform(
                        save_network_dialog(self.get_and_create_network_default_dir()),
                        WorkspaceMessage::EndSaveNetwork,
                    );
                }
            },
            WorkspaceMessage::EndSaveNetwork(file) => {
                self.action = Action::Idle;
                if let Some(file) = file {
                    std::fs::write(
                        &file,
                        ron::ser::to_string_pretty(
                            &self.network,
                            ron::ser::PrettyConfig::default().compact_arrays(true),
                        )
                        .unwrap(),
                    )
                    .expect("Could not save to file");
                    info!("saved network {file:?}");
                    self.network.file = Some(file.clone());
                    self.network.unsaved_changes = false;
                    self.user_data.set_recent_network_file(Some(file));
                } else {
                    info!("File not picked")
                }
            }
            WorkspaceMessage::ReloadNodes => {
                self.reload_nodes();
                return Task::done(WorkspaceMessage::ComputeAll);
            }
            WorkspaceMessage::ResizeWindow(id, size) => {
                if Some(id) == self.main_window_id {
                    self.network.shapes.camera.bounds_size = size;
                } else {
                    // panic!("Multiple windows not yet supported");
                }
            }

            //// History
            WorkspaceMessage::Undo => {
                //TODO: move into Network
                if let Some(prev) = self.network.undo_stack.pop() {
                    self.network.redo_stack.push((
                        self.network.graph.clone(),
                        self.network.shapes.shape_positions.clone(),
                    ));
                    self.network.graph = prev.0;
                    self.network.shapes.shape_positions = prev.1;
                    return Task::done(WorkspaceMessage::ComputeAll);
                }
            }
            WorkspaceMessage::Redo => {
                //TODO: move into Network
                if let Some(next) = self.network.redo_stack.pop() {
                    self.network.undo_stack.push((
                        self.network.graph.clone(),
                        self.network.shapes.shape_positions.clone(),
                    ));
                    self.network.graph = next.0;
                    self.network.shapes.shape_positions = next.1;
                    return Task::done(WorkspaceMessage::ComputeAll);
                }
            }
            WorkspaceMessage::ComputeAll => {
                //TODO: move into Network
                let nodes = self.network.graph.get_roots();
                trace!("Queuing root nodes: {nodes:?}");
                return Task::batch(
                    nodes
                        .into_iter()
                        .map(|nx| Task::done(WorkspaceMessage::QueueCompute(nx))),
                );
            }
            WorkspaceMessage::QueueCompute(nx) => {
                //TODO: move into Network
                //// Modify node status
                {
                    let node = self.network.graph.get_mut_node(nx);

                    // Check if in an error state
                    if let NodeStatus::Error(e) = &node.status {
                        if e.iter().any(|e| match e {
                            ForayNodeError::PyNodeConifgError(PyNodeConfigError::Runtime(_)) => {
                                false
                            }
                            _ => true,
                        }) {
                            warn!(
                                "Did not compute node  {e:?} compute: {:?} #{nx}",
                                node.template,
                            );
                            return Task::none();
                        }
                    }
                    // Re-queue
                    if let NodeStatus::Running { .. } = node.status {
                        // trace!("Re-queue, {} #{nx}", node.template);
                        self.network.queued_nodes.insert(nx);
                        return Task::none();
                    };

                    node.status = NodeStatus::Running {
                        start: Instant::now(),
                    };
                    trace!("Beginning compute: {:?} #{nx}", node.template,);
                }

                //// Queue compute
                let node = self.network.graph.get_node(nx);
                return Task::perform(
                    Graph::async_compute(nx, node.clone(), self.network.graph.get_input_data(&nx)),
                    move |(nx, res)| WorkspaceMessage::ComputeComplete(nx, res),
                );
            }
            WorkspaceMessage::ComputeComplete(nx, result) => {
                //TODO: move into Network
                let node = self.network.graph.get_node(nx);
                match result {
                    Ok(output) => {
                        // Assert that status is what is expected
                        let run_time = match &node.status {
                            NodeStatus::Idle => panic!("Node should not be idle here!"),
                            NodeStatus::Running { start: start_inst } => {
                                Instant::now() - *start_inst
                            }
                            NodeStatus::Error(py_node_error) => panic!(
                                "Node should not be in an error state here!{py_node_error:?}"
                            ),
                        };

                        trace!(
                            "Compute complete: {:?} #{nx}, {run_time:.1?}",
                            node.template,
                        );

                        //// Update node
                        self.network.graph.set_node_data(
                            nx,
                            ForayNodeInstance {
                                status: NodeStatus::Idle,
                                parameters_values: node.parameters_values.clone(),
                                visualization: Visualization::new(node, &output),
                                // run_time: Some(run_time),
                                // We *don't* update template here for some nodes
                                // because that causes stuttery behaviour for
                                // fast update scenarios like the slider of the 'constant'
                                // node. alternatively, canceling in progress compute tasks
                                // might address this, and may be necessary in the future.
                                // similar to TODO: below
                                template: match node.template {
                                    ForayNodeTemplate::PyNode(_) => {
                                        self.network.graph.get_node(nx).template.clone()
                                    }
                                    _ => node.template.clone(),
                                },
                            },
                        );
                        //// Update wire
                        self.network.graph.update_wire_data(nx, output);

                        //// Queue children for compute
                        let to_queue: Vec<_> = self
                            .network
                            .graph
                            .outgoing_edges(&nx)
                            .into_iter()
                            .map(|port_ref| port_ref.node)
                            .unique() // Don't queue a child multiple times
                            // TODO: instead of requeing after compute is done,
                            // potentially abort the running compute task, and restart
                            // immediately when new input data is received
                            .chain(
                                once(self.network.queued_nodes.remove(&nx).then_some(nx)).flatten(),
                            ) // Re-execute node if it got queued up in the meantime
                            .collect();
                        trace!("Queuing children for compute {to_queue:?}");
                        return Task::batch(
                            to_queue
                                .into_iter()
                                .map(|node| Task::done(WorkspaceMessage::QueueCompute(node))),
                        );
                    }
                    Err(node_error) => {
                        //// Update Node
                        let node = self.network.graph.get_mut_node(nx);
                        warn!("Compute failed {node:?},{node_error:?}");

                        node.status = NodeStatus::Error(vec![node_error]);
                        node.visualization.image_handle = None;

                        //// Update wire
                        self.network.graph.clear_outputs(nx);

                        return Task::none();
                    }
                };
            }
        };
        Task::none()
    }

    /// App View
    pub fn view<'a>(
        &'a self,
        app_theme: &'a AppTheme,
    ) -> Element<'a, WorkspaceMessage, Theme, Renderer> {
        let network_panel: Element<WorkspaceMessage, Theme, Renderer> = match self.action {
            Action::LoadingNetwork => container(text("loading...")).center(Fill).into(),
            Action::SavingNetwork => container(text("saving...")).center(Fill).into(),
            _ => node_canvas::node_canvas(
                &self.network.shapes.shape_positions,
                self.network.shapes.camera,
                self,
                app_theme,
            )
            .into(),
        };

        // Stack side bar over panel, to avoid canvas drawing images outside bounds (iced bug)
        let content = stack![
            network_panel,
            row![
                iced::widget::opaque(side_bar(self)),
                vertical_rule(SEPERATOR),
                horizontal_space()
            ],
        ];
        //match self.show_palette_ui {
        //    true => column![horizontal_rule(SEPERATOR), self.app_theme.view()],
        //    false => column![],
        //}

        let output: Element<WorkspaceMessage, Theme, Renderer> = match &self.action {
            Action::AddingNode => stack![
                content,
                // Barrier to stop interaction
                mouse_area(
                    container(text(""))
                        .center(Fill)
                        .style(container::transparent)
                )
                // Stop any mouseover cursor interactions from showing,
                .interaction(mouse::Interaction::Idle)
                .on_press(WorkspaceMessage::Cancel),
                //// Add node modal
                container(
                    mouse_area(add_node_tree_panel(
                        &self.projects,
                        self.user_data.get_new_node_path()
                    ))
                    .interaction(mouse::Interaction::Idle) //.on_press(Message::NOP)
                )
                .center(Fill)
            ]
            .into(),
            _ => content.into(),
        };

        // Potentially add a specific mouse cursor
        let output = match self.action {
            Action::DragNode(_) => mouse_area(output)
                .interaction(mouse::Interaction::Grabbing)
                .into(),
            _ => output,
        };

        //if self.debug {
        //    output.explain(iced::Color::from_rgba(0.7, 0.7, 0.8, 0.2))
        //} else {
        output
        //}
    }
}

#[derive(Debug)]
pub enum WorkspaceError {
    NoVenv,
}

impl Workspace {
    pub fn is_valid_workspace(workspace_dir: &PathBuf) -> bool {
        workspace_dir.join(".venv").is_dir()
    }

    pub fn new(
        workspace_dir: PathBuf,
        network_path: Option<PathBuf>,
        main_window_id: Option<window::Id>,
    ) -> Result<Self, WorkspaceError> {
        if !Self::is_valid_workspace(&workspace_dir) {
            return Err(WorkspaceError::NoVenv);
        };

        let network = match network_path {
            Some(np) => match Network::load_network(&np) {
                Ok(n) => n,
                Err(err) => {
                    warn!("{err:?}");
                    //user_data.set_recent_network_file(None); // Recent network failed to load,
                    // remove it from user data
                    Network::default()
                }
            },
            //// If no network provided, get the most recent network
            None => Network::default(),
        };
        let venv_dir = workspace_dir.join(".venv");
        python_env::setup_python(venv_dir);
        let projects = read_python_projects();
        trace!(
            "Configured Python Projects: {:?}",
            projects
                .iter()
                .map(|p| p.absolute_path.clone())
                .collect::<Vec<_>>()
        );

        let mut workspace = Self {
            workspace_dir,
            network,
            projects,
            user_data: UserData::read_user_data(),
            main_window_id,
            action: Default::default(),
            cursor_position: Default::default(),
        };
        workspace.reload_nodes();
        Ok(workspace)
    }

    pub fn get_and_create_network_default_dir(&self) -> PathBuf {
        let network_dir = self.workspace_dir.join("networks");

        // Create the network directory if it doesn't exist
        let _ = fs::create_dir_all(&network_dir);
        network_dir
    }

    /// Read node definitions from disk, and copies node configuration (parameters and port connections) forward.
    /// *Does not trigger the compute function of any nodes.*
    /// TODO: This has been edited several times as the data model has changed. This may be
    /// able to be cleaned up significantly
    fn reload_nodes(&mut self) {
        // Update any existing nodes in the graph that could change based on file changes
        self.network.graph.nodes_ref().iter().for_each(|nx| {
            let node = self.network.graph.get_node(*nx).clone();
            if let ForayNodeTemplate::PyNode(old_py_node) = node.template {
                let PyNodeTemplate {
                    name: _node_name,
                    py_path,
                    config: old_config,
                } = old_py_node;

                let PyConfig {
                    inputs: old_inputs,
                    outputs: old_outputs,
                    parameters: _old_parameters,
                } = old_config.unwrap_or_default();

                //// Read new node from disk
                let new_py_node_template = PyNodeTemplate::new(py_path);

                //// Update Ports, and Graph Edges
                {
                    let new_in_ports = new_py_node_template.inputs().unwrap_or_default();
                    let new_out_ports = new_py_node_template.outputs().unwrap_or_default();

                    // Get old version's ports
                    let old_in_ports = old_inputs.unwrap_or_default();
                    let old_out_ports = old_outputs.unwrap_or_default();

                    // Find any nodes that previously existed, but now doesn't
                    let invalid_in = old_in_ports
                        .into_iter()
                        .filter(|(old_name, old_type)| new_in_ports.get(old_name) != Some(old_type))
                        .map(|(old_name, _)| PortRef {
                            node: *nx,
                            name: old_name,
                            io: IO::In,
                        });
                    let invalid_out = old_out_ports
                        .into_iter()
                        .filter(|(old_name, old_type)| {
                            new_out_ports.get(old_name) != Some(old_type)
                        })
                        .map(|(old_name, _)| PortRef {
                            node: *nx,
                            name: old_name,
                            io: IO::Out,
                        });

                    // Remove invalid edges from Graph
                    invalid_in.chain(invalid_out).for_each(|p| {
                        warn!(
                            "Removing port {:?} from node {:?}",
                            p.name, new_py_node_template.name
                        );
                        self.network.graph.remove_edge(&p);
                    });
                }

                let mut new_node_instance: ForayNodeInstance =
                    ForayNodeTemplate::PyNode(new_py_node_template).into();

                // Merge parameters
                node.parameters_values.into_iter().for_each(|(key, value)| {
                    new_node_instance
                        .parameters_values
                        .entry(key)
                        .and_modify(|v| *v = value);
                });
                // Update Graph Node
                self.network.graph.set_node_data(*nx, new_node_instance);
            }
        });
        // Update list of available nodes
        let mut projects = read_python_projects();
        projects.push(rust_project());
        self.projects = projects;
    }

    pub fn subscriptions(&self) -> Subscription<WorkspaceMessage> {
        Subscription::batch(
            self.projects
                .iter()
                .filter(|p| !p.absolute_path.to_string_lossy().is_empty())
                .enumerate()
                .map(|(id, p)| {
                    file_watch_subscription(
                        id,
                        p.absolute_path.clone(),
                        WorkspaceMessage::ReloadNodes,
                    )
                })
                .chain([
                    window::resize_events()
                        .map(|(id, size)| WorkspaceMessage::ResizeWindow(id, size)),
                    listen_with(|event, _status, _id| match event {
                        Keyboard(KeyPressed { key, modifiers, .. }) => match key {
                            Key::Named(Named::Delete) => {
                                Some(WorkspaceMessage::DeleteSelectedNodes)
                            }
                            Key::Named(Named::Escape) => Some(WorkspaceMessage::Cancel),
                            Key::Character(smol_str) => {
                                if modifiers.control() && smol_str == "a" {
                                    Some(WorkspaceMessage::OpenAddNodeUi)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    }),
                    // Refresh for animation while nodes are actively running
                    if self.network.any_nodes_running() {
                        iced::time::every(Duration::from_micros(1_000_000 / 16))
                            .map(|_| WorkspaceMessage::AnimationTick)
                    } else {
                        Subscription::none()
                    },
                ]),
        )
    }
}

#[derive(Clone, Debug)]
pub enum FileError {
    DialogClosed,
    FileReadFailed(String),
}

/// Open a file dialog to pick a network file
pub async fn load_network_dialog(
    start_directory: PathBuf,
) -> Result<(PathBuf, Arc<String>), FileError> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a network file...")
        .set_directory(start_directory)
        .add_filter("network", &["network"])
        .pick_file()
        .await
        .ok_or(FileError::DialogClosed)?;

    match read_to_string(picked_file.path()) {
        Ok(network_string) => Ok((picked_file.path().to_path_buf(), Arc::new(network_string))),
        Err(err) => Err(FileError::FileReadFailed(err.to_string()))?,
    }
}

/// Open a file dialog to save a network file
async fn save_network_dialog(default_dir: PathBuf) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(default_dir)
        .add_filter("network", &["network"])
        .save_file()
        .await
        .map(|fh| fh.into())
}
