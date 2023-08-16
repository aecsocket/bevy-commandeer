//#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::nursery)]
//#![warn(clippy::cargo)]

pub mod dispatch;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;
pub mod plugin;
#[cfg(feature = "stdin")]
pub mod stdin;

#[cfg(feature = "derive")]
pub use bevy_commands_derive::AppCommand;
pub use clap;

pub use crate::dispatch::{
    AppCommand, CommandContext, CommandDispatch, CommandResponse, Outcome, QueuedCommands,
};
pub use crate::plugin::{AddAppCommand, CommandInput, CommandSet, CommandsPlugin};
#[cfg(feature = "stdin")]
pub use crate::stdin::StdinInputPlugin;
