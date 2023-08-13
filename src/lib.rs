pub mod command;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;
pub mod plugin;
#[cfg(feature = "readline")]
pub mod readline;
#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "derive")]
pub use bevy_commandeer_derive::AppCommand;

pub use clap;
pub use command::*;
pub use plugin::*;
#[cfg(feature = "inbuilt")]
pub use inbuilt::*;
#[cfg(feature = "readline")]
pub use readline::*;
#[cfg(feature = "ui")]
pub use ui::*;
