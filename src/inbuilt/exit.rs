use std::marker::PhantomData;

use bevy::app::AppExit;
use bevy::prelude::*;
use crate::prelude::*;

use crate::plugin::AppExt;

pub struct ExitCommandPlugin<S> {
    marker: PhantomData<S>,
}

impl<S> ExitCommandPlugin<S> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::default(),
        }
    }
}

impl<S: CommandSender> Plugin for ExitCommandPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_command::<ExitCommand, _>(exit_command::<S>);
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

fn exit_command<S>(
    ctx: CommandContext<ExitCommand, S>,
    mut exit: EventWriter<AppExit>,
) {
    for (_, _) in ctx {
        exit.send(AppExit);
    }
}
