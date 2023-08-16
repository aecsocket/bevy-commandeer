use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex};
use std::thread;

use bevy::app::PluginGroupBuilder;
use bevy::{app::AppExit, prelude::*};
use rustyline::{error::ReadlineError, history::MemHistory, Editor};

use crate::inbuilt::InbuiltCommandsPlugin;
use crate::{CommandInput, CommandResponse, CommandSet, CommandsPlugin, Outcome};

pub type StdioEditor = Editor<(), MemHistory>;

pub struct StdioInputPlugin {
    editor: Mutex<Option<StdioEditor>>,
}

impl StdioInputPlugin {
    pub fn with_editor(editor: StdioEditor) -> Self {
        Self {
            editor: Mutex::new(Some(editor)),
        }
    }

    pub fn new() -> Self {
        let editor = StdioEditor::with_history(rustyline::Config::default(), MemHistory::new())
            .unwrap_or_else(|e| {
                error!("Could not create stdio editor: {}", e);
                panic!();
            });
        Self::with_editor(editor)
    }
}

impl Plugin for StdioInputPlugin {
    fn build(&self, app: &mut App) {
        let (mut tx_input, rx_input) = mpsc::channel::<StdioInput>();
        let (tx_prompt, rx_prompt) = mpsc::channel::<String>();

        let mut editor = self.editor.lock().unwrap().take().unwrap();
        thread::spawn(move || read_stdio(editor, tx_input, rx_prompt));

        app.insert_non_send_resource(rx_input)
            .insert_resource(StdioPrompt("".into()))
            .add_systems(Startup, setup_stdio_sender)
            .add_systems(Update, (receive_stdio_input, send_stdio_prompt).in_set(CommandSet::Dispatch))
            .add_systems(Update, respond_default_stdio.in_set(CommandSet::Response));
    }
}

pub struct StdioChannels {
    pub rx_input: Receiver<StdioInput>,
    pub tx_prompt: Sender<String>,
}

#[derive(Resource)]
pub struct StdioPrompt(String);

#[derive(Resource)]
pub struct StdioCommandSender(Entity);

#[derive(Component)]
struct DefaultStdioCommandSender;

enum StdioInput {
    Buf(String),
    Exit,
}

fn setup_stdio_sender(mut commands: Commands) {
    let sender = commands
        .spawn((Name::new("Stdio command sender"), DefaultStdioCommandSender))
        .id();
    commands.insert_resource(StdioCommandSender(sender));
}

fn read_stdio(
    mut editor: StdioEditor,
    mut tx_input: Sender<StdioInput>,
    rx_prompt: Receiver<String>,
) {
    let mut prompt = "";
    loop {
        for new_prompt in rx_prompt.try_iter() {
            prompt = &new_prompt;
        }
        match editor.readline(prompt) {
            Ok(buf) => {
                if let Err(e) = tx_input.send(StdioInput::Buf(buf)) {
                    warn!("Could not send command buffer to app: {}", e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C (press ^D to exit)");
            }
            Err(ReadlineError::Eof) => {
                println!("^D (exiting)");
                if let Err(e) = tx_input.send(StdioInput::Exit) {
                    warn!("Could not send exit signal to app: {}", e);
                }
            }
            Err(e) => {
                warn!("Could not read line from stdio: {}", e);
            }
        }
    }
}

fn receive_stdio_input(
    channels: NonSend<StdioChannels>,
    mut command_sent: EventWriter<CommandInput>,
    mut app_exit: EventWriter<AppExit>,
    sender: Res<StdioCommandSender>,
) {
    for input in channels.rx_input.try_iter() {
        match input {
            StdioInput::Buf(buf) => {
                let mut args = shlex::split(&buf).unwrap_or_default();
                if args.is_empty() {
                    continue;
                }
                let binary = args.remove(0);
                command_sent.send(CommandInput {
                    sender: sender.0,
                    name: binary,
                    args,
                })
            }
            StdioInput::Exit => app_exit.send(AppExit),
        }
    }
}

fn send_stdio_prompt(
    channels: NonSend<StdioChannels>,
    prompt: Res<StdioPrompt>,
) {
    if !prompt.is_changed() {
        return;
    }
    if let Err(e) = channels.tx_prompt.send(prompt.0.clone()) {
        warn!("Could not send new prompt to stdio: {}", e);
    }
}

fn respond_default_stdio(
    mut resps: EventReader<CommandResponse>,
    stdio_sender: Query<Entity, With<DefaultStdioCommandSender>>,
) {
    let Ok(sender) = stdio_sender.get_single() else {
        return;
    };
    for resp in resps.iter().filter(|r| r.target == sender) {
        match resp.outcome {
            Outcome::Ok => info!("{}", resp.message),
            Outcome::Err => warn!("{}", resp.message),
        }
    }
}

pub struct CommandsStdioPlugins {
    pub stdio: StdioInputPlugin,
}

impl CommandsStdioPlugins {
    pub fn new() -> Self {
        Self {
            stdio: StdioInputPlugin::new(),
        }
    }
}

impl PluginGroup for CommandsStdioPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CommandsPlugin::new())
            .add(InbuiltCommandsPlugin)
            .add(self.stdio)
    }
}
