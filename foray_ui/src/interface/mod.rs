use std::fmt::Debug;
pub mod add_node;
pub mod node;
pub mod node_config;
pub mod numeric_input;
pub mod port;
pub mod side_bar;
pub mod theme_config;
pub mod wire;

pub const SEPERATOR: f32 = 1.0;

pub fn debug_format<T, U>(debug: &bool, default_text: T, debug_info: U) -> String
where
    T: derive_more::Display,
    U: Debug,
{
    match debug {
        true => format!("{default_text}{debug_info:?}"),
        false => format!("{default_text}"),
    }
}
