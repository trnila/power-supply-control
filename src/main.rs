#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod components;
pub mod config;
pub mod line_codec;
pub mod mx100qp;

use dioxus::{
    desktop::{Config, WindowBuilder},
    prelude::*,
};
use dioxus_desktop::LogicalSize;
use log::{info, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use std::path::PathBuf;
use tracing::Level;

use crate::components::app::AppComponent;

fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("power-supply-control")
}

fn main() {
    std::fs::create_dir_all(get_config_dir()).unwrap();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l}: {m}\n")))
        .build(get_config_dir().join("output.log"))
        .unwrap();

    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l}: {m}\n")))
        .target(Target::Stderr)
        .build();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("console")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
    log_panics::init();
    dioxus_logger::init(Level::INFO).unwrap();

    info!(
        "Power supply version {} started",
        env!("VERGEN_GIT_DESCRIBE")
    );

    LaunchBuilder::new()
        .with_cfg(
            Config::default()
                .with_window(
                    WindowBuilder::new()
                        .with_maximized(false)
                        .with_title("Power supply control")
                        .with_min_inner_size(LogicalSize::new(1280, 768)),
                )
                .with_menu(None)
                .with_disable_context_menu(true),
        )
        .launch(AppComponent)
}
