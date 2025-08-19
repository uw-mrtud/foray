use crate::config::Config;
use crate::file_watch::file_watch_subscription;
use crate::interface::add_node::add_node_tree_panel;
use crate::interface::theme_config::{AppThemeMessage, GuiColorMessage};
use crate::interface::{side_bar::side_bar, SEPERATOR};
use crate::math::{Point, Vector};
use crate::network::Network;
use crate::node_instance::visualiztion::Visualization;
use crate::node_instance::{ForayNodeInstance, ForayNodeTemplate, NodeStatus};
use crate::project::{read_python_projects, Project};
use crate::python_env;
use crate::rust_nodes::RustNodeTemplate;
use crate::style::theme::AppTheme;
use crate::user_data::UserData;
use crate::widget::shapes::ShapeId;
use crate::widget::workspace::workspace;

use foray_data_model::node::{Dict, PortData};
use foray_graph::graph::{ForayNodeError, Graph, PortRef, IO};
use foray_py::py_node::{PyConfig, PyNodeTemplate};

use iced::advanced::graphics::core::Element;
use iced::event::listen_with;
use iced::keyboard::key::Named;
use iced::keyboard::Event::KeyPressed;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{column, *};
use iced::Event::Keyboard;
use iced::Length::Fill;
use iced::{mouse, window, Subscription, Task};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use rfd::FileDialog;
use std::fs::read_to_string;
use std::iter::once;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Default, Clone, PartialEq)]
pub enum Action {
    #[default]
    InitialLoad,
    Idle,
    DragPan(Vector),
    DragNode(Vec<(u32, Vector)>),
    CreatingInputWire(PortRef, Option<PortRef>),
    CreatingOutputWire(PortRef, Option<PortRef>),
    AddingNode,
}

pub struct App {
    /// Node, Wire and Shape data that is executed, and saved to disk
    pub network: Network,

    /// Persitant user data
    pub user_data: UserData,
    /// List of all known Node types, including system and user nodes
    pub projects: Vec<Project>,
    pub app_theme: AppTheme,
    pub config: Config,

    /// current editor action
    pub action: Action,
    pub cursor_position: Point,
    /// Currently held keyboard modifiers, used for shortcuts
    pub modifiers: Modifiers,

    pub debug: bool,
    pub show_palette_ui: bool,
}
impl App {
    pub fn new(cli_network_path: Option<PathBuf>) -> Self {
        let mut user_data = UserData::read_user_data();

        let network_path = match cli_network_path {
            Some(np) => Some(np),
            //// If no network provided, get the most recent network
            None => user_data.get_recent_network_file(),
        };
        //// Venv selection precedence
        //// 1. if network specified, venv = "network_path/../.venv"
        ////    - if no venv, panic
        ////    - if venv, create a new default network
        //// 3. If no recent network use check current directory for venv
        ////    - if no venv, panic
        ////    - if venv, create a new default network

        let venv_dir = match &network_path {
            Some(np) => np
                .parent()
                .expect("Network should be a file")
                .join("../.venv"),
            None => panic!("No venv found relative to network path"),
        };

        let config = Config::read_config();
        python_env::setup_python(venv_dir);
        let projects = read_python_projects();
        trace!(
            "Configured Python Projects: {:?}",
            projects
                .iter()
                .map(|p| p.absolute_path.clone())
                .collect::<Vec<_>>()
        );

        let app_theme = Config::load_theme();

        let network = match network_path {
            Some(np) => match Network::load_network(&np) {
                Ok(n) => n,
                Err(err) => {
                    warn!("{err:?}");
                    user_data.set_recent_network_file(None); // Recent network failed to load,
                                                             // remove it from user data
                    Network::default()
                }
            },
            //// If no network provided, get the most recent network
            None => Network::default(),
        };

        App {
            network,
            config,

            debug: false,
            show_palette_ui: false,
            cursor_position: Default::default(),
            action: Default::default(),
            app_theme,
            modifiers: Default::default(),
            projects,
            user_data,
        }
    }
}

#[derive(Clone, derive_more::Debug)]
pub enum Message {
    //// Workspace
    OnMove(Point),
    ScrollPan(Vector),

    //// Port
    PortStartHover(PortRef),
    PortEndHover(PortRef),
    PortPress(PortRef),
    PortRelease,
    PortDelete(PortRef),

