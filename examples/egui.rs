use bevy::prelude::*;
use bevy_commands::{
    egui::{CommandsEguiPlugins, ConsoleUiOpen},
    respond_ok, AddAppCommand, AppCommand, CommandResponder, QueuedCommands,
};
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_plugins(CommandsEguiPlugins)
        .insert_resource(ConsoleUiOpen(true))
        .add_app_command::<RepeatCommand, _>(repeat_command)
        .run();
}

/// Prints the provided message back to the sender <COUNT> times
#[derive(clap::Parser, AppCommand)]
#[command(name = "repeat")]
struct RepeatCommand {
    /// The message to echo back
    message: String,
    /// The amount of times to echo the message back
    #[arg(short, long, default_value_t = 1)]
    count: usize,
}

fn repeat_command(mut queue: QueuedCommands<RepeatCommand>) {
    queue.consume(|mut ctx| {
        for _ in 0..ctx.data.count {
            respond_ok!(ctx, "Sent: {}", ctx.data.message);
        }
    })
}
