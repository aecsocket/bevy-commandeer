use bevy::{log::LogPlugin, prelude::*};
use bevy_commander::*;

pub enum Sender {
    Console,
}

impl CommandSender for Sender {
    fn send_all<'a>(&self, lines: impl IntoIterator<Item = &'a str>) {
        for line in lines {
            info!("{}", line);
        }
    }
}

impl ConsoleCommandSender for Sender {
    fn console() -> Self {
        Self::Console
    }
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins((
            CommanderPlugin::<Sender>::new(),
            CommanderReadlinePlugin::<Sender>::with_prompt(""),
            InbuiltCommandPlugins::<Sender>::new(),
        ))
        .add_command::<EchoCommand, _>(echo_command)
        .add_systems(Startup, setup)
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

fn setup() {
    info!("Type `echo <message>` to echo a message back to the console");
    info!("Or type `help` to see all registered commands");
}
