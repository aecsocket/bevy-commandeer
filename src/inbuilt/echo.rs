use crate as bevy_commands;
use crate::{AppCommand, CommandResponder, QueuedCommands};

/// Displays text back to the sender.
#[derive(clap::Parser, AppCommand)]
#[command(name = "echo")]
pub struct Echo {
    /// The message to send.
    pub message: String,
}

pub fn echo(mut queue: QueuedCommands<Echo>) {
    queue.consume(|mut ctx| {
        let message = &ctx.data.message;
        ctx.ok(message);
    });
}
