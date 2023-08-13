use bevy::{log::LogPlugin, prelude::*};
use bevy_commander::{prelude::*, readline::CommanderReadlinePlugin};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins((CommanderPlugin, CommanderReadlinePlugin::with_prompt("> ")))
        .add_command::<EchoCommand, _>(echo_command)
        .add_systems(Startup, setup)
        .run();
}

#[derive(clap::Parser, Resource)]
struct EchoCommand {
    msg: String,
}

impl AppCommand for EchoCommand {
    fn name() -> &'static str {
        "echo"
    }
}

fn echo_command(ctx: CommandContext<EchoCommand>) {
    for (cmd, sender) in ctx {
        sender.send(&cmd.msg);
    }
}

fn setup() {
    info!("Type `echo <message>` to echo a message back to the console");
}
