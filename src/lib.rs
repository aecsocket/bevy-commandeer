pub mod command;
pub mod plugin;
#[cfg(feature = "readline")]
pub mod readline;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;

pub use clap;

pub mod prelude {
    pub use crate::clap;
    pub use crate::command::*;
    pub use crate::plugin::*;
}
