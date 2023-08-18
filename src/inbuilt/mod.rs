pub mod echo;
pub mod exit;
pub mod help;

use bevy::prelude::*;

use crate::plugin::AddAppCommand;

pub struct InbuiltCommandsPlugin;

impl Plugin for InbuiltCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_app_command::<echo::Echo, _>(echo::echo)
            .add_app_command::<exit::Exit, _>(exit::exit)
            .add_app_command::<help::Help, _>(help::help);
    }
}
