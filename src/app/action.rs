// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::context_page::ContextPage;
use super::message::Message;
use crate::config::{color_scheme::ColorSchemeKind, profile::ProfileId};
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::pane_grid;
use cosmic::widget::segmented_button;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    ColorSchemes(ColorSchemeKind),
    Copy,
    CopyOrSigint,
    // CopyPrimary,
    Find,
    PaneFocusDown,
    PaneFocusLeft,
    PaneFocusRight,
    PaneFocusUp,
    PaneSplitHorizontal,
    PaneSplitVertical,
    PaneToggleMaximized,
    Paste,
    PastePrimary,
    ProfileOpen(ProfileId),
    Profiles,
    SelectAll,
    Settings,
    ShowHeaderBar(bool),
    TabActivate0,
    TabActivate1,
    TabActivate2,
    TabActivate3,
    TabActivate4,
    TabActivate5,
    TabActivate6,
    TabActivate7,
    TabActivate8,
    TabClose,
    TabNew,
    // TabNewNoProfile,
    TabNext,
    TabPrev,
    WindowClose,
    WindowNew,
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

impl Action {
    pub(super) fn message(&self, entity_opt: Option<segmented_button::Entity>) -> Message {
        match self {
            Self::About => Message::ToggleContextPage(ContextPage::About),
            Self::ColorSchemes(color_scheme_kind) => {
                Message::ToggleContextPage(ContextPage::ColorSchemes(*color_scheme_kind))
            }
            Self::Copy => Message::Copy(entity_opt),
            Self::CopyOrSigint => Message::CopyOrSigint(entity_opt),
            // Self::CopyPrimary => Message::CopyPrimary(entity_opt),
            Self::Find => Message::Find(true),
            Self::PaneFocusDown => Message::PaneFocusAdjacent(pane_grid::Direction::Down),
            Self::PaneFocusLeft => Message::PaneFocusAdjacent(pane_grid::Direction::Left),
            Self::PaneFocusRight => Message::PaneFocusAdjacent(pane_grid::Direction::Right),
            Self::PaneFocusUp => Message::PaneFocusAdjacent(pane_grid::Direction::Up),
            Self::PaneSplitHorizontal => Message::PaneSplit(pane_grid::Axis::Horizontal),
            Self::PaneSplitVertical => Message::PaneSplit(pane_grid::Axis::Vertical),
            Self::PaneToggleMaximized => Message::PaneToggleMaximized,
            Self::Paste => Message::Paste(entity_opt),
            Self::PastePrimary => Message::PastePrimary(entity_opt),
            Self::ProfileOpen(profile_id) => Message::ProfileOpen(*profile_id),
            Self::Profiles => Message::ToggleContextPage(ContextPage::Profiles),
            Self::SelectAll => Message::SelectAll(entity_opt),
            Self::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Self::ShowHeaderBar(show_headerbar) => Message::ShowHeaderBar(*show_headerbar),
            Self::TabActivate0 => Message::TabActivateJump(0),
            Self::TabActivate1 => Message::TabActivateJump(1),
            Self::TabActivate2 => Message::TabActivateJump(2),
            Self::TabActivate3 => Message::TabActivateJump(3),
            Self::TabActivate4 => Message::TabActivateJump(4),
            Self::TabActivate5 => Message::TabActivateJump(5),
            Self::TabActivate6 => Message::TabActivateJump(6),
            Self::TabActivate7 => Message::TabActivateJump(7),
            Self::TabActivate8 => Message::TabActivateJump(8),
            Self::TabClose => Message::TabClose(entity_opt),
            Self::TabNew => Message::TabNew,
            // Self::TabNewNoProfile => Message::TabNewNoProfile,
            Self::TabNext => Message::TabNext,
            Self::TabPrev => Message::TabPrev,
            Self::WindowClose => Message::WindowClose,
            Self::WindowNew => Message::WindowNew,
            Self::ZoomIn => Message::ZoomIn,
            Self::ZoomOut => Message::ZoomOut,
            Self::ZoomReset => Message::ZoomReset,
        }
    }
}

impl MenuAction for Action {
    type Message = Message;

    fn message(&self) -> Message {
        self.message(None)
    }
}
