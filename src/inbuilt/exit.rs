use crate as bevy_commands;
use bevy::{app::AppExit, prelude::EventWriter};

use crate::{AppCommand, QueuedCommands};

/// Immediately exits the application.
#[derive(clap::Parser, AppCommand)]
#[command(name = "exit")]
pub struct ExitCommand;

pub fn exit_command(mut queue: QueuedCommands<ExitCommand>, mut app_exit: EventWriter<AppExit>) {
    queue.consume(|_| {
        app_exit.send(AppExit);
    });
}
