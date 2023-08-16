pub mod echo;
pub mod exit;
pub mod help;

use bevy::prelude::*;

use crate::plugin::AddAppCommand;

pub struct InbuiltCommandsPlugin;

impl Plugin for InbuiltCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_app_command::<echo::EchoCommand, _>(echo::echo_command)
            .add_app_command::<exit::ExitCommand, _>(exit::exit_command)
            .add_app_command::<help::HelpCommand, _>(help::help_command);
    }
}
