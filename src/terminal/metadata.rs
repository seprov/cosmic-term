use alacritty_terminal::term::cell::Flags;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Metadata {
    pub bg: cosmic_text::Color,
    pub underline_color: cosmic_text::Color,
    pub flags: Flags,
}

impl Metadata {
    pub(super) fn new(bg: cosmic_text::Color, underline_color: cosmic_text::Color) -> Self {
        let flags = Flags::empty();
        Self {
            bg,
            underline_color,
            flags,
        }
    }

    pub(super) fn with_underline_color(self, underline_color: cosmic_text::Color) -> Self {
        Self {
            underline_color,
            ..self
        }
    }

    pub(super) fn with_flags(self, flags: Flags) -> Self {
        Self { flags, ..self }
    }
}
