use crate as bevy_commands;
use bevy::{app::AppExit, prelude::EventWriter};

use crate::{AppCommand, QueuedCommands};

/// Immediately exits the application.
#[derive(clap::Parser, AppCommand)]
#[command(name = "exit")]
pub struct Exit;

pub fn exit(mut queue: QueuedCommands<Exit>, mut app_exit: EventWriter<AppExit>) {
    queue.consume(|_| {
        app_exit.send(AppExit);
    });
}
