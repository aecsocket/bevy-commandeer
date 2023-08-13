pub mod exit;

use std::marker::PhantomData;

use bevy::{prelude::*, app::PluginGroupBuilder};

use crate::prelude::*;

pub struct InbuiltCommandPlugins<S> {
    marker: PhantomData<S>,
}

impl<S: CommandSender> PluginGroup for InbuiltCommandPlugins<S> {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(exit::ExitCommandPlugin::<S>::new())
    }
}
