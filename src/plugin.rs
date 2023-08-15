use bevy::prelude::*;
use expedition::Message;

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandSent>()
            .add_event::<CommandResponse>();
    }
}

#[derive(Event)]
pub struct CommandSent {
    pub binary: String,
    pub args: Vec<String>,
    pub sender: Entity,
}

#[derive(Event)]
pub struct CommandResponse {
    pub target: Entity,
    pub msg: Message,
}
