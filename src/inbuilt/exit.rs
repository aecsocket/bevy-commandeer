use std::marker::PhantomData;

use crate::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;

pub struct ExitCommandPlugin<S> {
    marker: PhantomData<S>,
}

impl<S> ExitCommandPlugin<S> {
    pub fn new() -> Self {
        Self { marker: default() }
    }
}

impl<S: CommandSender> Plugin for ExitCommandPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_command::<ExitCommand, _>(exit_command::<S>);
    }
}

/// Immediately exits the application
#[derive(clap::Parser, Resource)]
struct ExitCommand;

// todo
impl AppCommand for ExitCommand {
    fn name() -> &'static str {
        "exit"
    }
}

fn exit_command<S>(ctx: CommandContext<ExitCommand, S>, mut exit: EventWriter<AppExit>) {
    for (_, _) in ctx {
        exit.send(AppExit);
    }
}
