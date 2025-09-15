use iced::{
    self, Element, Renderer, Task, Theme,
    event::listen_with,
    widget::{
        canvas, canvas::Stroke, column, container, container::bordered_box, mouse_area, text,
    },
};

use iced::Event::Keyboard;
use iced::keyboard::Event::KeyPressed;
use node_canvas::node_canvas::{Camera, ShapeContext};

struct State {
    pub nodes: Vec<((f32, f32), Node)>,
    pub camera: Camera,
}

pub fn main() -> Result<(), iced::Error> {
    iced::application("example canvas", update, view)
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
            let a = 20.0;
            let b = 200.0;
            (
                State {
                    nodes: vec![
                        ((0.0, 0.0), "0".into()),
                        ((a, 0.0), "1".into()),
                        ((-a, 0.0), "2".into()),
                        ((0.0, a), "3".into()),
                        ((0.0, -a), "4".into()),
                        ((b, b), "5".into()),
                        ((a + b, b), "6".into()),
                        ((-a + b, b), "7".into()),
                        ((b, a + b), "8".into()),
                        ((b, -a + b), "9".into()),
                    ],
                    camera: Default::default(),
                },
                Task::none(),
            )
        })
}

#[derive(Default)]
struct Node {
    name: String,
}
impl From<&str> for Node {
    fn from(name: &str) -> Self {
        Node {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    UpdateCamera(Camera),
    // Pan(ScrollDelta),
    // ZoomIn,
    // ZoomOut,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::UpdateCamera(camera) => state.camera = camera,
    }
    Task::none()
}

fn view(state: &'_ State) -> Element<'_, Message> {
    column![
        container(
            container(
                mouse_area(
                    node_canvas::node_canvas::NodeCanvas::new(&state.nodes, state.camera)
                        .on_update_camera(Message::UpdateCamera)
                ) // .on_scroll(Message::Pan),
            )
            .style(bordered_box),
        )
        .padding(20),
        text(format!(
            "camera: ({},{}),zoom: {}",
            state.camera.position.0, state.camera.position.1, state.camera.zoom
        ))
    ]
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

        let text = canvas::Text::from(self.name.clone());
        text.draw_with(|path, color| {
            frame.fill(&path, color);
            frame.stroke(&path, Stroke::default().with_color(color));
        });

        vec![frame.into_geometry()]
    }
}
