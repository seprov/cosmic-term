// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::{
    iced::{
        event::{Event, Status},
        keyboard::Modifiers,
        mouse::{self},
        Element, Length, Padding, Point, Rectangle, Size,
    },
    iced_core::{
        clipboard::Clipboard,
        layout::{self, Layout},
        renderer::{self},
        widget::{
            self,
            operation::{self, Operation, OperationOutputWrapper},
            tree, Id, Widget,
        },
        Border, Shell,
    },
    theme::Theme,
    Renderer,
};

use std::{
    cell::Cell,
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{key_bind::key_binds, Action, Terminal};

use super::{drawer, event_handler};

pub struct TerminalBox<'a, Message> {
    pub(super) terminal: &'a Mutex<Terminal>,
    id: Option<Id>,
    pub(super) border: Border,
    pub(super) padding: Padding,
    pub(super) click_timing: Duration,
    pub(super) context_menu: Option<Point>,
    pub(super) on_context_menu: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    pub(super) on_mouse_enter: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) opacity: Option<f32>,
    pub(super) mouse_inside_boundary: Option<bool>,
    pub(super) on_middle_click: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) key_binds: HashMap<KeyBind, Action>,
}

impl<'a, Message> TerminalBox<'a, Message>
where
    Message: Clone,
{
    pub fn new(terminal: &'a Mutex<Terminal>) -> Self {
        Self {
            terminal,
            id: None,
            border: Border::default(),
            padding: Padding::new(0.0),
            click_timing: Duration::from_millis(500),
            context_menu: None,
            on_context_menu: None,
            on_mouse_enter: None,
            opacity: None,
            mouse_inside_boundary: None,
            on_middle_click: None,
            key_binds: key_binds(),
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn border<B: Into<Border>>(mut self, border: B) -> Self {
        self.border = border.into();
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn click_timing(mut self, click_timing: Duration) -> Self {
        self.click_timing = click_timing;
        self
    }

    pub fn context_menu(mut self, position: Point) -> Self {
        self.context_menu = Some(position);
        self
    }

    pub fn on_context_menu(
        mut self,
        on_context_menu: impl Fn(Option<Point>) -> Message + 'a,
    ) -> Self {
        self.on_context_menu = Some(Box::new(on_context_menu));
        self
    }

    pub fn on_mouse_enter(mut self, on_mouse_enter: impl Fn() -> Message + 'a) -> Self {
        self.on_mouse_enter = Some(Box::new(on_mouse_enter));
        self
    }

    pub fn on_middle_click(mut self, on_middle_click: impl Fn() -> Message + 'a) -> Self {
        self.on_middle_click = Some(Box::new(on_middle_click));
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }
}

pub fn terminal_box<Message>(terminal: &Mutex<Terminal>) -> TerminalBox<'_, Message>
where
    Message: Clone,
{
    TerminalBox::new(terminal)
}

impl<'a, Message> Widget<Message, cosmic::Theme, Renderer> for TerminalBox<'a, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Fill);

        let mut terminal = self.terminal.lock().unwrap();

        //TODO: set size?

        // Update if needed
        if terminal.needs_update {
            terminal.update();
            terminal.needs_update = false;
        }

        // Calculate layout lines
        terminal.with_buffer(|buffer| {
            let mut layout_lines = 0;
            for line in &buffer.lines {
                if let Some(layout) = line.layout_opt() {
                    layout_lines += layout.len()
                }
            }

            let height = layout_lines as f32 * buffer.metrics().line_height;
            let size = Size::new(limits.max().width, height);

            layout::Node::new(limits.resolve(Length::Fill, Length::Fill, size))
        })
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.focusable(state, self.id.as_ref());
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if let Some(Dragging::Scrollbar { .. }) = &state.dragging {
            return mouse::Interaction::Idle;
        }

        if let Some(p) = cursor_position.position_in(layout.bounds()) {
            let terminal = self.terminal.lock().unwrap();
            let buffer_size = terminal.with_buffer(|buffer| buffer.size());

            let x = p.x - self.padding.left;
            let y = p.y - self.padding.top;
            if x >= 0.0
                && x < buffer_size.0.unwrap_or(0.0)
                && y >= 0.0
                && y < buffer_size.1.unwrap_or(0.0)
            {
                return mouse::Interaction::Text;
            }
        }

        mouse::Interaction::Idle
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        drawer::draw(
            self,
            tree,
            renderer,
            theme,
            _style,
            layout,
            cursor_position,
            viewport,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle<f32>,
    ) -> Status {
        event_handler::handle_event(
            self,
            tree,
            event,
            layout,
            cursor_position,
            _renderer,
            _clipboard,
            shell,
            _viewport,
        )
    }
}

// fn draw_do<Message>(arg: &TerminalBox<'a, Message>, tree: &tree::Tree, renderer: &mut cosmic::iced::Renderer, theme: &Theme, _style: Style, layout: Layout<'_>, cursor_position: mouse::Cursor, viewport: Rectangle<f32>) -> _ where Message: Clone {
//     todo!()
// }

impl<'a, Message> From<TerminalBox<'a, Message>> for Element<'a, Message, cosmic::Theme, Renderer>
where
    Message: Clone + 'a,
{
    fn from(terminal_box: TerminalBox<'a, Message>) -> Self {
        Self::new(terminal_box)
    }
}

pub(super) enum ClickKind {
    Single,
    Double,
    Triple,
}

pub(super) enum Dragging {
    Buffer,
    Scrollbar {
        start_y: f32,
        start_scroll: (f32, f32),
    },
}

pub struct State {
    pub(super) modifiers: Modifiers,
    pub(super) click: Option<(ClickKind, Instant)>,
    pub(super) dragging: Option<Dragging>,
    pub(super) is_focused: bool,
    pub(super) scroll_pixels: f32,
    pub(super) scrollbar_rect: Cell<Rectangle<f32>>,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        Self {
            modifiers: Modifiers::empty(),
            click: None,
            dragging: None,
            is_focused: false,
            scroll_pixels: 0.0,
            scrollbar_rect: Cell::new(Rectangle::default()),
        }
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }
}
