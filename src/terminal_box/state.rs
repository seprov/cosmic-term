use cosmic::{
    iced::{keyboard::Modifiers, Rectangle},
    iced_core::widget::operation::{self},
};

use std::{cell::Cell, time::Instant};

use super::enums::{ClickKind, Dragging};

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