    //// Node
    OnCanvasDown(Option<ShapeId>),
    OnCanvasUp,
    OpenAddNodeUi,
    AddNode(ForayNodeTemplate),
    SelectNodeGroup(Vec<String>),

    UpdateNodeTemplate(u32, ForayNodeTemplate),
    UpdateNodeParameter(u32, String, PortData),
    DeleteSelectedNodes,

    QueueCompute(u32),
    ComputeComplete(
        u32,
        #[debug(skip)] Result<Dict<String, PortData>, ForayNodeError>,
    ),
    ComputeAll,

    //// Application
    AnimationTick,
    ThemeValueChange(AppThemeMessage, GuiColorMessage),
    ToggleDebug,
    TogglePaletteUI,
    New,
    Load,
    Save,
    ReloadNodes,
    WindowOpen,
    ModifiersChanged(Modifiers),

    //// Focus
    FocusNext,
    FocusPrevious,
    Cancel,

    //// History
    Undo,
    Redo,
    //// Misc
    /// Hacky way to have a message that does nothing
    NOP,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OnMove(_) => {}
            _ => trace!("---Message--- {message:?} {:?}", Instant::now()),
        }
        match message {
            Message::NOP => {}
            Message::Cancel => self.action = Action::Idle,
            Message::OnMove(cursor_position) => {
                self.cursor_position = cursor_position;

                // Update node position if currently dragging
                match &self.action {
                    Action::DragNode(offsets) => {
                        offsets.iter().for_each(|(id, offset)| {
                            *self
                                .network
                                .shapes
                                .shape_positions
                                .get_mut(id)
                                .expect("Shape index must exist") =
                                (cursor_position + self.network.shapes.camera.position) + *offset
                        });
                    }
                    Action::DragPan(offset) => {
                        self.network.shapes.camera.position =
                            -cursor_position.to_vector() + *offset;
                    }
                    _ => (),
                }
            }

            Message::ScrollPan(delta) => {
                self.network.shapes.camera.position.x -= delta.x * 2.;
                self.network.shapes.camera.position.y -= delta.y * 2.;
            }

            //// Port
            Message::PortStartHover(hover_port) => match &self.action {
                Action::CreatingInputWire(input, _) => {
                    if *input != hover_port {
                        self.action = Action::CreatingInputWire(input.clone(), Some(hover_port))
                    }
                }
                Action::CreatingOutputWire(output, _) => {
                    if *output != hover_port {
                        self.action = Action::CreatingOutputWire(output.clone(), Some(hover_port))
                    }
                }
                _ => {}
            },
            Message::PortEndHover(_port) => match &self.action {
                Action::CreatingInputWire(input, _) => {
                    self.action = Action::CreatingInputWire(input.clone(), None)
                }
                Action::CreatingOutputWire(output, _) => {
                    self.action = Action::CreatingOutputWire(output.clone(), None)
                }
                _ => {}
            },
            Message::PortPress(port) => match port.io {
                IO::In => self.action = Action::CreatingInputWire(port, None),
                IO::Out => self.action = Action::CreatingOutputWire(port, None),
            },
            Message::PortRelease => {
                let task = match &self.action.clone() {
                    Action::CreatingInputWire(input, Some(output))
                    | Action::CreatingOutputWire(output, Some(input)) => {
                        self.network.add_edge(input, output);
                        Task::done(Message::QueueCompute(output.node))
                    }
                    _ => Task::none(),
                };
                self.action = Action::Idle;
                return task;
            }
            Message::PortDelete(port) => {
                self.network.remove_edge(port);
            }

            //// Node
            Message::OnCanvasDown(clicked_id) => {
                //TODO: break this logic down into pure functions
                //// Clicked on a node
                if let Some(nx) = clicked_id {
                    self.action =
                        self.network
                            .select_node(nx, self.modifiers, self.cursor_position);
                    return Task::done(Message::QueueCompute(nx));
                } else
                //// Clicked on the canvas background
                {
                    //// Clear selected shapes
                    self.network.selected_shapes = Default::default();

                    //// Start Pan
                    self.action = Action::DragPan(
                        self.network.shapes.camera.position + self.cursor_position.to_vector(),
                    );
                }
            }
            Message::OnCanvasUp => {
                // TODO: push undo stack if shape has moved
                match self.action {
                    Action::DragNode(..) => self.action = Action::Idle,
                    Action::DragPan(_) => self.action = Action::Idle,
                    _ => (),
                }
            }

            Message::UpdateNodeTemplate(id, new_template) => {
                //TODO: move into Network
                let old_template = &self.network.graph.get_node(id).template;
                if *old_template != new_template {
                    self.network.stash_state();
                    // Now we can aquire mutable reference
                    let old_template = &mut self.network.graph.get_mut_node(id).template;
                    *old_template = new_template;
                    return Task::done(Message::QueueCompute(id));
                };
            }
            Message::UpdateNodeParameter(id, name, updated_widget) => {
                //TODO: move into Network
                self.network.stash_state();
                let old_template = &mut self.network.graph.get_mut_node(id);
                old_template.parameters_values.insert(name, updated_widget);
                return Task::done(Message::QueueCompute(id));
            }
            Message::OpenAddNodeUi => self.action = Action::AddingNode,
            Message::SelectNodeGroup(selected_tree_path) => match &self.action {
                Action::AddingNode => {
                    let current_path = self.user_data.get_new_node_path();
                    if current_path.starts_with(&selected_tree_path) {
                        self.user_data.set_new_node_path(
                            &selected_tree_path[0..selected_tree_path.len().saturating_sub(1)],
                        );
                    } else {
                        self.user_data.set_new_node_path(&selected_tree_path);
                        //self.action = Action::AddingNode(selected_tree_path)
                    }
                }
                _ => error!(
                    "should not be able to select a nope group while Add Node UI is not active"
                ),
            },
            Message::AddNode(template) => {
                //TODO: move into Network
                self.network.stash_state();
                let id = self.network.graph.node(template.into());
                self.network.selected_shapes = [id].into();
                self.network.shapes.shape_positions.insert_before(
                    0,
                    id,
                    self.cursor_position + self.network.shapes.camera.position,
                );
                self.action = Action::DragNode(vec![(id, [0.0, 0.0].into())])
            }
            Message::DeleteSelectedNodes => {
                //TODO: move into Network
                if !self.network.selected_shapes.is_empty() {
                    self.network.stash_state();
                    self.network.selected_shapes.iter().for_each(|id| {
                        self.network.graph.delete_node(*id);
                        self.network.shapes.shape_positions.swap_remove(id);
                    });
                    self.network.selected_shapes = [].into();
                    //PERF: ideally, we should only execute affected nodes
                    return Task::done(Message::ComputeAll);
                }
            }

            Message::AnimationTick => {}
            Message::ThemeValueChange(tm, tv) => self.app_theme.update(tm, tv),
            Message::ToggleDebug => {
                self.debug = !self.debug;
            }
            Message::TogglePaletteUI => {
                self.show_palette_ui = !self.show_palette_ui;
            }
            Message::New => {
                //TODO: move into Network
                if self.network.unsaved_changes {
                    let result = rfd::MessageDialog::new()
                        .set_title("Save Changes?")
                        .set_description(
                            "Network has unsaved changes, save before opening new network?",
                        )
                        .set_buttons(rfd::MessageButtons::YesNoCancel)
                        .show();
                    match result {
                        rfd::MessageDialogResult::Yes => {
                            return Task::done(Message::Save).chain(Task::done(Message::New))
                        }
                        rfd::MessageDialogResult::Cancel => return Task::none(),
                        _ => {}
                    }
                }
                self.network = Network::default();
                self.reload_nodes();
            }
            Message::Load => {
                //TODO: move into Network
                if self.network.unsaved_changes {
                    let result = rfd::MessageDialog::new()
                        .set_title("Save Changes?")
                        .set_description(
                            "Network has unsaved changes, save before opening new network?",
                        )
                        .set_buttons(rfd::MessageButtons::YesNoCancel)
                        .show();
                    match result {
                        rfd::MessageDialogResult::Yes => {
                            return Task::done(Message::Save).chain(Task::done(Message::Load))
                        }
                        rfd::MessageDialogResult::Cancel => return Task::none(),
                        _ => {}
                    }
                }
                let file = FileDialog::new()
                    .set_directory(self.user_data.network_search_dir())
                    .add_filter("network", &["ron"])
                    .pick_file();

                if let Some(file) = file {
                    self.network = ron::from_str(
                        &read_to_string(&file)
                            .unwrap_or_else(|e| panic!("Could not read network {file:?}\n {e}")),
                    )
                    .unwrap_or_else(|e| panic!("Could not parse network {file:?}\n {e}"));
                    self.network.file = Some(file.clone());
                    self.user_data.set_recent_network_file(Some(file));
                    self.reload_nodes();
                    return Task::done(Message::ComputeAll);
                } else {
                    info!("File not picked")
                }
            }
            Message::Save => {
                //TODO: move into Network
                let file = match self.network.file.clone() {
                    Some(file) => Some(file),
                    None => FileDialog::new()
                        .set_directory(self.user_data.network_search_dir())
                        .add_filter("network", &["ron"])
                        .save_file(),
                };
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
            Message::ReloadNodes => {
                self.reload_nodes();
                return Task::done(Message::ComputeAll);
            }
            Message::WindowOpen => {
                if self.action == Action::InitialLoad {
                    self.action = Action::Idle;
                    return Task::done(Message::ComputeAll);
                }
            }
            Message::ModifiersChanged(m) => {
                self.modifiers = m;
            }

            //// Focus
            Message::FocusNext => return focus_next(),
            Message::FocusPrevious => return focus_previous(),

            //// History
            Message::Undo => {
                //TODO: move into Network
                if let Some(prev) = self.network.undo_stack.pop() {
                    self.network.redo_stack.push((
                        self.network.graph.clone(),
                        self.network.shapes.shape_positions.clone(),
                    ));
                    self.network.graph = prev.0;
                    self.network.shapes.shape_positions = prev.1;
                    return Task::done(Message::ComputeAll);
                }
            }
            Message::Redo => {
                //TODO: move into Network
                if let Some(next) = self.network.redo_stack.pop() {
                    self.network.undo_stack.push((
                        self.network.graph.clone(),
                        self.network.shapes.shape_positions.clone(),
                    ));
                    self.network.graph = next.0;
                    self.network.shapes.shape_positions = next.1;
                    return Task::done(Message::ComputeAll);
                }
            }
            Message::ComputeAll => {
                //TODO: move into Network
                let nodes = self.network.graph.get_roots();
                trace!("Queuing root nodes: {nodes:?}");
                return Task::batch(
                    nodes
                        .into_iter()
                        .map(|nx| Task::done(Message::QueueCompute(nx))),
                );
            }
            Message::QueueCompute(nx) => {
                //TODO: move into Network
                //// Modify node status
                {
                    let node = self.network.graph.get_mut_node(nx);

                    // Check if in an error state
                    if let NodeStatus::Error(e) = &node.status {
                        trace!(
                            "Did not compute node in error state {e:?} compute: {:?} #{nx}",
                            node.template,
                        );
                        return Task::none();
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
                    move |(nx, res)| Message::ComputeComplete(nx, res),
                );
            }
            Message::ComputeComplete(nx, result) => {
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
                                    ForayNodeTemplate::RustNode(RustNodeTemplate::Constant(_)) => {
                                        self.network.graph.get_node(nx).template.clone()
                                    }
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
                                .map(|node| Task::done(Message::QueueCompute(node))),
                        );
                    }
                    Err(node_error) => {
                        //// Update Node
                        let node = self.network.graph.get_mut_node(nx);
                        warn!("Compute failed {node:?},{node_error:?}");
                        //TODO: set node error!!!!
                        node.status = NodeStatus::Idle;

                        //// Update Wire
                        self.network.graph.update_wire_data(nx, [].into());

                        return Task::none();
                    }
                };
            }
        };
        Task::none()
    }

    /// App View
    pub fn view(&'_ self) -> Element<'_, Message, Theme, Renderer> {
        let content = column![
            row![
                side_bar(self),
                vertical_rule(SEPERATOR),
                container(
                    workspace(
                        &self.network.shapes,
                        //// Node view
                        |id| self.node_content(id),
                        //// Wires paths
                        |wire_end_node, points| self.wire_curve(wire_end_node, points),
                    )
                    .on_cursor_move(Message::OnMove)
                    .on_press(Message::OnCanvasDown)
                    .on_release(Message::OnCanvasUp)
                    .pan(Message::ScrollPan)
                )
                .height(Fill)
                .width(Fill)
            ],
            match self.show_palette_ui {
                true => column![horizontal_rule(SEPERATOR), self.app_theme.view()],
                false => column![],
            }
        ];

        let output: Element<Message, Theme, Renderer> = match &self.action {
            Action::AddingNode => stack![
                content,
                // Barrier to stop interaction
                mouse_area(
                    container(text(""))
                        .center(Fill)
                        .style(container::transparent)
                )
                // Stop any mouseover interactions from showing,
                .interaction(mouse::Interaction::Idle)
                .on_press(Message::Cancel),
                //// Add node modal
                container(
                    mouse_area(add_node_tree_panel(
                        &self.projects,
                        self.user_data.get_new_node_path()
                    ))
                    .interaction(mouse::Interaction::Idle)
                    .on_press(Message::NOP)
                )
                .center(Fill)
            ]
            .into(),
            _ => content.into(),
        };

        // Potentially add a specific mouse cursor
        let output = match self.action {
            Action::DragNode(_) => mouse_area(output)
                .interaction(mouse::Interaction::Move)
                .into(),
            _ => output,
        };

        if self.debug {
            output.explain(iced::Color::from_rgba(0.7, 0.7, 0.8, 0.2))
        } else {
            output
        }
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
                let new_py_node = PyNodeTemplate::new(py_path);

                // TODO: Implement parameter merging

                //// Update Parameters
                // let new_parameters = {
                //     // If Ok, copy old parameters to new parameters
                //     if let (Ok(new_parameters), Ok(old_param)) =
                //         (new_py_node.parameters(), &old_parameters)
                //     {
                //         // Only keep old values that are still present in the new parameters list
                //         Ok(new_parameters
                //             .clone()
                //             .into_iter()
                //             .chain(old_param.clone().into_iter().filter(|(k, v)| {
                //                 if let Some(new_v) = new_parameters.get(k) {
                //                     discriminant(v) == discriminant(new_v)
                //                 } else {
                //                     false
                //                 }
                //             }))
                //             .collect())
                //     } else {
                //         warn!(
                //             "Paramaters not ok, not loading.\nNew: {:?}\nOld: {:?}",
                //             &new_py_node.parameters(),
                //             &old_parameters
                //         );
                //         new_py_node.parameters()
                //     }
                // };

                //// Update Ports, and Graph Edges
                {
                    let new_in_ports = new_py_node.inputs().unwrap_or_default();
                    let new_out_ports = new_py_node.outputs().unwrap_or_default();

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
                            p.name, new_py_node.name
                        );
                        self.network.graph.remove_edge(&p);
                    });
                }

                // Update Graph Node
                self.network
                    .graph
                    .set_node_data(*nx, ForayNodeTemplate::PyNode(new_py_node).into());
            }
        });
        // Update list of available nodes
        self.projects = read_python_projects();
    }
}

