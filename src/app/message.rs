// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::action::Action;
use crate::app::context_page::ContextPage;
use crate::config::app_theme::AppTheme;
use crate::config::color_scheme::{ColorSchemeId, ColorSchemeKind};
use crate::config::config::Config;
use crate::config::profile::ProfileId;
use crate::dnd::DndDrop;
use alacritty_terminal::event::Event as TermEvent;
use cosmic::{
    iced::{
        keyboard::{Key, Modifiers},
        Point,
    },
    widget::{self, pane_grid, segmented_button},
};
use cosmic_files::dialog::{DialogMessage, DialogResult};
use tokio::sync::mpsc;

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    AppTheme(AppTheme),
    ColorSchemeCollapse,
    ColorSchemeDelete(ColorSchemeKind, ColorSchemeId),
    ColorSchemeExpand(ColorSchemeKind, Option<ColorSchemeId>),
    ColorSchemeExport(ColorSchemeKind, Option<ColorSchemeId>),
    ColorSchemeExportResult(ColorSchemeKind, Option<ColorSchemeId>, DialogResult),
    ColorSchemeImport(ColorSchemeKind),
    ColorSchemeImportResult(ColorSchemeKind, DialogResult),
    ColorSchemeRename(ColorSchemeKind, ColorSchemeId, String),
    ColorSchemeRenameSubmit,
    ColorSchemeTabActivate(widget::segmented_button::Entity),
    Config(Config),
    Copy(Option<segmented_button::Entity>),
    CopyOrSigint(Option<segmented_button::Entity>),
    CopyPrimary(Option<segmented_button::Entity>),
    DefaultBoldFontWeight(usize),
    DefaultDimFontWeight(usize),
    DefaultFont(usize),
    DefaultFontSize(usize),
    DefaultFontStretch(usize),
    DefaultFontWeight(usize),
    DefaultZoomStep(usize),
    DialogMessage(DialogMessage),
    Drop(Option<(pane_grid::Pane, segmented_button::Entity, DndDrop)>),
    Find(bool),
    FindNext,
    FindPrevious,
    FindSearchValueChanged(String),
    MiddleClick(pane_grid::Pane, Option<segmented_button::Entity>),
    FocusFollowMouse(bool),
    Key(Modifiers, Key),
    LaunchUrl(String),
    Modifiers(Modifiers),
    MouseEnter(pane_grid::Pane),
    Opacity(u8),
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneFocusAdjacent(pane_grid::Direction),
    PaneResized(pane_grid::ResizeEvent),
    PaneSplit(pane_grid::Axis),
    PaneToggleMaximized,
    Paste(Option<segmented_button::Entity>),
    PastePrimary(Option<segmented_button::Entity>),
    PasteValue(Option<segmented_button::Entity>, String),
    ProfileCollapse(ProfileId),
    ProfileCommand(ProfileId, String),
    ProfileDirectory(ProfileId, String),
    ProfileExpand(ProfileId),
    ProfileHold(ProfileId, bool),
    ProfileName(ProfileId, String),
    ProfileNew,
    ProfileOpen(ProfileId),
    ProfileRemove(ProfileId),
    ProfileSyntaxTheme(ProfileId, ColorSchemeKind, usize),
    ProfileTabTitle(ProfileId, String),
    SelectAll(Option<segmented_button::Entity>),
    ShowAdvancedFontSettings(bool),
    ShowHeaderBar(bool),
    SyntaxTheme(ColorSchemeKind, usize),
    SystemThemeChange,
    TabActivate(segmented_button::Entity),
    TabActivateJump(usize),
    TabClose(Option<segmented_button::Entity>),
    TabContextAction(segmented_button::Entity, Action),
    TabContextMenu(pane_grid::Pane, Option<Point>),
    TabNew,
    TabNewNoProfile,
    TabNext,
    TabPrev,
    TermEvent(pane_grid::Pane, segmented_button::Entity, TermEvent),
    TermEventTx(mpsc::Sender<(pane_grid::Pane, segmented_button::Entity, TermEvent)>),
    ToggleContextPage(ContextPage),
    UpdateDefaultProfile((bool, ProfileId)),
    UseBrightBold(bool),
    WindowClose,
    WindowNew,
    ZoomIn,
    ZoomOut,
    ZoomReset,
}
