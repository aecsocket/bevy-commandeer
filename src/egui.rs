use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_egui::{
    egui::{self, FontId, TextStyle},
    EguiContexts,
};
use expedition::{egui::StyleToFormat, Color32, Message, MessageStyle, Styleable};

use crate::{
    CommandBufInput, CommandResponse, CommandSet, CommandsPlugin, InbuiltCommandsPlugin, Outcome,
    DEFAULT_PROMPT,
};

pub struct EguiInputPlugin;

impl Plugin for EguiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PushConsoleUiLine>()
            .insert_resource(ConsoleUiOpen(false))
            .insert_resource(ConsoleUiConfig::default())
            .insert_resource(ConsoleUiState::default())
            .add_systems(Startup, setup_ui_sender)
            .add_systems(Update, (console_ui).run_if(console_ui_open))
            .add_systems(Update, (respond_default_ui).in_set(CommandSet::Response))
            .add_systems(Update, (push_console_lines).after(CommandSet::Response));
    }
}

#[derive(Resource)]
pub struct EguiCommandSender(pub Entity);

#[derive(Component)]
struct DefaultEguiCommandSender;

fn setup_ui_sender(mut commands: Commands) {
    let sender = commands
        .spawn((
            Name::new("Console UI command sender"),
            DefaultEguiCommandSender,
        ))
        .id();
    commands.insert_resource(EguiCommandSender(sender));
}

#[derive(Resource)]
pub struct ConsoleUiOpen(pub bool);

#[derive(Resource)]
pub struct ConsoleUiConfig {
    pub error_style: MessageStyle,
    pub scrollback_cap: usize,
}

impl Default for ConsoleUiConfig {
    fn default() -> Self {
        Self {
            error_style: MessageStyle::new().color(Color32::RED),
            scrollback_cap: 10_000,
        }
    }
}

#[derive(Resource)]
pub struct ConsoleUiState {
    pub prompt: String,
    scrollback: Vec<Message>,
    pub buf: String,
}

impl Default for ConsoleUiState {
    fn default() -> Self {
        Self {
            prompt: DEFAULT_PROMPT.into(),
            scrollback: Vec::new(),
            buf: String::new(),
        }
    }
}

impl ConsoleUiState {
    pub fn scrollback(&self) -> &Vec<Message> {
        &self.scrollback
    }
}

#[derive(Event)]
pub struct PushConsoleUiLine(pub Message);

fn console_ui_open(res: Res<ConsoleUiOpen>) -> bool {
    res.0
}

fn console_ui(
    mut egui: EguiContexts,
    open: Res<ConsoleUiOpen>,
    mut state: ResMut<ConsoleUiState>,
    mut command_input: EventWriter<CommandBufInput>,
    sender: Res<EguiCommandSender>,
    mut push_line: EventWriter<PushConsoleUiLine>,
) {
    let formatter = StyleToFormat {
        font_id: FontId::monospace(14.0),
        ..Default::default()
    };

    egui::Window::new("Console")
        .collapsible(false)
        .resizable(true)
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
                let entered =
                    buf_edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if entered {
                    let buf = state.buf.trim().to_owned();
                    push_line.send(PushConsoleUiLine(format!("{}{}", state.prompt, buf).into()));
                    state.buf.clear();
                    if !buf.is_empty() {
                        info!("Issued console command: {}", buf);
                        command_input.send(CommandBufInput {
                            sender: sender.0,
                            buf,
                        });
                    }
                }

                if entered || open.is_changed() {
                    ui.memory_mut(|m| m.request_focus(buf_edit_resp.id));
                }
            });
        });
}

fn respond_default_ui(
    mut resps: EventReader<CommandResponse>,
    sender: Query<Entity, With<DefaultEguiCommandSender>>,
    ui_config: Res<ConsoleUiConfig>,
    mut push_lines: EventWriter<PushConsoleUiLine>,
) {
    let Ok(sender) = sender.get_single() else {
        return;
    };
    for resp in resps.iter().filter(|r| r.target == sender) {
        push_lines.send(PushConsoleUiLine(match resp.outcome {
            Outcome::Ok => resp.message.clone(),
            Outcome::Err => resp.message.clone().with_style(ui_config.error_style),
        }));
    }
}

fn push_console_lines(
    mut lines: EventReader<PushConsoleUiLine>,
    ui_config: Res<ConsoleUiConfig>,
    mut ui_state: ResMut<ConsoleUiState>,
) {
    for line in lines.iter() {
        ui_state.scrollback.push(line.0.clone());
    }
    // remove old entries
    if ui_state.scrollback.len() > ui_config.scrollback_cap {
        ui_state.scrollback.drain(0..ui_config.scrollback_cap);
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
