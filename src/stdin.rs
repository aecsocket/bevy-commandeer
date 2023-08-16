use std::sync::{mpsc, Mutex};
use std::thread;

use bevy::app::PluginGroupBuilder;
use bevy::{app::AppExit, prelude::*};
use rustyline::{error::ReadlineError, history::MemHistory, Editor};

use crate::inbuilt::InbuiltCommandsPlugin;
use crate::{CommandInput, CommandResponse, CommandSet, CommandsPlugin, Outcome};

pub type StdinEditor = Editor<(), MemHistory>;

pub struct StdinInputPlugin {
    editor: Mutex<Option<StdinEditor>>,
    prompt: String,
}

impl StdinInputPlugin {
    pub fn with_editor(editor: StdinEditor) -> Self {
        Self {
            editor: Mutex::new(Some(editor)),
            prompt: String::new(),
        }
    }

    pub fn new() -> Self {
        let editor = StdinEditor::with_history(rustyline::Config::default(), MemHistory::new())
            .unwrap_or_else(|e| {
                error!("Could not create stdin editor: {}", e);
                panic!();
            });
        Self::with_editor(editor)
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }
}

impl Plugin for StdinInputPlugin {
    fn build(&self, app: &mut App) {
        let (mut tx, rx) = mpsc::channel::<StdinInput>();

        let mut editor = self.editor.lock().unwrap().take().unwrap();
        let prompt = self.prompt.clone();
        thread::spawn(move || loop {
            read_stdin(&mut editor, &prompt, &mut tx);
        });

        app.insert_non_send_resource(rx)
            .add_systems(Startup, setup_stdio_sender)
            .add_systems(Update, process_stdin_input.in_set(CommandSet::Dispatch))
            .add_systems(Update, respond_default_stdio.in_set(CommandSet::Response));
    }
}

#[derive(Resource)]
pub struct StdioCommandSender(Entity);

#[derive(Component)]
struct DefaultStdioCommandSender;

enum StdinInput {
    Buf(String),
    Exit,
}

fn setup_stdio_sender(mut commands: Commands) {
    let sender = commands
        .spawn((Name::new("Stdio command sender"), DefaultStdioCommandSender))
        .id();
    commands.insert_resource(StdioCommandSender(sender));
}

fn read_stdin(editor: &mut StdinEditor, prompt: &String, tx: &mut mpsc::Sender<StdinInput>) {
    match editor.readline(&prompt) {
        Ok(buf) => {
            if let Err(e) = tx.send(StdinInput::Buf(buf)) {
                warn!("Could not send command buffer to app: {}", e);
            }
        }
        Err(ReadlineError::Interrupted) => {
            println!("^C (press ^D to exit)");
        }
        Err(ReadlineError::Eof) => {
            println!("^D (exiting)");
            if let Err(e) = tx.send(StdinInput::Exit) {
                warn!("Could not send exit signal to app: {}", e);
            }
        }
        Err(e) => {
            warn!("Could not read line from stdin: {}", e);
        }
    }
}

fn process_stdin_input(
    rx: NonSend<mpsc::Receiver<StdinInput>>,
    mut command_sent: EventWriter<CommandInput>,
    mut app_exit: EventWriter<AppExit>,
    sender: Res<StdioCommandSender>,
) {
    for input in rx.try_iter() {
        match input {
            StdinInput::Buf(buf) => {
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
            StdinInput::Exit => app_exit.send(AppExit),
        }
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

pub struct CommandsStdinPlugins {
    pub stdin: StdinInputPlugin,
}

impl CommandsStdinPlugins {
    pub fn new() -> Self {
        Self {
            stdin: StdinInputPlugin::new(),
        }
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.stdin = self.stdin.prompt(prompt);
        self
    }
}

impl PluginGroup for CommandsStdinPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CommandsPlugin::new())
            .add(InbuiltCommandsPlugin)
            .add(self.stdin)
    }
}
