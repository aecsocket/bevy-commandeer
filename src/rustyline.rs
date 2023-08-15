use std::{
    sync::{mpsc, Mutex},
    thread,
};

use bevy::{app::AppExit, prelude::*};
use rustyline::{error::ReadlineError, history::MemHistory, Editor};

use crate::{CommandResponse, CommandSent};

pub type StdinEditor = Editor<(), MemHistory>;

pub struct RustylineInputPlugin {
    editor: Mutex<Option<StdinEditor>>,
    prompt: String,
}

impl RustylineInputPlugin {
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

impl Plugin for RustylineInputPlugin {
    fn build(&self, app: &mut App) {
        let (mut tx, rx) = mpsc::channel::<StdinInput>();

        let mut editor = self.editor.lock().unwrap().take().unwrap();
        let prompt = self.prompt.clone();
        thread::spawn(move || loop {
            read_stdin(&mut editor, &prompt, &mut tx);
        });

        app.insert_non_send_resource(rx)
            .add_systems(Update, process_stdin_input);
    }
}

enum StdinInput {
    Buf(String),
    Exit,
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

#[derive(Component)]
pub struct StdioCommandSender;

fn process_stdin_input(
    rx: NonSend<mpsc::Receiver<StdinInput>>,
    mut command_sent: EventWriter<CommandSent>,
    mut app_exit: EventWriter<AppExit>,
    senders: Query<Entity, With<StdioCommandSender>>,
) {
    for input in rx.try_iter() {
        match input {
            StdinInput::Buf(buf) => {
                let mut args = shlex::split(&buf).unwrap_or_default();
                if args.is_empty() {
                    continue;
                }
                let binary = args.remove(0);
                command_sent.send(CommandSent {
                    binary,
                    args,
                    sender: sender.0, // ??!?!?!?!??!
                })
            }
            StdinInput::Exit => app_exit.send(AppExit),
        }
    }
}

fn output_response_stdio(mut events: EventReader<CommandResponse>) {}
