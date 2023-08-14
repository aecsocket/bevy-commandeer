pub mod exit;
pub mod help;

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::*;

pub struct InbuiltCommandsPlugin<S> {
    marker: PhantomData<S>,
}

impl<S> InbuiltCommandsPlugin<S> {
    pub fn new() -> Self {
        Self { marker: default() }
    }
}

impl<S: CommandSender> Plugin for InbuiltCommandsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_command::<exit::ExitCommand, _>(exit::exit_command::<S>)
            .add_command::<help::HelpCommand, _>(help::help_command::<S>);
    }
}
