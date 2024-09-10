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
