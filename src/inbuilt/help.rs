use std::marker::PhantomData;

use crate as bevy_commander; // for derive
use crate::*;
use bevy::prelude::*;

pub struct HelpCommandPlugin<S> {
    marker: PhantomData<S>,
}

impl<S> HelpCommandPlugin<S> {
    pub fn new() -> Self {
        Self { marker: default() }
    }
}

impl<S: CommandSender> Plugin for HelpCommandPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_command::<HelpCommand, _>(help_command::<S>);
    }
}

/// Displays information on registered commands
#[derive(clap::Parser, AppCommand)]
#[command(name = "help")]
struct HelpCommand {
    /// The command to get help information for
    query: Option<String>,
}

fn help_command<S: CommandSender>(
    ctx: CommandContext<HelpCommand, S>,
    mut commands: ResMut<AppCommands>,
) {
    let commands = &mut commands.all;
    for (cmd, sender) in ctx {
        match cmd {
            HelpCommand { query: Some(query) } => match commands.get_mut(query.as_str()) {
                Some(cmd) => sender.send_all(cmd.render_long_help().to_string().lines()),
                None => sender.send(&format!("Invalid command '{}'", query)),
            },
            _ => {
                let longest_name = commands.keys().map(|name| name.len()).max().unwrap_or(0);
                let lines: Vec<String> = commands
                    .iter()
                    .map(|(name, cmd)| {
                        let mut line =
                            format!("  {}{}", name, " ".repeat(longest_name - name.len()));
                        match cmd.get_about() {
                            Some(about) => line.push_str(&format!(" - {}", about.to_string())),
                            None => {}
                        }
                        line
                    })
                    .collect();

                sender.send("Available commands:");
                sender.send_all(lines.iter().map(|s| &**s));
            }
        }
    }
}
