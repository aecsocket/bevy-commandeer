use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use crate::prelude::*;
use bevy::prelude::*;

pub struct CommanderPlugin<S> {
    marker: PhantomData<S>,
}

impl<S> CommanderPlugin<S> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::default(),
        }
    }
}

impl<S: CommandSender> Plugin for CommanderPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandSent<S>>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum CommandHandleSet {
    Commands,
}

#[derive(Event)]
pub struct CommandSent<S> {
    pub name: String,
    pub args: VecDeque<String>,
    pub sender: Arc<S>,
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
