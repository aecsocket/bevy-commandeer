use std::{collections::VecDeque, marker::PhantomData, sync::Mutex};

use bevy::ecs::{
    component::Tick, schedule::BoxedCondition, world::unsafe_world_cell::UnsafeWorldCell,
};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, text::LayoutJob, Color32, FontId, ScrollArea, TextEdit, TextFormat, TextStyle},
    EguiContexts,
};
use clap::builder::StyledStr;

use crate::prelude::*;

pub struct ConsoleUiPlugin<S> {
    condition: Mutex<Option<BoxedCondition>>,
    marker: PhantomData<S>,
}

impl<S> ConsoleUiPlugin<S> {
    pub fn new() -> Self {
        Self {
            condition: Mutex::new(None),
            marker: default(),
        }
    }

    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl<S: CommandSender> Plugin for ConsoleUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConsoleState::default())
            .insert_resource(ConsoleConfig::default());

        let mut console_ui = console_ui.into_configs();
        if let Some(condition) = self.condition.lock().unwrap().take() {
            console_ui = console_ui.run_if(BoxedConditionHelper(condition));
        }
        app.add_systems(Update, console_ui);
    }
}

pub struct CommandeerUiPlugin<S> {
    plugin: Mutex<Option<ConsoleUiPlugin<S>>>,
}

impl<S> CommandeerUiPlugin<S> {
    pub fn new() -> Self {
        Self {
            plugin: Mutex::new(Some(ConsoleUiPlugin::new())),
        }
    }

    pub fn run_if<M>(self, condition: impl Condition<M>) -> Self {
        {
            let mut plugin = self.plugin.lock().unwrap();
            *plugin = plugin.take().map(|p| p.run_if(condition));
        }
        self
    }
}

impl<S: CommandSender> Plugin for CommandeerUiPlugin<S> {
    fn build(&self, app: &mut App) {
        let plugin = self
            .plugin
            .lock()
            .unwrap()
            .take()
            .expect("plugin has already been applied");
        app.add_plugins((
            CommandeerPlugin::<S>::new(),
            InbuiltCommandsPlugin::<S>::new(),
            plugin,
        ));
    }
}

#[derive(Debug, Resource)]
pub struct ConsoleConfig {
    pub pos: egui::Pos2,
    pub size: egui::Vec2,
    pub history_size: usize,
    pub prompt: String,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            pos: egui::pos2(200.0, 100.0),
            size: egui::vec2(400.0, 800.0),
            history_size: 100,
            prompt: "".to_owned(),
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct ConsoleState {
    pub buf: String,
    pub scrollback: Vec<StyledStr>,
    pub history: VecDeque<String>,
    pub history_index: usize,
}

fn console_ui(mut egui: EguiContexts, config: Res<ConsoleConfig>, mut state: ResMut<ConsoleState>) {
    egui::Window::new("Console")
        .collapsible(false)
        .default_pos(config.pos)
        .default_size(config.size)
        .resizable(true)
        .show(egui.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for line in &state.scrollback {
                                let mut text = LayoutJob::default();
                                text.append(
                                    &line.to_string(),
                                    0.0,
                                    TextFormat::simple(FontId::monospace(14.0), Color32::GRAY),
                                );
                                ui.label(text);
                            }
                        })
                    });

                ui.separator();

                let buf_edit = TextEdit::singleline(&mut state.buf)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .font(TextStyle::Monospace);

                ui.add(buf_edit);
            })
        });
}

// copied from
// https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/crates/bevy-inspector-egui/src/quick.rs

struct BoxedConditionHelper(BoxedCondition);

// SAFETY: BoxedCondition is a Box<dyn ReadOnlySystem>
unsafe impl ReadOnlySystem for BoxedConditionHelper {}

impl System for BoxedConditionHelper {
    type In = ();
    type Out = bool;

    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.0.name()
    }

    fn type_id(&self) -> std::any::TypeId {
        self.0.type_id()
    }

    fn component_access(&self) -> &bevy::ecs::query::Access<bevy::ecs::component::ComponentId> {
        self.0.component_access()
    }

    fn archetype_component_access(
        &self,
    ) -> &bevy::ecs::query::Access<bevy::ecs::archetype::ArchetypeComponentId> {
        self.0.archetype_component_access()
    }

    fn is_send(&self) -> bool {
        self.0.is_send()
    }

    fn is_exclusive(&self) -> bool {
        self.0.is_exclusive()
    }
    unsafe fn run_unsafe(&mut self, input: Self::In, world: UnsafeWorldCell) -> Self::Out {
        // SAFETY: same as this method
        unsafe { self.0.run_unsafe(input, world) }
    }

    fn apply_deferred(&mut self, world: &mut World) {
        self.0.apply_deferred(world)
    }

    fn initialize(&mut self, _world: &mut World) {
        self.0.initialize(_world)
    }

    fn update_archetype_component_access(&mut self, world: UnsafeWorldCell) {
        self.0.update_archetype_component_access(world)
    }
    fn check_change_tick(&mut self, change_tick: Tick) {
        self.0.check_change_tick(change_tick)
    }
    fn get_last_run(&self) -> Tick {
        self.0.get_last_run()
    }
    fn set_last_run(&mut self, last_run: Tick) {
        self.0.set_last_run(last_run)
    }
}
