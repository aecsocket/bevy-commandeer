pub mod exit;
pub mod help;

use std::marker::PhantomData;

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::prelude::*;

pub struct InbuiltCommandPlugins<S> {
    marker: PhantomData<S>,
}

impl<S> InbuiltCommandPlugins<S> {
    pub fn new() -> Self {
        Self { marker: default() }
    }
}

impl<S: CommandSender> PluginGroup for InbuiltCommandPlugins<S> {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(exit::ExitCommandPlugin::<S>::new())
            .add(help::HelpCommandPlugin::<S>::new())
    }
}
