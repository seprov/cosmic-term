use serde::{Deserialize, Serialize};

use crate::fl;

use super::constants::{COSMIC_THEME_DARK, COSMIC_THEME_LIGHT};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProfileId(pub u64);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub syntax_theme_dark: String,
    #[serde(default)]
    pub syntax_theme_light: String,
    #[serde(default)]
    pub tab_title: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default)]
    pub hold: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: fl!("new-profile"),
            command: String::new(),
            syntax_theme_dark: COSMIC_THEME_DARK.to_string(),
            syntax_theme_light: COSMIC_THEME_LIGHT.to_string(),
            tab_title: String::new(),
            working_directory: String::new(),
            hold: false,
        }
    }
}
