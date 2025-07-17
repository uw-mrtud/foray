use crate::app::{App, Message};
use crate::gui_node::config_view;
use crate::interface::node::format_node_output;
use crate::interface::status::{node_status_icon, node_status_text_element};
use crate::interface::{debug_format, SEPERATOR};
use crate::style::button::{primary_icon, secondary_icon};
use crate::style::icon::icon;
use iced::*;
use widget::{column, *};

/// Create the sidebar view
pub fn side_bar(app: &App) -> Element<'_, Message> {
    fn file_button<'a>(lbl: impl Into<String>, message: Message) -> Button<'a, Message> {
        button(icon(lbl.into()))
            .on_press(message)
            .style(primary_icon)
            .padding(0.0)
    }
    fn undo_button<'a>(
        lbl: impl Into<String>,
        enabled: bool,
        message: Message,
    ) -> Button<'a, Message> {
        button(icon(lbl.into()))
            .on_press_maybe(if enabled { Some(message) } else { None })
            .padding(0.0)
            .style(secondary_icon)
    }

    //''
    //''
    //''
    //''
    //''
    let file_commands = row![
        file_button('󰝒', Message::New),
        file_button('󰝰', Message::Load),
        file_button('󰆓', Message::Save),
        file_button('󰃤', Message::ToggleDebug),
        file_button('󰏘', Message::TogglePaletteUI),
    ]
    .spacing(3.0);

    let undo = undo_button(
        debug_format(&app.debug, '', app.network.undo_stack.len()),
        !app.network.undo_stack.is_empty(),
        Message::Undo,
    );
    let redo = undo_button(
        debug_format(&app.debug, '', app.network.redo_stack.len()),
        !app.network.redo_stack.is_empty(),
        Message::Redo,
    );
    let action_commands = row![horizontal_space(), undo, redo].spacing(4.0);

    //// Config
    let config: Element<Message> =
        if let Some(selected_id) = app.network.selected_shapes.iter().next() {
            let node = app.network.graph.get_node(*selected_id);
            let input_data = app.network.graph.get_input_data(selected_id);
            let out_port_display: Element<Message> = if app.debug {
                column![format_node_output(
                    node,
                    &app.network.graph.get_output_data(*selected_id)
                )]
                .into()
            } else {
                text("").into()
            };
            column![
                container(text(node.template.name().clone()).size(20.)).center_x(Fill),
                horizontal_rule(0),
                row![
                    node_status_icon(&node.status),
                    node_status_text_element(&node.status).size(12.)
                ]
                .align_y(Center)
                .spacing(8.0),
                vertical_space().height(10.),
                config_view(node, *selected_id, input_data).unwrap_or(text("...").into()),
                // node.config_view(*selected_id, input_data)
                //     .unwrap_or(text("...").into()),
                vertical_space(),
                scrollable(out_port_display),
                row![button(text("delete node"))
                    .style(button::danger)
                    .padding([1, 4])
                    .on_press(Message::DeleteSelectedNodes)]
            ]
            .align_x(Center)
            .height(Fill)
            .spacing(5.)
            .padding([10., 5.])
            .into()
        } else {
            text("").into()
        };
    container(
        column![
            row![
                //// File
                file_commands.align_y(Alignment::Center),
                horizontal_space(),
                //// Actions
                action_commands.align_y(Alignment::Center),
            ]
            .padding([2., 4.]),
            horizontal_rule(SEPERATOR),
            //// Config
            config
        ]
        .height(Fill)
        .width(200.),
    )
    .into()
}
