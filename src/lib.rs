pub mod command;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;
pub mod plugin;
#[cfg(feature = "readline")]
pub mod readline;

#[cfg(feature = "derive")]
pub use bevy_commander_derive::AppCommand;

pub use clap;
pub use command::*;
pub use plugin::*;
#[cfg(feature = "inbuilt")]
pub use crate::inbuilt::*;
#[cfg(feature = "readline")]
pub use crate::readline::*;
