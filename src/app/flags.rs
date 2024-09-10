use alacritty_terminal::{term, tty};
use cosmic::cosmic_config;

use crate::config::config::Config;

#[derive(Clone, Debug)]
pub struct Flags {
    pub(crate) config_handler: Option<cosmic_config::Config>,
    pub(crate) config: Config,
    pub(crate) startup_options: Option<tty::Options>,
    pub(crate) term_config: term::Config,
}
