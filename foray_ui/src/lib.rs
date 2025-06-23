use std::collections::BTreeMap;

use iced::Font;

pub type StableMap<K, V> = BTreeMap<K, V>;
pub const DEFAULT_FONT: Font = Font::with_name("CaskaydiaCove");
pub const MATH_FONT: Font = Font::with_name("DejaVu Math TeX Gyre");
pub const SYMBOL_FONT: Font = Font::with_name("CaskaydiaCove Nerd Font");

pub mod app;
pub mod config;
pub mod file_watch;
pub mod graph;
pub mod gui_node;
pub mod interface;
pub mod math;
pub mod network;
pub mod nodes;
pub mod project;
pub mod python;
pub mod style;
pub mod user_data;
pub mod widget;
