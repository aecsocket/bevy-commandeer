pub mod exit;

use bevy::{prelude::*, app::PluginGroupBuilder};

pub struct InbuiltCommandPlugins;

impl PluginGroup for InbuiltCommandPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(exit::ExitCommandPlugin)
    }
}
