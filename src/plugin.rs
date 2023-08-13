use std::collections::VecDeque;

use crate::prelude::*;
use bevy::prelude::*;

pub struct CommanderPlugin;

impl Plugin for CommanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandSent>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum CommandHandleSet {
    Commands,
}

#[derive(Event)]
pub struct CommandSent {
    pub name: String,
    pub args: VecDeque<String>,
    pub sender: BoxedSender,
}

pub trait AppExt {
    fn add_command<C: AppCommand, Params>(
        &mut self,
        system: impl IntoSystemConfigs<Params>,
    ) -> &mut Self;
}

impl AppExt for App {
    fn add_command<C: AppCommand, Params>(
        &mut self,
        system: impl IntoSystemConfigs<Params>,
    ) -> &mut Self {
        self.add_systems(Update, system.in_set(CommandHandleSet::Commands))
    }
}
