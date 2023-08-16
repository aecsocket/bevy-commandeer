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
#[cfg(feature = "inbuilt")]
pub use crate::inbuilt::InbuiltCommandsPlugin;
pub use crate::plugin::{AddAppCommand, CommandInput, CommandSet, CommandsPlugin, CommandMetaMap};
#[cfg(feature = "stdio")]
pub use crate::stdio::{CommandsStdioPlugins, StdioInputPlugin, StdioPrompt};
#[cfg(feature = "ui")]
pub use crate::ui::{CommandsUiPlugins, UiInputPlugin};
