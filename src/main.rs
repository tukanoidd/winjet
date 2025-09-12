mod app;
mod controller;
mod util;

use color_eyre::{Result, eyre::OptionExt};
use directories::ProjectDirs;
use iced_fonts::NERD_FONT_BYTES;
use tracing_subscriber::prelude::*;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    init_tracing()?;

    iced::application(
        || {
            let dirs = ProjectDirs::from("com", "tukanoid", "winjet")
                .ok_or_eyre("Failed to initialize project directories")
                .inspect_err(|err| tracing::error!("{err}"))
                .unwrap();

            App::new(dirs)
        },
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .theme(App::theme)
    .font(NERD_FONT_BYTES)
    .run()?;

    Ok(())
}

fn init_tracing() -> Result<()> {
    let level = match cfg!(debug_assertions) {
        true => "debug",
        false => "info",
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::EnvFilter::new(level))
        .try_init()?;

    tracing::debug!("Logging Initialized!");

    Ok(())
}
