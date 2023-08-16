use bevy::{ecs::system::SystemParam, prelude::*};
use clap::{CommandFactory, FromArgMatches};
use expedition::Message;

pub trait AppCommand: Send + Sync + CommandFactory + FromArgMatches + 'static {
    fn name() -> &'static str;
}

#[derive(Event)]
pub struct CommandDispatch<C> {
    pub sender: Entity,
    pub data: C,
}

pub enum Outcome {
    Ok,
    Err,
}

#[derive(Event)]
pub struct CommandResponse {
    pub target: Entity,
    pub message: Message,
    pub outcome: Outcome,
}

impl CommandResponse {
    pub fn ok(target: Entity, message: impl Into<Message>) -> Self {
        Self {
            target,
            message: message.into(),
            outcome: Outcome::Ok,
        }
    }

    pub fn err(target: Entity, message: impl Into<Message>) -> Self {
        Self {
            target,
            message: message.into(),
            outcome: Outcome::Err,
        }
    }
}

#[derive(SystemParam)]
pub struct QueuedCommands<'w, 's, C: AppCommand> {
    commands: EventReader<'w, 's, CommandDispatch<C>>,
    responses: EventWriter<'w, CommandResponse>,
}

impl<C: AppCommand> QueuedCommands<'_, '_, C> {
    pub fn consume<F>(&mut self, mut consume: F)
    where
        F: FnMut(CommandContext<C>),
    {
        for event in &mut self.commands {
            consume(CommandContext {
                sender: event.sender,
                data: &event.data,
                responses: &mut self.responses,
            })
        }
    }
}

pub struct CommandContext<'a, 'w, C: AppCommand> {
    pub sender: Entity,
    pub data: &'a C,
    responses: &'a mut EventWriter<'w, CommandResponse>,
}

impl<C: AppCommand> CommandContext<'_, '_, C> {
    pub fn respond(&mut self, outcome: Outcome, message: impl Into<Message>) {
        self.responses.send(CommandResponse {
            target: self.sender,
            message: message.into(),
            outcome,
        });
    }

    pub fn ok(&mut self, message: impl Into<Message>) {
        self.respond(Outcome::Ok, message)
    }

    pub fn err(&mut self, message: impl Into<Message>) {
        self.respond(Outcome::Err, message)
    }
}
