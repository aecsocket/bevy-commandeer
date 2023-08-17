use bevy::prelude::ResMut;

use crate::{self as bevy_commands, CommandMetaMap};
use crate::{AppCommand, QueuedCommands};

/// Provides usage information on registered commands.
#[derive(clap::Parser, AppCommand)]
#[command(name = "help")]
pub struct HelpCommand {
    /// The command to view help information for.
    pub query: Option<String>,
}

pub fn help_command(mut queue: QueuedCommands<HelpCommand>, mut command_meta: ResMut<CommandMetaMap>) {
    queue.consume(|mut ctx| {
        match &ctx.data.query {
            Some(query) => match command_meta.0.get_mut(query.as_str()) {
                Some(command) => {
                    for line in command.render_long_help().to_string().lines() {
                        ctx.ok(line);
                    }
                }
                None => ctx.err(format!("No such command: {}", query)),
            }
            None => {
                let longest_name_len = command_meta.0
                    .keys()
                    .map(|name| name.len())
                    .max()
                    .unwrap_or(0);
                ctx.ok("Available commands:");
                for (name, command) in command_meta.0.iter() {
                    let indent = " ".repeat(longest_name_len - name.len());
                    let message = match command.get_about() {
                        Some(about) => format!("  {}{} - {}", name, indent, about),
                        None => format!("  {}{}", name, indent),
                    };
                    ctx.ok(message);
                }
            }
        }
    });
}
