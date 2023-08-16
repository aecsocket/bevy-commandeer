//#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::nursery)]
//#![warn(clippy::cargo)]

pub mod dispatch;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;
pub mod plugin;
#[cfg(feature = "stdio")]
pub mod stdio;
#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "derive")]
pub use bevy_commands_derive::AppCommand;
pub use clap;

pub use crate::dispatch::{
    AppCommand, CommandContext, CommandDispatch, CommandResponse, Outcome, QueuedCommands,
};
pub use crate::plugin::{AddAppCommand, CommandInput, CommandSet, CommandsPlugin};
#[cfg(feature = "stdio")]
pub use crate::stdio::{StdioInputPlugin, CommandsStdioPlugins};
#[cfg(feature = "ui")]
pub use crate::ui::{UiInputPlugin, CommandsUiPlugins};
