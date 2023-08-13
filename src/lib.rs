pub mod command;
#[cfg(feature = "inbuilt")]
pub mod inbuilt;
pub mod plugin;
#[cfg(feature = "readline")]
pub mod readline;

pub use clap;

pub mod prelude {
    pub use crate::clap;
    pub use crate::command::*;
    #[cfg(feature = "inbuilt")]
    pub use crate::inbuilt::*;
    pub use crate::plugin::*;
    #[cfg(feature = "readline")]
    pub use crate::readline::*;
}
