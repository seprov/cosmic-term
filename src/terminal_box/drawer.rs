use alacritty_terminal::{
    term::cell::Flags,
    vte::ansi::{CursorShape, NamedColor},
};

use cosmic::{
    cosmic_theme::palette::{blend::Compose, WithAlpha},
    iced::{
        advanced::graphics::text::Raw,
        mouse::{self},
        Color, Point, Rectangle, Size, Vector,
    },
    iced_core::{
        layout::Layout,
        renderer::{self, Quad, Renderer as _},
        text::Renderer as _,
        widget::tree,
        Border,
    },
    theme::Theme,
};
use cosmic_text::LayoutGlyph;
use indexmap::IndexSet;
use std::{array, cmp, time::Instant};

use crate::{
    terminal::Metadata,
    terminal_box::terminal_box::{Dragging, State},
};

use super::terminal_box::TerminalBox;

pub(super) fn draw<'a, Message>(
    terminal_box: &TerminalBox<'a, Message>,
    tree: &tree::Tree,
    renderer: &mut cosmic::iced::Renderer,
    theme: &Theme,
    _style: &renderer::Style,
    layout: Layout<'_>,
    cursor_position: mouse::Cursor,
    viewport: &Rectangle<f32>,
) {
    let instant = Instant::now();

    let state = tree.state.downcast_ref::<State>();

    let cosmic_theme = theme.cosmic();
    let scrollbar_w = f32::from(cosmic_theme.spacing.space_xxs);

    let view_position =
        layout.position() + [terminal_box.padding.left, terminal_box.padding.top].into();
    let view_w = cmp::min(viewport.width as i32, layout.bounds().width as i32)
        - terminal_box.padding.horizontal() as i32
        - scrollbar_w as i32;
    let view_h = cmp::min(viewport.height as i32, layout.bounds().height as i32)
        - terminal_box.padding.vertical() as i32;

    if view_w <= 0 || view_h <= 0 {
        // Zero sized image
        return;
    }

    let mut terminal = terminal_box.terminal.lock().unwrap();

    // Ensure terminal is the right size
    terminal.resize(view_w as u32, view_h as u32);

    // Update if needed
    if terminal.needs_update {
        terminal.update();
        terminal.needs_update = false;
    }

    // Render default background
    {
        let meta = &terminal.metadata_set[terminal.default_attrs().metadata];
        let background_color = shade(meta.bg, state.is_focused);

        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: terminal_box.border,
                ..Default::default()
            },
            Color::new(
                f32::from(background_color.r()) / 255.0,
                f32::from(background_color.g()) / 255.0,
                f32::from(background_color.b()) / 255.0,
                match terminal_box.opacity {
                    Some(opacity) => opacity,
                    None => f32::from(background_color.a()) / 255.0,
                },
            ),
        );
    }

    // Render cell backgrounds that do not match default
    terminal.with_buffer(|buffer| {
        for run in buffer.layout_runs() {
            struct BgRect<'a> {
                default_metadata: usize,
                metadata: usize,
                glyph_font_size: f32,
                start_x: f32,
                end_x: f32,
                line_height: f32,
                line_top: f32,
                view_position: Point,
                metadata_set: &'a IndexSet<Metadata>,
            }

            impl<'a> BgRect<'a> {
                fn update<Renderer: renderer::Renderer>(
                    &mut self,
                    glyph: &LayoutGlyph,
                    renderer: &mut Renderer,
                    is_focused: bool,
                ) {
                    if glyph.metadata == self.metadata {
                        self.end_x = glyph.x + glyph.w;
                    } else {
                        self.fill(renderer, is_focused);
                        self.metadata = glyph.metadata;
                        self.glyph_font_size = glyph.font_size;
                        self.start_x = glyph.x;
                        self.end_x = glyph.x + glyph.w;
                    }
                }

                fn fill<Renderer: renderer::Renderer>(
                    &mut self,
                    renderer: &mut Renderer,
                    is_focused: bool,
                ) {
                    let cosmic_text_to_iced_color = |color: cosmic_text::Color| {
                        Color::new(
                            f32::from(color.r()) / 255.0,
                            f32::from(color.g()) / 255.0,
                            f32::from(color.b()) / 255.0,
                            f32::from(color.a()) / 255.0,
                        )
                    };

                    macro_rules! mk_pos_offset {
                        ($x_offset:expr, $bottom_offset:expr) => {
                            Vector::new(
                                self.start_x + $x_offset,
                                self.line_top + self.line_height - $bottom_offset,
                            )
                        };
                    }

                    macro_rules! mk_quad {
                        ($pos_offset:expr, $style_line_height:expr, $width:expr) => {
                            Quad {
                                bounds: Rectangle::new(
                                    self.view_position + $pos_offset,
                                    Size::new($width, $style_line_height),
                                ),
                                ..Default::default()
                            }
                        };
                        ($pos_offset:expr, $style_line_height:expr) => {
                            mk_quad!($pos_offset, $style_line_height, self.end_x - self.start_x)
                        };
                    }

                    let metadata = &self.metadata_set[self.metadata];
                    if metadata.bg != self.metadata_set[self.default_metadata].bg {
                        let color = shade(metadata.bg, is_focused);
                        renderer.fill_quad(
                            mk_quad!(mk_pos_offset!(0.0, self.line_height), self.line_height),
                            cosmic_text_to_iced_color(color),
                        );
                    }

                    if !metadata.flags.is_empty() {
                        let style_line_height = (self.glyph_font_size / 10.0).clamp(2.0, 16.0);

                        let line_color = cosmic_text_to_iced_color(metadata.underline_color);

                        if metadata.flags.contains(Flags::STRIKEOUT) {
                            let bottom_offset = (self.line_height - style_line_height) / 2.0;
                            let pos_offset = mk_pos_offset!(0.0, bottom_offset);
                            let underline_quad = mk_quad!(pos_offset, style_line_height);
                            renderer.fill_quad(underline_quad, line_color);
                        }

                        if metadata.flags.contains(Flags::UNDERLINE) {
                            let bottom_offset = style_line_height * 2.0;
                            let pos_offset = mk_pos_offset!(0.0, bottom_offset);
                            let underline_quad = mk_quad!(pos_offset, style_line_height);
                            renderer.fill_quad(underline_quad, line_color);
                        }

                        if metadata.flags.contains(Flags::DOUBLE_UNDERLINE) {
                            let style_line_height = style_line_height / 2.0;
                            let gap = style_line_height.max(1.5);
                            let bottom_offset = (style_line_height + gap) * 2.0;

                            let pos_offset1 = mk_pos_offset!(0.0, bottom_offset);
                            let underline1_quad = mk_quad!(pos_offset1, style_line_height);

                            let pos_offset2 = mk_pos_offset!(0.0, bottom_offset / 2.0);
                            let underline2_quad = mk_quad!(pos_offset2, style_line_height);

                            renderer.fill_quad(underline1_quad, line_color);
                            renderer.fill_quad(underline2_quad, line_color);
                        }

                        // rects is a slice of (width, Option<y>), `None` means a gap.
                        let mut draw_repeated = |rects: &[(f32, Option<f32>)]| {
                            let full_width = self.end_x - self.start_x;
                            let pattern_len: f32 = rects.iter().map(|x| x.0).sum(); // total length of the pattern
                            let mut accu_width = 0.0;
                            let mut index = {
                                let in_pattern = self.start_x % pattern_len;

                                let mut sum = 0.0;
                                let mut index = 0;
                                for (i, rect) in rects.iter().enumerate() {
                                    sum += rect.0;
                                    if in_pattern < sum {
                                        let width = sum - in_pattern;
                                        if let Some(height) = rect.1 {
                                            // draw first rect cropped to span
                                            let pos_offset = mk_pos_offset!(accu_width, height);
                                            let underline_quad =
                                                mk_quad!(pos_offset, style_line_height, width);
                                            renderer.fill_quad(underline_quad, line_color);
                                        }
                                        index = i + 1;
                                        accu_width += width;
                                        break;
                                    }
                                }
                                index // index of first full rect
                            };
                            while accu_width < full_width {
                                let (width, x) = rects[index % rects.len()];
                                let cropped_width = width.min(full_width - accu_width);
                                if let Some(height) = x {
                                    let pos_offset = mk_pos_offset!(accu_width, height);
                                    let underline_quad =
                                        mk_quad!(pos_offset, style_line_height, cropped_width);
                                    renderer.fill_quad(underline_quad, line_color);
                                }
                                accu_width += cropped_width;
                                index += 1;
                            }
                        };

                        if metadata.flags.contains(Flags::DOTTED_UNDERLINE) {
                            let bottom_offset = style_line_height * 2.0;
                            let dot = (2.0, Some(bottom_offset));
                            let gap = (2.0, None);
                            draw_repeated(&[dot, gap]);
                        }

                        if metadata.flags.contains(Flags::DASHED_UNDERLINE) {
                            let bottom_offset = style_line_height * 2.0;
                            let dash = (6.0, Some(bottom_offset));
                            let gap = (3.0, None);
                            draw_repeated(&[dash, gap]);
                        }

                        if metadata.flags.contains(Flags::UNDERCURL) {
                            let style_line_height = style_line_height.floor();
                            let bottom_offset = style_line_height * 1.5;
                            let pattern: [(f32, Option<f32>); 8] = array::from_fn(|i| match i {
                                3..=5 => (1.0, Some(bottom_offset + style_line_height)),
                                2 | 6 => (1.0, Some(bottom_offset + 2.0 * style_line_height / 3.0)),
                                1 | 7 => (1.0, Some(bottom_offset + 1.0 * style_line_height / 3.0)),
                                0 => (1.0, Some(bottom_offset)),
                                _ => unreachable!(),
                            });
                            draw_repeated(&pattern)
                        }
                    }
                }
            }

            let default_metadata = terminal.default_attrs().metadata;
            let metadata_set = &terminal.metadata_set;
            let mut bg_rect = BgRect {
                default_metadata,
                metadata: default_metadata,
                glyph_font_size: 0.0,
                start_x: 0.0,
                end_x: 0.0,
                line_height: buffer.metrics().line_height,
                line_top: run.line_top,
                view_position,
                metadata_set,
            };
            for glyph in run.glyphs {
                bg_rect.update(glyph, renderer, state.is_focused);
            }
            bg_rect.fill(renderer, state.is_focused);
        }
    });

    renderer.fill_raw(Raw {
        buffer: terminal.buffer_weak(),
        position: view_position,
        color: Color::new(1.0, 1.0, 1.0, 1.0), // TODO
        clip_bounds: Rectangle::new(view_position, Size::new(view_w as f32, view_h as f32)),
    });

    // Draw scrollbar
    if let Some((start, end)) = terminal.scrollbar() {
        let scrollbar_y = start * view_h as f32;
        let scrollbar_h = end * view_h as f32 - scrollbar_y;
        let scrollbar_rect = Rectangle::new(
            [view_w as f32, scrollbar_y].into(),
            Size::new(scrollbar_w, scrollbar_h),
        );

        let pressed = matches!(&state.dragging, Some(Dragging::Scrollbar { .. }));

        let mut hover = false;
        if let Some(p) = cursor_position.position_in(layout.bounds()) {
            let x = p.x - terminal_box.padding.left;
            if x >= scrollbar_rect.x && x < (scrollbar_rect.x + scrollbar_rect.width) {
                hover = true;
            }
        }

        let mut scrollbar_draw = scrollbar_rect + Vector::new(view_position.x, view_position.y);
        if !hover && !pressed {
            // Decrease draw width and keep centered when not hovered or pressed
            scrollbar_draw.width /= 2.0;
            scrollbar_draw.x += scrollbar_draw.width / 2.0;
        }

        // neutral_6, 0.7
        let base_color = cosmic_theme
            .palette
            .neutral_6
            .without_alpha()
            .with_alpha(0.7);
        let scrollbar_color: Color = if pressed {
            // pressed_state_color, 0.5
            cosmic_theme
                .background
                .component
                .pressed
                .without_alpha()
                .with_alpha(0.5)
                .over(base_color)
                .into()
        } else if hover {
            // hover_state_color, 0.2
            cosmic_theme
                .background
                .component
                .hover
                .without_alpha()
                .with_alpha(0.2)
                .over(base_color)
                .into()
        } else {
            base_color.into()
        };

        renderer.fill_quad(
            Quad {
                bounds: scrollbar_draw,
                border: Border {
                    radius: (scrollbar_draw.width / 2.0).into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            },
            scrollbar_color,
        );

        state.scrollbar_rect.set(scrollbar_rect);
    } else {
        state.scrollbar_rect.set(Rectangle::default())
    }

    // Draw cursor
    {
        let cursor = terminal.term.lock().renderable_content().cursor;
        let col = cursor.point.column.0;
        let line = cursor.point.line.0;
        let color = terminal.term.lock().colors()[NamedColor::Cursor]
            .or(terminal.colors()[NamedColor::Cursor])
            .map(|rgb| Color::from_rgb8(rgb.r, rgb.g, rgb.b))
            .unwrap_or(Color::WHITE); // TODO default color from theme?
        let width = terminal.size().cell_width;
        let height = terminal.size().cell_height;
        let top_left = view_position
            + Vector::new((col as f32 * width).floor(), (line as f32 * height).floor());
        match cursor.shape {
            CursorShape::Beam => {
                let quad = Quad {
                    bounds: Rectangle::new(top_left, Size::new(1.0, height)),
                    ..Default::default()
                };
                renderer.fill_quad(quad, color);
            }
            CursorShape::Underline => {
                let quad = Quad {
                    bounds: Rectangle::new(
                        view_position
                            + Vector::new(
                                (col as f32 * width).floor(),
                                ((line + 1) as f32 * height).floor(),
                            ),
                        Size::new(width, 1.0),
                    ),
                    ..Default::default()
                };
                renderer.fill_quad(quad, color);
            }
            CursorShape::HollowBlock => {} // TODO not sure when this would even be activated
            CursorShape::Block | CursorShape::Hidden => {} // Block is handled seperately
        }
    }

    let duration = instant.elapsed();
    log::trace!("redraw {}, {}: {:?}", view_w, view_h, duration);
}

fn shade(color: cosmic_text::Color, is_focused: bool) -> cosmic_text::Color {
    if is_focused {
        color
    } else {
        let shade = 0.92;
        cosmic_text::Color::rgba(
            (f32::from(color.r()) * shade) as u8,
            (f32::from(color.g()) * shade) as u8,
            (f32::from(color.b()) * shade) as u8,
            color.a(),
        )
    }
}
