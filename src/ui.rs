use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_egui::{
    egui::{self, FontId},
    EguiContexts,
};
use expedition::{egui::StyleToFormat, Message};

use crate::{CommandsPlugin, InbuiltCommandsPlugin};

pub struct UiInputPlugin;

impl Plugin for UiInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConsoleUiOpen(false))
            .insert_resource(ConsoleUiState::default())
            .add_systems(Update, (console_ui).run_if(console_ui_open));
    }
}

#[derive(Resource)]
pub struct ConsoleUiOpen(pub bool);

#[derive(Resource, Default)]
pub struct ConsoleUiState {
    pub scrollback: Vec<Message>,
}

fn console_ui_open(res: Res<ConsoleUiOpen>) -> bool {
    res.0
}

fn console_ui(mut egui: EguiContexts, mut state: ResMut<ConsoleUiState>) {
    // todo smth with this
    let fmt = StyleToFormat {
        font_id: FontId::monospace(14.0),
        ..Default::default()
    };

    egui::Window::new("Console")
        .collapsible(false)
        .show(egui.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for line in &state.scrollback {
                    ui.label(fmt.to_job(line));
                }
            });
        });
}

pub struct CommandsUiPlugins;

impl PluginGroup for CommandsUiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CommandsPlugin)
            .add(InbuiltCommandsPlugin)
            .add(UiInputPlugin)
    }
}
