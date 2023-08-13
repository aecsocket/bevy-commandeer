use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_commandeer::*;

pub struct Sender;

impl CommandSender for Sender {
    fn send_all<'a>(&self, lines: impl IntoIterator<Item = &'a str>) {
        for line in lines {
            info!("{}", line);
        }
    }
}

impl ConsoleCommandSender for Sender {
    fn console() -> Self {
        Self
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_plugins((
            CommandeerPlugin::<Sender>::new(),
            InbuiltCommandPlugins::<Sender>::new(),
            CommandeerUiPlugin::<Sender>::new(),
        ))
        .add_command::<EchoCommand, _>(echo_command)
        .run();
}

/// Prints the provided message back to the sender
#[derive(clap::Parser, AppCommand)]
#[command(name = "echo")]
struct EchoCommand {
    /// The message to echo back
    message: String,
}

fn echo_command(ctx: CommandContext<EchoCommand, Sender>) {
    for (cmd, sender) in ctx {
        sender.send(&cmd.message);
    }
}
