pub mod cfg;
pub mod cli;
pub mod commands;
pub mod files;
pub mod helper;
pub mod project_toml;

// TODO other os support

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_GAME: &str = "valve";
