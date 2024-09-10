use hex_color::HexColor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ColorSchemeKind {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ColorSchemeId(pub u64);

//TODO: there is a lot of extra code to keep the exported color scheme clean,
//consider how to reduce this
fn de_color_opt<'de, D>(deserializer: D) -> Result<Option<HexColor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hex_color: HexColor = Deserialize::deserialize(deserializer)?;
    Ok(Some(hex_color))
}

fn ser_color_opt<S>(hex_color_opt: &Option<HexColor>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::Error as _;
    match hex_color_opt {
        Some(hex_color) => Serialize::serialize(hex_color, serializer),
        None => Err(S::Error::custom("ser_color_opt called with None")),
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorSchemeAnsi {
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub black: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub red: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub green: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub yellow: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub blue: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub magenta: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub cyan: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub white: Option<HexColor>,
}

impl ColorSchemeAnsi {
    pub fn is_empty(&self) -> bool {
        self.black.is_none()
            && self.red.is_none()
            && self.green.is_none()
            && self.yellow.is_none()
            && self.blue.is_none()
            && self.magenta.is_none()
            && self.cyan.is_none()
            && self.white.is_none()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorScheme {
    pub name: String,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub foreground: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub background: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub cursor: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub bright_foreground: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub dim_foreground: Option<HexColor>,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub normal: ColorSchemeAnsi,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub bright: ColorSchemeAnsi,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub dim: ColorSchemeAnsi,
}
