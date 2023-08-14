use std::marker::PhantomData;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use bevy::app::AppExit;
use bevy::prelude::*;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::Editor;

use crate::prelude::*;

pub type ConsoleEditor = Editor<(), MemHistory>;

pub trait ConsoleCommandSender: CommandSender {
    fn console() -> Self;
}

enum ReadlineInput {
    Command(String),
    Exit,
}

pub struct ReadlinePlugin<S> {
    editor: Mutex<Option<ConsoleEditor>>,
    prompt: String,
    marker: PhantomData<S>,
}

impl<S> ReadlinePlugin<S> {
    pub fn with_editor(editor: ConsoleEditor) -> Self {
        Self {
            editor: Mutex::new(Some(editor)),
            prompt: "".to_owned(),
            marker: default(),
        }
    }

    pub fn new() -> Self {
        let editor = Editor::with_history(default(), MemHistory::new()).unwrap_or_else(|e| {
            error!("Could not create console command input: {}", e);
            panic!();
        });
        Self::with_editor(editor)
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }
}

impl<S: ConsoleCommandSender> Plugin for ReadlinePlugin<S> {
    fn build(&self, app: &mut App) {
        let (tx, rx) = mpsc::channel::<ReadlineInput>();

        let mut editor = self
            .editor
            .lock()
            .unwrap()
            .take()
            .expect("plugin has already been applied");
        let prompt = self.prompt.clone();

        thread::spawn(move || loop {
            match editor.readline(&prompt) {
                Ok(inp) => {
                    if inp.is_empty() {
                        continue;
                    }

                    match editor.add_history_entry(&inp) {
                        Ok(_) => {}
                        Err(e) => warn!("Could not add entry to console history: {}", e),
                    }

                    for line in inp.lines() {
                        match tx.send(ReadlineInput::Command(line.to_owned())) {
                            Ok(_) => {}
                            Err(e) => warn!("Could not send command input to app: {}", e),
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C (press ^D to exit)");
                }
                Err(ReadlineError::Eof) => {
                    println!("^D (exiting)");
                    match tx.send(ReadlineInput::Exit) {
                        Ok(_) => {}
                        Err(e) => warn!("Could not send exit input to app: {}", e),
                    }
                }
                Err(e) => warn!("Could not read line from console input: {}", e),
            }
        });

        app.insert_non_send_resource(rx)
            .add_systems(Update, receive_readline::<S>);
    }
}

pub struct CommandeerReadlinePlugin<S> {
    plugin: Mutex<Option<ReadlinePlugin<S>>>,
}

impl<S> CommandeerReadlinePlugin<S> {
    pub fn new() -> Self {
        Self {
            plugin: Mutex::new(Some(ReadlinePlugin::new())),
        }
    }

    pub fn prompt(self, prompt: impl Into<String>) -> Self {
        {
            let mut plugin = self.plugin.lock().unwrap();
            *plugin = plugin.take().map(|p| p.prompt(prompt));
        }
        self
    }
}

impl<S: ConsoleCommandSender> Plugin for CommandeerReadlinePlugin<S> {
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

fn receive_readline<S: ConsoleCommandSender>(
    receiver: NonSend<mpsc::Receiver<ReadlineInput>>,
    mut sent_command: EventWriter<CommandSent<S>>,
    mut exit: EventWriter<AppExit>,
) {
    for input in receiver.try_iter() {
        match input {
            ReadlineInput::Command(command) => {
                let Some(mut args) = shlex::split(&command) else {
                    continue;
                };
                if args.is_empty() {
                    continue;
                }
                let name = args.remove(0);

                sent_command.send(CommandSent {
                    name,
                    args,
                    sender: Arc::new(S::console()),
                });
            }
            ReadlineInput::Exit => {
                exit.send(AppExit);
            }
        }
    }
}
