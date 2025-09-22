use iced::{
    self, Color, Element, Font, Renderer, Task, Theme,
    event::listen_with,
    font::Weight,
    widget::{canvas::Stroke, column, mouse_area, text},
};

use iced::Event::Keyboard;
use iced::keyboard::Event::KeyPressed;
use iced::widget::canvas;
use iced::widget::container;
use node_canvas::{camera::Camera, shape_context::ShapeContext};

struct State {
    pub nodes: Vec<((f32, f32), Node)>,
    pub camera: Camera,
    pub selected_node: Option<u32>,
}

pub fn main() -> Result<(), iced::Error> {
    iced::application("example canvas", update, view)
        .antialiasing(true)
        .subscription(|_s| {
            listen_with(|event, _status, _id| match event {
                Keyboard(KeyPressed { key, .. }) => match key {
                    // Key::Named(Named::ArrowUp) => Some(Message::ZoomIn),
                    // Key::Named(Named::ArrowDown) => Some(Message::ZoomOut),
                    _ => None,
                },
                _ => None,
            })
        })
        .run_with(|| {
            (
                State {
                    nodes: (1..=200)
                        .rev()
                        .map(|s| ((0.0, s as f32 * 30.0), Node::new(16, "multiply")))
                        .collect(),
                    camera: Default::default(),
                    selected_node: None,
                },
                Task::none(),
            )
        })
}

#[derive(Default)]
struct Node {
    size: u32,
    name: String,
}

impl Node {
    fn new(size: u32, text: &str) -> Self {
        Self {
            size,
            name: text.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    UpdateCamera(Camera),
    NodeClicked(u32),
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::UpdateCamera(camera) => state.camera = camera,
        Message::NodeClicked(id) => state.selected_node = Some(id),
    }
    Task::none()
}

fn view(state: &'_ State) -> Element<'_, Message> {
    container(column![
        container(container(mouse_area(
            node_canvas::node_canvas::NodeCanvas::new(&state.nodes, state.camera)
                .on_update_camera(Message::UpdateCamera)
        ),))
        .padding(20),
        text(format!(
            "camera: ({},{}),zoom: {}",
            state.camera.position.0, state.camera.position.1, state.camera.zoom
        ))
    ])
    .style(|_| container::background(iced::Color::new(0.5, 0.5, 0.5, 0.5)))
    .into()
}

impl iced::widget::canvas::Program<Message> for Node {
    type State = ShapeContext;

    fn draw(
        &self,
        shape_context: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry<Renderer>> {
        let mut frame = shape_context.frame_in_shape_space(&renderer, bounds);

        frame.scale(self.size as f32 / 22.0);
        let text = canvas::Text {
            font: Font {
                family: iced::font::Family::Name("Courier New"),
                ..Default::default()
            },
            content: self.name.clone(),
            color: Color::from_rgb(1.0, 1.0, 1.0),
            // size: (self.size as u16).into(),
            size: 22.into(),
            ..Default::default()
        };
        frame.fill_text(text);
        // text.draw_with(|path, _color| {
        //     frame.fill(&path, text_color);
        //     // frame.stroke(&path, Stroke::default().with_color(text_color));
        // });

        vec![frame.into_geometry()]
    }
    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(event) => match event {
                iced::mouse::Event::ButtonPressed(button) => match button {
                    iced::mouse::Button::Left => {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::NodeClicked(self.size)),
                        );
                    }
                    _ => {}
                },
                _ => {}
            },
            canvas::Event::Touch(event) => todo!(),
            canvas::Event::Keyboard(event) => todo!(),
        }
        (canvas::event::Status::Ignored, None)
    }
}
