pub mod cfg;
pub mod cli;
pub mod commands;
pub mod files;
pub(crate) mod helper;
pub mod project_toml;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_GAME: &str = "valve";
