use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{mpsc, Mutex, Arc};
use std::thread;

use crate::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::Editor;

pub type ConsoleEditor = Editor<(), MemHistory>;

pub struct CommanderReadlinePlugin<S> {
    pub editor: Mutex<Option<ConsoleEditor>>,
    pub prompt: String,
    pub marker: PhantomData<S>,
}

impl<S> Default for CommanderReadlinePlugin<S> {
    fn default() -> Self {
        let editor = Editor::with_history(rustyline::Config::default(), MemHistory::new())
            .unwrap_or_else(|e| {
                error!("Could not create console command input: {}", e);
                panic!();
            });
        Self {
            editor: Mutex::new(Some(editor)),
            prompt: "".to_owned(),
            marker: PhantomData::default(),
        }
    }
}

enum ReadlineInput {
    Command(String),
    Exit,
}

pub trait ConsoleCommandSender: CommandSender {
    fn console() -> Self;
}

impl<S: ConsoleCommandSender> Plugin for CommanderReadlinePlugin<S> {
    fn build(&self, app: &mut App) {
        let (tx, rx) = mpsc::channel::<ReadlineInput>();

        let mut editor = self
            .editor
            .lock()
            .expect("could not lock readline plugin editor")
            .take()
            .expect("plugin has already been built and consumed the editor");
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

fn receive_readline<S: ConsoleCommandSender>(
    receiver: NonSend<mpsc::Receiver<ReadlineInput>>,
    mut sent_command: EventWriter<CommandSent<S>>,
    mut exit: EventWriter<AppExit>,
) {
    for input in receiver.try_iter() {
        match input {
            ReadlineInput::Command(command) => {
                let mut args: VecDeque<String> = command
                    .split(" ")
                    .map(|s| s.to_owned())
                    .collect();
                let Some(name) = args.pop_front() else {
                    continue;
                };

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