pub fn theme(state: &App) -> Theme {
    state.app_theme.clone().into()
}

pub fn subscriptions(state: &App) -> Subscription<Message> {
    Subscription::batch(
        state
            .projects
            .iter()
            .filter(|p| !p.absolute_path.to_string_lossy().is_empty())
            .enumerate()
            .map(|(id, p)| file_watch_subscription(id, p.absolute_path.clone()))
            .chain([
                window::open_events().map(|_| Message::WindowOpen),
                listen_with(|event, _status, _id| match event {
                    Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                        Some(Message::ModifiersChanged(m))
                    }
                    Keyboard(KeyPressed { key, modifiers, .. }) => match key {
                        Key::Named(Named::Tab) => {
                            if modifiers.contains(Modifiers::SHIFT) {
                                Some(Message::FocusPrevious)
                            } else {
                                Some(Message::FocusNext)
                            }
                        }
                        Key::Named(Named::Delete) => Some(Message::DeleteSelectedNodes),
                        Key::Named(Named::Escape) => Some(Message::Cancel),
                        Key::Character(smol_str) => {
                            if modifiers.control() && smol_str == "a" {
                                Some(Message::OpenAddNodeUi)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    _ => None,
                }),
                // Refresh for animation while nodes are actively running
                if state.network.any_nodes_running() {
                    iced::time::every(Duration::from_micros(1_000_000 / 16))
                        .map(|_| Message::AnimationTick)
                } else {
                    Subscription::none()
                },
            ]),
    )
}

pub fn title(state: &App) -> String {
    let pre_pend = match state.network.unsaved_changes {
        true => "*",
        false => "",
    };
    pre_pend.to_string()
        + &state
            .network
            .file
            .clone()
            .map(|p| p.file_stem().unwrap().to_string_lossy().to_string())
            .unwrap_or("*new".to_string())
}
