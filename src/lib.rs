pub mod command;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;
pub mod plugin;
#[cfg(feature = "readline")]
pub mod readline;
#[cfg(feature = "ui")]
pub mod ui;

pub mod prelude {
    #[cfg(feature = "derive")]
    pub use bevy_commandeer_derive::AppCommand;
    pub use clap;

    pub use crate::command::{AppCommand, CommandContext, CommandSender};
    #[cfg(feature = "inbuilt")]
    pub use crate::inbuilt::InbuiltCommandsPlugin;
    pub use crate::plugin::{AppCommands, AppExt, CommandSent, CommandeerPlugin};
    #[cfg(feature = "readline")]
    pub use crate::readline::{CommandeerReadlinePlugin, ConsoleCommandSender, ReadlinePlugin};
    #[cfg(feature = "ui")]
    pub use crate::ui::{CommandeerUiPlugin, ConsoleUiPlugin};
}
