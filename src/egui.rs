use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_egui::{
    egui::{self, FontId, TextStyle},
    EguiContexts,
};
use expedition::{egui::StyleToFormat, Message, Styleable, Color32, MessageStyle};

use crate::{CommandBufInput, CommandsPlugin, InbuiltCommandsPlugin, DEFAULT_PROMPT, CommandResponse, CommandSet, Outcome};

pub struct EguiInputPlugin;

impl Plugin for EguiInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConsoleUiOpen(false))
            .insert_resource(ConsoleUiState::default())
            .add_systems(Startup, setup_ui_sender)
            .add_systems(Update, (console_ui).run_if(console_ui_open))
            .add_systems(Update, (respond_default_ui).in_set(CommandSet::Response));
    }
}

#[derive(Resource)]
pub struct EguiCommandSender(pub Entity);

#[derive(Component)]
struct DefaultEguiCommandSender;

fn setup_ui_sender(mut commands: Commands) {
    let sender = commands
        .spawn((Name::new("Console UI command sender"), DefaultEguiCommandSender))
        .id();
    commands.insert_resource(EguiCommandSender(sender));
}

#[derive(Resource)]
pub struct ConsoleUiOpen(pub bool);

#[derive(Resource)]
pub struct ConsoleUiState {
    pub prompt: String,
    pub scrollback: Vec<Message>,
    pub buf: String,
    pub error_style: MessageStyle,
}

impl Default for ConsoleUiState {
    fn default() -> Self {
        Self {
            prompt: DEFAULT_PROMPT.into(),
            scrollback: Vec::new(),
            buf: String::new(),
            error_style: MessageStyle::new().color(Color32::RED),
        }
    }
}

impl ConsoleUiState {
    pub fn push_line(&mut self, message: impl Into<Message>) {
        // todo remove old entries
        self.scrollback.push(message.into());
    }
}

fn console_ui_open(res: Res<ConsoleUiOpen>) -> bool {
    res.0
}

fn console_ui(
    mut egui: EguiContexts,
    mut state: ResMut<ConsoleUiState>,
    mut command_input: EventWriter<CommandBufInput>,
    sender: Res<EguiCommandSender>,
) {
    let formatter = StyleToFormat {
        font_id: FontId::monospace(14.0),
        ..Default::default()
    };

    egui::Window::new("Console")
        .collapsible(false)
        .resizable(true)
        .default_pos([200.0, 100.0])
        .default_size([800.0, 400.0])
        .show(egui.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 30.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &state.scrollback {
                            ui.label(formatter.to_job(line));
                        }
                    });

                ui.separator();

                let buf_edit = egui::TextEdit::singleline(&mut state.buf)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .font(TextStyle::Monospace);

                let buf_edit_resp = ui.add(buf_edit);
                if buf_edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let buf = state.buf.trim().to_owned();
                    let prompt_line = format!("{}{}", state.prompt, buf);
                    state.push_line(prompt_line);
                    state.buf.clear();
                    if !buf.is_empty() {
                        command_input.send(CommandBufInput {
                            sender: sender.0,
                            buf,
                        });
                    }

                    ui.memory_mut(|m| m.request_focus(buf_edit_resp.id));
                }
            });
        });
}

fn respond_default_ui(
    mut resps: EventReader<CommandResponse>,
    sender: Query<Entity, With<DefaultEguiCommandSender>>,
    mut ui_state: ResMut<ConsoleUiState>,
) {
    let Ok(sender) = sender.get_single() else {
        return;
    };
    for resp in resps.iter().filter(|r| r.target == sender) {
        let error_style = ui_state.error_style;
        ui_state.push_line(match resp.outcome {
            Outcome::Ok => resp.message.clone(),
            Outcome::Err => resp.message.clone().with_style(error_style),
        });
    }
}

pub struct CommandsEguiPlugins;

impl PluginGroup for CommandsEguiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CommandsPlugin)
            .add(InbuiltCommandsPlugin)
            .add(EguiInputPlugin)
    }
}
