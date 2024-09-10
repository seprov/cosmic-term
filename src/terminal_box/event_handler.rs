// SPDX-License-Identifier: GPL-3.0-only

use alacritty_terminal::{
    index::{Column as TermColumn, Point as TermPoint, Side as TermSide},
    selection::{Selection, SelectionType},
    term::TermMode,
};

use cosmic::{
    iced::{
        event::{Event, Status},
        keyboard::{Event as KeyEvent, Key},
        mouse::{self, Button, Event as MouseEvent, ScrollDelta},
        Point, Rectangle,
    },
    iced_core::{clipboard::Clipboard, keyboard::key::Named, layout::Layout, widget::tree, Shell},
};

use std::time::Instant;

use crate::TerminalScroll;

use super::terminal_box::{ClickKind, Dragging, State, TerminalBox};

pub(super) fn handle_event<'a, Message>(
    terminal_box: &mut TerminalBox<'a, Message>,
    tree: &mut tree::Tree,
    event: Event,
    layout: Layout<'_>,
    cursor_position: mouse::Cursor,
    _renderer: &cosmic::iced::Renderer,
    _clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    _viewport: &Rectangle<f32>,
) -> Status
where
    Message: Clone,
{
    let state = tree.state.downcast_mut::<State>();
    let scrollbar_rect = state.scrollbar_rect.get();
    let mut terminal = terminal_box.terminal.lock().unwrap();
    let buffer_size = terminal.with_buffer(|buffer| buffer.size());

    let is_app_cursor = terminal.term.lock().mode().contains(TermMode::APP_CURSOR);
    let is_mouse_mode = terminal.term.lock().mode().intersects(TermMode::MOUSE_MODE);
    let mut status = Status::Ignored;
    match event {
        Event::Keyboard(KeyEvent::KeyPressed {
            key: Key::Named(named),
            modifiers,
            ..
        }) if state.is_focused => {
            for key_bind in terminal_box.key_binds.keys() {
                if key_bind.matches(modifiers, &Key::Named(named)) {
                    return Status::Captured;
                }
            }

            let mod_no = calculate_modifier_number(state);
            let escape_code = match named {
                Named::Insert => csi("2", "~", mod_no),
                Named::Delete => csi("3", "~", mod_no),
                Named::PageUp => {
                    if modifiers.shift() {
                        terminal.scroll(TerminalScroll::PageUp);
                        None
                    } else {
                        csi("5", "~", mod_no)
                    }
                }
                Named::PageDown => {
                    if modifiers.shift() {
                        terminal.scroll(TerminalScroll::PageDown);
                        None
                    } else {
                        csi("6", "~", mod_no)
                    }
                }
                Named::ArrowUp => {
                    if is_app_cursor {
                        ss3("A", mod_no)
                    } else {
                        csi2("A", mod_no)
                    }
                }
                Named::ArrowDown => {
                    if is_app_cursor {
                        ss3("B", mod_no)
                    } else {
                        csi2("B", mod_no)
                    }
                }
                Named::ArrowRight => {
                    if is_app_cursor {
                        ss3("C", mod_no)
                    } else {
                        csi2("C", mod_no)
                    }
                }
                Named::ArrowLeft => {
                    if is_app_cursor {
                        ss3("D", mod_no)
                    } else {
                        csi2("D", mod_no)
                    }
                }
                Named::End => {
                    if modifiers.shift() {
                        terminal.scroll(TerminalScroll::Bottom);
                        None
                    } else if is_app_cursor {
                        ss3("F", mod_no)
                    } else {
                        csi2("F", mod_no)
                    }
                }
                Named::Home => {
                    if modifiers.shift() {
                        terminal.scroll(TerminalScroll::Top);
                        None
                    } else if is_app_cursor {
                        ss3("H", mod_no)
                    } else {
                        csi2("H", mod_no)
                    }
                }
                Named::F1 => ss3("P", mod_no),
                Named::F2 => ss3("Q", mod_no),
                Named::F3 => ss3("R", mod_no),
                Named::F4 => ss3("S", mod_no),
                Named::F5 => csi("15", "~", mod_no),
                Named::F6 => csi("17", "~", mod_no),
                Named::F7 => csi("18", "~", mod_no),
                Named::F8 => csi("19", "~", mod_no),
                Named::F9 => csi("20", "~", mod_no),
                Named::F10 => csi("21", "~", mod_no),
                Named::F11 => csi("23", "~", mod_no),
                Named::F12 => csi("24", "~", mod_no),
                _ => None,
            };
            if let Some(escape_code) = escape_code {
                terminal.input_scroll(escape_code);
                return Status::Captured;
            }

            //Special handle Enter, Escape, Backspace and Tab as described in
            //https://sw.kovidgoyal.net/kitty/keyboard-protocol/#legacy-key-event-encoding
            //Also special handle Ctrl-_ to behave like xterm
            let alt_prefix = if modifiers.alt() { "\x1B" } else { "" };
            match named {
                Named::Backspace => {
                    let code = if modifiers.control() { "\x08" } else { "\x7f" };
                    terminal.input_scroll(format!("{alt_prefix}{code}").into_bytes());
                    status = Status::Captured;
                }
                Named::Enter => {
                    terminal.input_scroll(format!("{}{}", alt_prefix, "\x0D").into_bytes());
                    status = Status::Captured;
                }
                Named::Escape => {
                    //Escape with any modifier will cancel selection
                    let had_selection = {
                        let mut term = terminal.term.lock();
                        term.selection.take().is_some()
                    };
                    if had_selection {
                        terminal.update();
                    } else {
                        terminal.input_scroll(format!("{}{}", alt_prefix, "\x1B").into_bytes());
                    }
                    status = Status::Captured;
                }
                Named::Space => {
                    terminal.input_scroll(format!("{}{}", alt_prefix, " ").into_bytes());
                    status = Status::Captured;
                }
                Named::Tab => {
                    let code = if modifiers.shift() { "\x1b[Z" } else { "\x09" };
                    terminal.input_scroll(format!("{alt_prefix}{code}").into_bytes());
                    status = Status::Captured;
                }
                _ => {}
            }
        }
        Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
            state.modifiers = modifiers;
        }
        Event::Keyboard(KeyEvent::KeyPressed {
            text,
            modifiers,
            key,
            ..
        }) if state.is_focused => {
            for key_bind in terminal_box.key_binds.keys() {
                if key_bind.matches(modifiers, &key) {
                    return Status::Captured;
                }
            }
            let character = text.and_then(|c| c.chars().next()).unwrap_or_default();
            match (
                modifiers.logo(),
                modifiers.control(),
                modifiers.alt(),
                modifiers.shift(),
            ) {
                (true, _, _, _) => {
                    // Ignore super
                }
                (false, true, true, _) => {
                    // Handle ctrl-alt for non-control characters
                    // and control characters 0-32
                    if !character.is_control() || (character as u32) < 32 {
                        // Handle alt for non-control characters
                        let mut buf = [0x1B, 0, 0, 0, 0];
                        let len = {
                            let str = character.encode_utf8(&mut buf[1..]);
                            str.len() + 1
                        };
                        terminal.input_scroll(buf[..len].to_vec());
                        status = Status::Captured;
                    }
                }
                (false, true, _, false) => {
                    // Handle ctrl for control characters (Ctrl-A to Ctrl-Z)
                    if character.is_control() {
                        let mut buf = [0, 0, 0, 0];
                        let str = character.encode_utf8(&mut buf);
                        terminal.input_scroll(str.as_bytes().to_vec());
                        status = Status::Captured;
                    }
                }
                (false, true, _, true) => {
                    //This is normally Ctrl+Minus, but since that
                    //is taken by zoom, we send that code for
                    //Ctrl+Underline instead, like xterm and
                    //gnome-terminal
                    if key == Key::Character("_".into()) {
                        terminal.input_scroll(b"\x1F".as_slice());
                        status = Status::Captured;
                    }
                }
                (false, false, true, _) => {
                    if !character.is_control() {
                        // Handle alt for non-control characters
                        let mut buf = [0x1B, 0, 0, 0, 0];
                        let len = {
                            let str = character.encode_utf8(&mut buf[1..]);
                            str.len() + 1
                        };
                        terminal.input_scroll(buf[..len].to_vec());
                        status = Status::Captured;
                    }
                }
                (false, false, false, _) => {
                    // Handle no modifiers for non-control characters
                    if !character.is_control() {
                        let mut buf = [0, 0, 0, 0];
                        let str = character.encode_utf8(&mut buf);
                        terminal.input_scroll(str.as_bytes().to_vec());
                        status = Status::Captured;
                    }
                }
            }
        }
        Event::Mouse(MouseEvent::ButtonPressed(button)) => {
            if let Some(p) = cursor_position.position_in(layout.bounds()) {
                let x = p.x - terminal_box.padding.left;
                let y = p.y - terminal_box.padding.top;
                //TODO: better calculation of position
                let col = x / terminal.size().cell_width;
                let row = y / terminal.size().cell_height;

                if is_mouse_mode {
                    terminal.report_mouse(event, &state.modifiers, col as u32, row as u32);
                } else {
                    state.is_focused = true;

                    // Handle left click drag
                    #[allow(clippy::collapsible_if)]
                    if let Button::Left = button {
                        let x = p.x - terminal_box.padding.left;
                        let y = p.y - terminal_box.padding.top;
                        if x >= 0.0
                            && x < buffer_size.0.unwrap_or(0.0)
                            && y >= 0.0
                            && y < buffer_size.1.unwrap_or(0.0)
                        {
                            let click_kind =
                                if let Some((click_kind, click_time)) = state.click.take() {
                                    if click_time.elapsed() < terminal_box.click_timing {
                                        match click_kind {
                                            ClickKind::Single => ClickKind::Double,
                                            ClickKind::Double => ClickKind::Triple,
                                            ClickKind::Triple => ClickKind::Single,
                                        }
                                    } else {
                                        ClickKind::Single
                                    }
                                } else {
                                    ClickKind::Single
                                };
                            let location = terminal.viewport_to_point(TermPoint::new(
                                row as usize,
                                TermColumn(col as usize),
                            ));
                            let side = if col.fract() < 0.5 {
                                TermSide::Left
                            } else {
                                TermSide::Right
                            };
                            let selection = match click_kind {
                                ClickKind::Single => {
                                    Selection::new(SelectionType::Simple, location, side)
                                }
                                ClickKind::Double => {
                                    Selection::new(SelectionType::Semantic, location, side)
                                }
                                ClickKind::Triple => {
                                    Selection::new(SelectionType::Lines, location, side)
                                }
                            };
                            {
                                let mut term = terminal.term.lock();
                                term.selection = Some(selection);
                            }
                            terminal.needs_update = true;
                            state.click = Some((click_kind, Instant::now()));
                            state.dragging = Some(Dragging::Buffer);
                        } else if scrollbar_rect.contains(Point::new(x, y)) {
                            if let Some(start_scroll) = terminal.scrollbar() {
                                state.dragging = Some(Dragging::Scrollbar {
                                    start_y: y,
                                    start_scroll,
                                });
                            }
                        } else if x >= scrollbar_rect.x
                            && x < (scrollbar_rect.x + scrollbar_rect.width)
                        {
                            if terminal.scrollbar().is_some() {
                                let scroll_ratio = terminal
                                    .with_buffer(|buffer| y / buffer.size().1.unwrap_or(1.0));
                                terminal.scroll_to(scroll_ratio);
                                if let Some(start_scroll) = terminal.scrollbar() {
                                    state.dragging = Some(Dragging::Scrollbar {
                                        start_y: y,
                                        start_scroll,
                                    });
                                }
                            }
                        }
                    } else if button == Button::Middle {
                        if let Some(on_middle_click) = &terminal_box.on_middle_click {
                            shell.publish(on_middle_click());
                        }
                    }
                    // Update context menu state
                    if let Some(on_context_menu) = &terminal_box.on_context_menu {
                        shell.publish((on_context_menu)(match terminal_box.context_menu {
                            Some(_) => None,
                            None => match button {
                                Button::Right => Some(p),
                                _ => None,
                            },
                        }));
                    }
                    status = Status::Captured;
                }
            }
        }
        Event::Mouse(MouseEvent::ButtonReleased(Button::Left)) => {
            state.dragging = None;
            if let Some(p) = cursor_position.position_in(layout.bounds()) {
                let x = p.x - terminal_box.padding.left;
                let y = p.y - terminal_box.padding.top;
                //TODO: better calculation of position
                let col = x / terminal.size().cell_width;
                let row = y / terminal.size().cell_height;
                if is_mouse_mode {
                    terminal.report_mouse(event, &state.modifiers, col as u32, row as u32);
                } else {
                    status = Status::Captured;
                }
            } else {
                status = Status::Captured;
            }
        }
        Event::Mouse(MouseEvent::ButtonReleased(_button)) => {
            if let Some(p) = cursor_position.position_in(layout.bounds()) {
                let x = p.x - terminal_box.padding.left;
                let y = p.y - terminal_box.padding.top;
                //TODO: better calculation of position
                let col = x / terminal.size().cell_width;
                let row = y / terminal.size().cell_height;
                if is_mouse_mode {
                    terminal.report_mouse(event, &state.modifiers, col as u32, row as u32);
                }
            }
        }
        Event::Mouse(MouseEvent::CursorMoved { .. }) => {
            if let Some(on_mouse_enter) = &terminal_box.on_mouse_enter {
                let mouse_is_inside = cursor_position.position_in(layout.bounds()).is_some();
                if let Some(known_is_inside) = terminal_box.mouse_inside_boundary {
                    if mouse_is_inside != known_is_inside {
                        if mouse_is_inside {
                            shell.publish(on_mouse_enter());
                        }
                        terminal_box.mouse_inside_boundary = Some(mouse_is_inside);
                    }
                } else {
                    terminal_box.mouse_inside_boundary = Some(mouse_is_inside);
                }
            }
            if let Some(p) = cursor_position.position() {
                let x = (p.x - layout.bounds().x) - terminal_box.padding.left;
                let y = (p.y - layout.bounds().y) - terminal_box.padding.top;
                //TODO: better calculation of position
                let col = x / terminal.size().cell_width;
                let row = y / terminal.size().cell_height;
                if is_mouse_mode {
                    terminal.report_mouse(event, &state.modifiers, col as u32, row as u32);
                } else {
                    if let Some(dragging) = &state.dragging {
                        match dragging {
                            Dragging::Buffer => {
                                let location = terminal.viewport_to_point(TermPoint::new(
                                    row as usize,
                                    TermColumn(col as usize),
                                ));
                                let side = if col.fract() < 0.5 {
                                    TermSide::Left
                                } else {
                                    TermSide::Right
                                };
                                {
                                    let mut term = terminal.term.lock();
                                    if let Some(selection) = &mut term.selection {
                                        selection.update(location, side);
                                    }
                                }
                                terminal.needs_update = true;
                            }
                            Dragging::Scrollbar {
                                start_y,
                                start_scroll,
                            } => {
                                let scroll_offset = terminal.with_buffer(|buffer| {
                                    (y - start_y) / buffer.size().1.unwrap_or(1.0)
                                });
                                terminal.scroll_to(start_scroll.0 + scroll_offset);
                            }
                        }
                    }
                    status = Status::Captured;
                }
            }
        }
        Event::Mouse(MouseEvent::WheelScrolled { delta }) => {
            if let Some(p) = cursor_position.position_in(layout.bounds()) {
                if is_mouse_mode {
                    let x = (p.x - layout.bounds().x) - terminal_box.padding.left;
                    let y = (p.y - layout.bounds().y) - terminal_box.padding.top;
                    //TODO: better calculation of position
                    let col = x / terminal.size().cell_width;
                    let row = y / terminal.size().cell_height;
                    terminal.scroll_mouse(delta, &state.modifiers, col as u32, row as u32);
                } else {
                    match delta {
                        ScrollDelta::Lines { x: _, y } => {
                            //TODO: this adjustment is just a guess!
                            state.scroll_pixels = 0.0;
                            let lines = (-y * 6.0) as i32;
                            if lines != 0 {
                                terminal.scroll(TerminalScroll::Delta(-lines));
                            }
                            status = Status::Captured;
                        }
                        ScrollDelta::Pixels { x: _, y } => {
                            //TODO: this adjustment is just a guess!
                            state.scroll_pixels -= y * 6.0;
                            let mut lines = 0;
                            let metrics = terminal.with_buffer(|buffer| buffer.metrics());
                            while state.scroll_pixels <= -metrics.line_height {
                                lines -= 1;
                                state.scroll_pixels += metrics.line_height;
                            }
                            while state.scroll_pixels >= metrics.line_height {
                                lines += 1;
                                state.scroll_pixels -= metrics.line_height;
                            }
                            if lines != 0 {
                                terminal.scroll(TerminalScroll::Delta(-lines));
                            }
                            status = Status::Captured;
                        }
                    }
                }
            }
        }
        _ => (),
    }

    status
}

