use std::collections::VecDeque;

use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_egui::{
    egui::{self, FontId, TextStyle, text_edit::CCursorRange, text::CCursor},
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
        app.add_event::<ConsoleUiDispatch>()
            .add_event::<PushConsoleUiLine>()
            .add_event::<PushConsoleUiHistory>()
            .insert_resource(ConsoleUiOpen(false))
            .insert_resource(ConsoleUiConfig::default())
            .insert_resource(ConsoleUiState::default())
            .add_systems(Startup, setup_ui_sender)
            .add_systems(Update, (console_ui).run_if(console_ui_open))
            .add_systems(Update, (dispatch).in_set(CommandSet::Dispatch))
            .add_systems(Update, (respond_default).in_set(CommandSet::Response))
            .add_systems(
                Update,
                (push_lines, push_history).after(CommandSet::Response),
            );
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
    pub prompt: String,
    pub error_style: MessageStyle,
    pub scrollback_cap: usize,
    pub history_cap: usize,
}

impl Default for ConsoleUiConfig {
    fn default() -> Self {
        Self {
            prompt: DEFAULT_PROMPT.into(),
            error_style: MessageStyle::new().color(Color32::RED),
            scrollback_cap: 10000,
            history_cap: 100,
        }
    }
}

#[derive(Resource)]
pub struct ConsoleUiState {
    scrollback: Vec<Message>,
    pub buf: String,
    history: VecDeque<String>,
    history_index: usize,
}

impl Default for ConsoleUiState {
    fn default() -> Self {
        Self {
            scrollback: Vec::new(),
            buf: String::new(),
            history: VecDeque::from([String::new()]),
            history_index: 0,
        }
    }
}

impl ConsoleUiState {
    pub fn scrollback(&self) -> &Vec<Message> {
        &self.scrollback
    }
}

#[derive(Event)]
pub struct ConsoleUiDispatch(pub String);

#[derive(Event)]
pub struct PushConsoleUiLine(pub Message);

#[derive(Event)]
pub struct PushConsoleUiHistory(pub String);

fn console_ui_open(res: Res<ConsoleUiOpen>) -> bool {
    res.0
}

fn console_ui(
    mut egui: EguiContexts,
    open: Res<ConsoleUiOpen>,
    mut state: ResMut<ConsoleUiState>,
    mut dispatch: EventWriter<ConsoleUiDispatch>,
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
                    state.buf.clear();
                    state.history_index = 0;
                    dispatch.send(ConsoleUiDispatch(buf));
                }

                if entered || open.is_changed() {
                    ui.memory_mut(|m| m.request_focus(buf_edit_resp.id));
                }

                if buf_edit_resp.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                    && state.history.len() > 1
                    && state.history_index < state.history.len() - 1
                {
                    if state.history_index == 0 && !state.buf.trim().is_empty() {
                        state.history[0] = state.buf.to_owned();
                    }

                    state.history_index += 1;
                    state.buf = state.history[state.history_index].clone();

                    set_cursor_pos(ui.ctx(), buf_edit_resp.id, state.buf.len());
                } else if buf_edit_resp.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::ArrowDown))
                    && state.history_index > 0
                {
                    state.history_index -= 1;
                    state.buf = state.history[state.history_index].clone();

                    set_cursor_pos(ui.ctx(), buf_edit_resp.id, state.buf.len());
                }
            });
        });
}

fn set_cursor_pos(ctx: &egui::Context, id: egui::Id, pos: usize) {
    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
        state.set_ccursor_range(Some(CCursorRange::one(CCursor::new(pos))));
        state.store(ctx, id);
    }
}

fn dispatch(
    mut events: EventReader<ConsoleUiDispatch>,
    mut push_line: EventWriter<PushConsoleUiLine>,
    mut push_history: EventWriter<PushConsoleUiHistory>,
    mut command_input: EventWriter<CommandBufInput>,
    config: Res<ConsoleUiConfig>,
    sender: Res<EguiCommandSender>,
) {
    for event in events.iter() {
        let buf = event.0.clone();
        if !buf.is_empty() {
            info!("Issued console command: {}", buf);
            push_line.send(PushConsoleUiLine(
                format!("{}{}", config.prompt, buf).into(),
            ));
            push_history.send(PushConsoleUiHistory(buf.clone()));
            command_input.send(CommandBufInput {
                sender: sender.0,
                buf,
            });
        }
    }
}

fn respond_default(
    mut events: EventReader<CommandResponse>,
    sender: Query<Entity, With<DefaultEguiCommandSender>>,
    ui_config: Res<ConsoleUiConfig>,
    mut push_lines: EventWriter<PushConsoleUiLine>,
) {
    let Ok(sender) = sender.get_single() else {
        return;
    };
    for resp in events.iter().filter(|r| r.target == sender) {
        push_lines.send(PushConsoleUiLine(match resp.outcome {
            Outcome::Ok => resp.message.clone(),
            Outcome::Err => resp.message.clone().with_style(ui_config.error_style),
        }));
    }
}

fn push_lines(
    mut events: EventReader<PushConsoleUiLine>,
    config: Res<ConsoleUiConfig>,
    mut state: ResMut<ConsoleUiState>,
) {
    for event in events.iter() {
        state.scrollback.push(event.0.clone());
    }
    let len = state.scrollback.len();
    let cap = config.scrollback_cap + 1;
    if len > cap {
        state.scrollback.drain(0..(len - cap));
    }
}

fn push_history(
    mut events: EventReader<PushConsoleUiHistory>,
    config: Res<ConsoleUiConfig>,
    mut state: ResMut<ConsoleUiState>,
) {
    for event in events.iter() {
        state.history.insert(1, event.0.clone());
    }
    let len = state.history.len();
    let cap = config.history_cap;
    if len > cap {
        state.history.drain(0..(len - cap));
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
