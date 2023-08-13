use bevy::app::AppExit;
use bevy::prelude::*;
use crate::prelude::*;

use crate::plugin::AppExt;

pub struct ExitCommandPlugin;

impl Plugin for ExitCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_command::<ExitCommand, _>(exit_command);
    }
}

#[derive(clap::Parser, Resource)]
struct ExitCommand;

// todo
impl AppCommand for ExitCommand {
    fn name() -> &'static str {
        "exit"
    }
}

fn exit_command(
    ctx: CommandContext<ExitCommand>,
    mut exit: EventWriter<AppExit>,
) {
    for (_, _) in ctx {
        exit.send(AppExit);
    }
}
