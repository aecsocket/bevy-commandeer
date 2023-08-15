use bevy::{log::LogPlugin, prelude::*};
use bevy_commandeer::prelude::*;

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
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(CommandeerReadlinePlugin::<Sender>::new().prompt("> "))
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

fn echo_command(ctx: CommandQueue<EchoCommand, Sender>) {
    for (cmd, sender) in ctx {
        sender.send_lines(&cmd.message);
    }
}

fn setup() {
    info!("Type `echo <message>` to echo a message back to the console");
    info!("Or type `help` to see all registered commands");
}
