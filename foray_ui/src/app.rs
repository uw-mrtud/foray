use crate::config::Config;
use crate::interface::theme_config::{AppThemeMessage, GuiColorMessage};
use crate::style::theme::AppTheme;
use crate::workspace::{Workspace, WorkspaceMessage};

use derive_more::Debug;
use iced::advanced::graphics::core::Element;
use iced::event::listen_with;
use iced::Event::Keyboard;
use iced::Length::Fill;
use iced::{keyboard, window, Font, Renderer, Subscription, Task, Theme};
use iced::{
    keyboard::{key::Named, Event::KeyPressed, Key, Modifiers},
    widget::{
        button, column, container, markdown,
        operation::{focus_next, focus_previous},
        row, rule, space, text,
    },
    Alignment::Center,
};

use log::warn;
use std::path::PathBuf;

pub struct App {
    pub workspace: Option<Workspace>,

    pub app_theme: AppTheme,

    pub main_window_id: Option<window::Id>,
    /// Currently held keyboard modifiers, used for shortcuts
    pub modifiers: Modifiers,

    pub debug: bool,
    pub show_palette_ui: bool,

    markdown: Vec<markdown::Item>,
}
impl App {
    pub fn new(working_dir: PathBuf, cli_network_path: Option<PathBuf>) -> Self {
        let workspace = match Workspace::new(working_dir, cli_network_path, None) {
            Ok(workspace) => Some(workspace),
            Err(e) => {
                warn!("Workspace Initialialization Error: {:?}", e);
                None
            }
        };
        App {
            workspace,

            debug: false,
            show_palette_ui: false,

            app_theme: Config::load_theme(),
            main_window_id: None,
            modifiers: Default::default(),
            markdown: markdown::parse("First time?\n\nFollow the [guide](https://uw-mrtud.github.io/foray/book/installation.html#quick-start) for info on creating a workspace").collect()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    StartWorkspaceSelect,
    EndWorkspaceSelect(Option<PathBuf>),
    StartNetworkSelect,
    EndNetworkSelect(Option<PathBuf>),
    LinkClicked(markdown::Uri),

    ThemeValueChange(AppThemeMessage, GuiColorMessage),
    ToggleDebug,
    TogglePaletteUI,

    ModifiersChanged(Modifiers),

    //// Focus
    FocusNext,
    FocusPrevious,

    WorkspaceMessage(crate::workspace::WorkspaceMessage),
    OpenWindow(window::Id),
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        //match message {
        //    _ => trace!("---Message--- {message:?} {:?}", Instant::now()),
        //}
        match message {
            Message::WorkspaceMessage(m) => {
                return self
                    .workspace
                    .as_mut()
                    .expect("workspace should exist")
                    .update(m, self.modifiers)
                    .map(Message::WorkspaceMessage);
            }
            Message::ThemeValueChange(tm, tv) => self.app_theme.update(tm, tv),
            Message::ToggleDebug => {
                self.debug = !self.debug;
            }
            Message::TogglePaletteUI => {
                self.show_palette_ui = !self.show_palette_ui;
            }
            Message::ModifiersChanged(m) => {
                self.modifiers = m;
            }

            //// Focus
            Message::FocusNext => return focus_next(),
            Message::FocusPrevious => return focus_previous(),
            Message::StartWorkspaceSelect => {
                return Task::perform(select_workspace_dialog(), Message::EndWorkspaceSelect)
            }
            Message::EndWorkspaceSelect(Some(workspace_path)) => {
                self.workspace = match Workspace::is_valid_workspace(&workspace_path) {
                    true => match Workspace::new(workspace_path, None, self.main_window_id) {
                        Ok(workspace) => Some(workspace),
                        Err(_e) => {
                            warn!("Current directory is not a valid workspace");
                            None
                        }
                    },
                    false => None,
                };
                return Task::done(Message::WorkspaceMessage(WorkspaceMessage::ComputeAll));
            }
            Message::EndWorkspaceSelect(None) => {}
            Message::StartNetworkSelect => {
                return Task::perform(select_network_dialog(), Message::EndNetworkSelect)
            }
            Message::EndNetworkSelect(result) => match result {
                Some(network_path) => {
                    let workspace_path = network_path
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .to_path_buf();

                    self.workspace = match Workspace::is_valid_workspace(&workspace_path) {
                        true => match Workspace::new(
                            workspace_path,
                            Some(network_path),
                            self.main_window_id,
                        ) {
                            Ok(workspace) => Some(workspace),
                            Err(_e) => {
                                warn!("Current directory is not a valid workspace");
                                None
                            }
                        },
                        false => None,
                    };
                    return Task::done(Message::WorkspaceMessage(WorkspaceMessage::ComputeAll));
                }
                None => {}
            },
            Message::LinkClicked(link) => {
                let _ = open::that_in_background(link.to_string());
            }
            Message::OpenWindow(id) => {
                if self.main_window_id == None {
                    self.main_window_id = Some(id);
                    if let Some(workspace) = &mut self.workspace {
                        workspace.main_window_id = Some(id);
                        return Task::done(Message::WorkspaceMessage(WorkspaceMessage::ComputeAll));
                    }
                }
            }
        };
        Task::none()
    }

    /// App View
    pub fn view(&'_ self) -> Element<'_, Message, Theme, Renderer> {
        let workspace_content = match &self.workspace {
            Some(w) => w.view(&self.app_theme).map(Message::WorkspaceMessage),
            None => container(
                column![
                    // svg(svg::Handle::from_memory(
                    //     include_bytes!("../data/resources/Foray.svg").as_slice()
                    // )),
                    text("Foray").font(Font::with_name("Courier New")).size(120),
                    space(),
                    row![
                        button("Load Workspace").on_press(Message::StartWorkspaceSelect),
                        text("or"),
                        button("Load Network").on_press(Message::StartNetworkSelect),
                    ]
                    .align_y(Center)
                    .spacing(20),
                    space::vertical(),
                    rule::horizontal(1),
                    container(
                        markdown::view(&self.markdown, Theme::from(self.app_theme.clone()))
                            .map(Message::LinkClicked)
                    )
                    .padding([20, 20])
                ]
                .align_x(Center)
                .width(400),
            )
            .center_x(Fill)
            .center_y(Fill)
            .padding(60)
            .into(),
        };
        workspace_content
    }
}

pub fn theme(state: &App) -> Theme {
    state.app_theme.clone().into()
}

pub fn subscriptions(state: &App) -> Subscription<Message> {
    let worksace_sub = match &state.workspace {
        Some(workspace) => workspace.subscriptions().map(Message::WorkspaceMessage),
        None => Subscription::none(),
    };
    Subscription::batch([
        worksace_sub,
        window::open_events().map(|id| Message::OpenWindow(id)),
        listen_with(|event, _status, _id| match event {
            Keyboard(keyboard::Event::ModifiersChanged(m)) => Some(Message::ModifiersChanged(m)),
            Keyboard(KeyPressed { key, modifiers, .. }) => match key {
                Key::Named(Named::Tab) => {
                    if modifiers.contains(Modifiers::SHIFT) {
                        Some(Message::FocusPrevious)
                    } else {
                        Some(Message::FocusNext)
                    }
                }
                _ => None,
            },
            _ => None,
        }),
    ])
}

pub fn title(state: &App) -> String {
    match &state.workspace {
        Some(w) => {
            let pre_pend = match w.network.unsaved_changes {
                true => "*",
                false => "",
            };
            pre_pend.to_string()
                + &w.network
                    .file
                    .clone()
                    .map(|p| p.file_stem().unwrap().to_string_lossy().to_string())
                    .unwrap_or("*new".to_string())
        }
        None => "".into(),
    }
}
pub fn home_dir() -> PathBuf {
    let user_dirs = directories::UserDirs::new().expect("user directory should be  accessible");
    user_dirs.home_dir().to_path_buf()
}

async fn select_network_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Open a network file...")
        .set_directory(home_dir())
        .add_filter("network", &["network"])
        .pick_file()
        .await
        .map(|fh| fh.into())
}
/// Open a file dialog to save a network file
async fn select_workspace_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(home_dir())
        .pick_folder()
        .await
        .map(|fh| fh.into())
}

/// Open a generic dialog
pub async fn file_dialog(default_path: PathBuf) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(default_path)
        .pick_file()
        .await
        .map(|fh| fh.into())
}
