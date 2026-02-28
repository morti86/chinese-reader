#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod gui;
mod config;
mod utils;
mod cedict;
mod textbase;
mod ocr;
mod error;
mod ai;
mod scraper;
mod anki;


use tracing::Level;
pub use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt,
    prelude::*,
};
use tokio::sync::OnceCell;
use rig::{agent::Agent, client::completion::CompletionModelHandle};

static AGENT: OnceCell<Agent<CompletionModelHandle<'static>>> = OnceCell::const_new();

const FONT: &'static [u8] = include_bytes!("../SymbolsNerdFont-Regular.ttf");

pub fn run(theme: &str) -> Result<(), iced::Error> {
    for e in iced::Theme::ALL {

        debug!("Run with theme {}", theme);
    
        if theme.to_string() == e.to_string() {
            return iced::application(gui::App::new, gui::App::update, gui::App::view)
                .theme(gui::App::theme)
                .window_size(iced::Size::new(1400.0,800.0))
                .subscription(gui::App::subscription)
                .font(FONT)
                .run();
        }
    }
    iced::application(gui::App::default, gui::App::update, gui::App::view)
        .subscription(gui::App::subscription)
        .font(FONT)
        .run()
}

fn main() {
    #[cfg(debug_assertions)]
    let env_rust_log = Some(Level::DEBUG);
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .compact()
                .with_filter(LevelFilter::from_level(env_rust_log.unwrap_or(Level::INFO))),
        )
        .with(
            fmt::layer().with_writer(std::io::stdout)
        )
        .with(
            Targets::default()
            .with_target("cnreader", env_rust_log)
            .with_target("iced", Level::WARN)
            .with_target("rig_core", Level::INFO)
            .with_target("stardict", Level::INFO)
        )
        .init();

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();   
    let _ = run("Tokyo Night");
}
