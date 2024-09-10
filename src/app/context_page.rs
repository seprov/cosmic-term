use crate::{config::color_scheme::ColorSchemeKind, fl};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    ColorSchemes(ColorSchemeKind),
    Profiles,
    Settings,
}

impl ContextPage {
    pub(super) fn title(&self) -> String {
        match self {
            Self::About => String::new(),
            Self::ColorSchemes(_color_scheme_kind) => fl!("color-schemes"),
            Self::Profiles => fl!("profiles"),
            Self::Settings => fl!("settings"),
        }
    }
}
