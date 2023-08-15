//#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::nursery)]
//#![warn(clippy::cargo)]

pub mod plugin;
#[cfg(feature = "rustyline")]
pub mod rustyline;

#[cfg(feature = "derive")]
pub use bevy_commands_derive::AppCommand;
pub use clap;

pub use crate::plugin::{CommandResponse, CommandSent, CommandsPlugin};
