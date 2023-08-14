use bevy::app::AppExit;
use bevy::prelude::*;

use crate as bevy_commandeer;
use crate::prelude::*;

/// Immediately exits the application
#[derive(clap::Parser, AppCommand)]
#[command(name = "exit")]
pub struct ExitCommand;

pub fn exit_command<S>(ctx: CommandContext<ExitCommand, S>, mut exit: EventWriter<AppExit>) {
    for (_, _) in ctx {
        exit.send(AppExit);
    }
}
