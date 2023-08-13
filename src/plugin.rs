use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

pub struct CommanderPlugin<S> {
    marker: PhantomData<S>,
}

impl<S> CommanderPlugin<S> {
    pub fn new() -> Self {
        Self { marker: default() }
    }
}

impl<S: CommandSender> Plugin for CommanderPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(AppCommands::default())
            .add_event::<CommandSent<S>>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum CommandHandleSet {
    Commands,
}

#[derive(Resource, Default)]
pub struct AppCommands {
    pub all: HashMap<&'static str, clap::Command>,
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
        let setup = move |mut commands: ResMut<AppCommands>| {
            let command = create_command::<C>();
            let name = C::name();
            if commands.all.contains_key(name) {
                warn!("Command '{}' is already registered, overwriting", name);
            }
            commands.all.insert(name, command);
        };

        self.add_systems(Startup, setup)
            .add_systems(Update, system.in_set(CommandHandleSet::Commands))
    }
}
