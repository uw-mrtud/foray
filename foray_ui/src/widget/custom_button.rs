use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::Operation;
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::border::{self, Border};
use iced::event::{self, Event};
use iced::mouse;
use iced::overlay;
use iced::theme::palette;
use iced::touch;
use iced::{Background, Color, Element, Length, Padding, Rectangle, Shadow, Size, Theme, Vector};

use crate::math::Point;

#[allow(missing_debug_implementations)]
pub struct Button<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_hover: Option<OnInteract<'a, Message>>,
    on_drag: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_press: Option<OnInteract<'a, Message>>,
    on_right_press: Option<OnInteract<'a, Message>>,
    on_release_self: Option<OnInteract<'a, Message>>,
    on_release_other: Option<OnInteract<'a, Message>>,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    class: Theme::Class<'a>,
}

enum OnInteract<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<Message: Clone> OnInteract<'_, Message> {
    fn get(&self) -> Message {
        match self {
            OnInteract::Direct(message) => message.clone(),
            OnInteract::Closure(f) => f(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Button {
            content,
            on_hover: None,
            on_drag: None,
            on_press: None,
            on_right_press: None,
            on_release_self: None,
            on_release_other: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: DEFAULT_PADDING,
            clip: false,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(OnInteract::Direct(on_press));
        self
    }
    /// Sets the message that will be produced when the right [`Button`] is pressed.
    pub fn on_right_press(mut self, on_right_press: Message) -> Self {
        self.on_right_press = Some(OnInteract::Direct(on_right_press));
        self
    }
    /// Sets the message that will be produced when the [`Button`] is released
    pub fn on_release_self(mut self, on_release_self: Message) -> Self {
        self.on_release_self = Some(OnInteract::Direct(on_release_self));
        self
    }
    pub fn on_release_other(mut self, on_release_other: Message) -> Self {
        self.on_release_other = Some(OnInteract::Direct(on_release_other));
        self
    }
    pub fn on_hover(mut self, on_hover: Message) -> Self {
        self.on_hover = Some(OnInteract::Direct(on_hover));
        self
    }

    pub fn on_drag(mut self, on_move: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_drag = Some(Box::new(on_move));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// This is analogous to [`Button::on_press`], but using a closure to produce
    /// the message.
    ///
    /// This closure will only be called when the [`Button`] is actually pressed and,
    /// therefore, this method is useful to reduce overhead if creating the resulting
    /// message is slow.
    pub fn on_press_with(mut self, on_press: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnInteract::Closure(Box::new(on_press)));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press.map(OnInteract::Direct);
        self
    }

    /// Sets whether the contents of the [`Button`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Button`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Button`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_pressed: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(on_press) = self.on_press.as_ref().map(OnInteract::get) {
                    if cursor.is_over(bounds) {
                        state.is_pressed = true;
                        if cursor.is_over(bounds) {
                            shell.publish(on_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let Some(on_right_press) = self.on_right_press.as_ref().map(OnInteract::get) {
                    if cursor.is_over(bounds) {
                        state.is_pressed = false;
                        if cursor.is_over(bounds) {
                            shell.publish(on_right_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if self.on_press.is_some() {
                    if state.is_pressed {
                        state.is_pressed = false;

                        if let Some(on_release_self) =
                            self.on_release_self.as_ref().map(OnInteract::get)
                        {
                            //if cursor.is_over(bounds) {
                            shell.publish(on_release_self);
                            //}

                            return event::Status::Captured;
                        }
                    } else if let Some(on_release_other) =
                        self.on_release_other.as_ref().map(OnInteract::get)
                    {
                        if cursor.is_over(bounds) {
                            shell.publish(on_release_other);
                            return event::Status::Captured;
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(on_hover) = self.on_hover.as_ref().map(OnInteract::get) {
                    if cursor.is_over(bounds) {
                        //let state = tree.state.downcast_mut::<State>();
                        //
                        //state.is_pressed = true;
                        //if cursor.is_over(bounds) {
                        shell.publish(on_hover);
                        //}

                        return event::Status::Captured;
                    }
                }

                if let Some(on_drag) = &self.on_drag {
                    if state.is_pressed {
                        shell.publish(on_drag(cursor.position().unwrap().into()));
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                let state = tree.state.downcast_mut::<State>();

                state.is_pressed = false;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let is_mouse_over = cursor.is_over(bounds);

        let status = if self.on_press.is_none() {
            Status::Disabled
        } else if is_mouse_over {
            let state = tree.state.downcast_ref::<State>();

            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        };

        let style = theme.style(&self.class, status);

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let is_mouse_over = cursor.is_over(layout.bounds());

        if state.is_pressed {
            mouse::Interaction::Grabbing
        } else if is_mouse_over && self.on_press.is_some() {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}

/// The default [`Padding`] of a [`Button`].
pub(crate) const DEFAULT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

/// The possible status of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Button`] can be pressed.
    Active,
    /// The [`Button`] can be pressed and it is being hovered.
    Hovered,
    /// The [`Button`] is being pressed.
    Pressed,
    /// The [`Button`] cannot be pressed.
    Disabled,
}

/// The style of a button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the button.
    pub background: Option<Background>,
    /// The text [`Color`] of the button.
    pub text_color: Color,
    /// The [`Border`] of the buton.
    pub border: Border,
    /// The [`Shadow`] of the butoon.
    pub shadow: Shadow,
}

impl Style {
    /// Updates the [`Style`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::BLACK,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

/// The theme catalog of a [`Button`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Button`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// circular button
pub fn custom(status: Status, outer_color: Color, hover_color: Color, fill_color: Color) -> Style {
    let base = Style {
        background: Some(Background::Color(fill_color)),
        border: border::color(outer_color).rounded(2).width(3),
        ..Style::default()
    };
    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(hover_color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}
/// A primary button; denoting a main action.
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = styled(palette.primary.strong);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.primary.base.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A secondary button; denoting a complementary action.
pub fn secondary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = styled(palette.secondary.base);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.secondary.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A success button; denoting a good outcome.
pub fn success(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = styled(palette.success.base);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.success.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A danger button; denoting a destructive action.
pub fn danger(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = styled(palette.danger.base);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.danger.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A text button; useful for links.
pub fn text(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let base = Style {
        text_color: palette.background.base.text,
        ..Style::default()
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            text_color: palette.background.base.text.scale_alpha(0.8),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn styled(pair: palette::Pair) -> Style {
    Style {
        background: Some(Background::Color(pair.color)),
        text_color: pair.text,
        border: border::rounded(2),
        ..Style::default()
    }
}

fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}
