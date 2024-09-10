// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::terminal::terminal::{Terminal, TerminalScroll};

use alacritty_terminal::{term, tty};
use app::app::App;
use app::flags::Flags;
use config::color_scheme::{ColorSchemeId, ColorSchemeKind};
use config::config::Config;
use config::constants::CONFIG_VERSION;
use cosmic::{
    app::Settings,
    cosmic_config::{self, CosmicConfigEntry},
    iced::Limits,
    widget::{self},
    Application,
};
use icon_cache::IconCache;
use std::{env, process, sync::Mutex};

mod app;
mod config;
mod dnd;
mod icon_cache;
mod key_bind;
mod localization;
mod menu;
mod mouse_reporter;
mod terminal;
mod terminal_box;
mod terminal_theme;

lazy_static::lazy_static! {
    static ref ICON_CACHE: Mutex<IconCache> = Mutex::new(IconCache::new());
}

pub fn icon_cache_get(name: &'static str, size: u16) -> widget::icon::Icon {
    let mut icon_cache = ICON_CACHE.lock().unwrap();
    icon_cache.get(name, size)
}

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell_program_opt = None;
    let mut shell_args = Vec::new();
    let mut parse_flags = true;
    let mut daemonize = true;
    for arg in env::args().skip(1) {
        if parse_flags {
            match arg.as_str() {
                // These flags indicate the end of parsing flags
                "-e" | "--command" | "--" => {
                    parse_flags = false;
                }
                "--no-daemon" => {
                    daemonize = false;
                }
                _ => {
                    //TODO: should this throw an error?
                    log::warn!("ignored argument {:?}", arg);
                }
            }
        } else if shell_program_opt.is_none() {
            shell_program_opt = Some(arg);
        } else {
            shell_args.push(arg);
        }
    }

    #[cfg(all(unix, not(target_os = "redox")))]
    if daemonize {
        match fork::daemon(true, true) {
            Ok(fork::Fork::Child) => (),
            Ok(fork::Fork::Parent(_child_pid)) => process::exit(0),
            Err(err) => {
                eprintln!("failed to daemonize: {:?}", err);
                process::exit(1);
            }
        }
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localization::localize();

    let (config_handler, config) = match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    log::info!("errors loading config: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            log::error!("failed to create config handler: {}", err);
            (None, Config::default())
        }
    };

    let startup_options = if let Some(shell_program) = shell_program_opt {
        let options = tty::Options {
            shell: Some(tty::Shell::new(shell_program, shell_args)),
            ..tty::Options::default()
        };
        Some(options)
    } else {
        None
    };

    let term_config = term::Config::default();
    // Set up environmental variables for terminal
    tty::setup_env();
    // Override TERM for better compatibility
    env::set_var("TERM", "xterm-256color");

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.map_to_cosmic_theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));

    let flags = Flags {
        config_handler,
        config,
        startup_options,
        term_config,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}