/*
 shift     0b1         (1)
alt       0b10        (2)
ctrl      0b100       (4)
super     0b1000      (8)
hyper     0b10000     (16)
meta      0b100000    (32)
caps_lock 0b1000000   (64)
num_lock  0b10000000  (128)
*/
fn calculate_modifier_number(state: &State) -> u8 {
    let mut mod_no = 0;
    if state.modifiers.shift() {
        mod_no |= 1;
    }
    if state.modifiers.alt() {
        mod_no |= 2;
    }
    if state.modifiers.control() {
        mod_no |= 4;
    }
    if state.modifiers.logo() {
        mod_no |= 8;
    }
    mod_no + 1
}

#[inline(always)]
fn csi(code: &str, suffix: &str, modifiers: u8) -> Option<Vec<u8>> {
    if modifiers == 1 {
        Some(format!("\x1B[{code}{suffix}").into_bytes())
    } else {
        Some(format!("\x1B[{code};{modifiers}{suffix}").into_bytes())
    }
}

// https://sw.kovidgoyal.net/kitty/keyboard-protocol/#legacy-functional-keys
// CSI 1 ; modifier {ABCDEFHPQS}
// code is ABCDEFHPQS
#[inline(always)]
fn csi2(code: &str, modifiers: u8) -> Option<Vec<u8>> {
    if modifiers == 1 {
        Some(format!("\x1B[{code}").into_bytes())
    } else {
        Some(format!("\x1B[1;{modifiers}{code}").into_bytes())
    }
}

#[inline(always)]
fn ss3(code: &str, modifiers: u8) -> Option<Vec<u8>> {
    if modifiers == 1 {
        Some(format!("\x1B\x4F{code}").into_bytes())
    } else {
        Some(format!("\x1B[1;{modifiers}{code}").into_bytes())
    }
}
