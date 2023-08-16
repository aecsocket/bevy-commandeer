use crate as bevy_commands;
use crate::{AppCommand, QueuedCommands};

/// Displays text back to the sender.
#[derive(clap::Parser, AppCommand)]
#[command(name = "echo")]
pub struct EchoCommand {
    /// The message to send.
    pub message: String,
}

pub fn echo_command(mut queue: QueuedCommands<EchoCommand>) {
    queue.consume(|mut ctx| {
        let message = &ctx.data.message;
        ctx.ok(message);
    });
}
