use std::marker::PhantomData;
use std::sync::Arc;

use bevy::ecs::component::Tick;
use bevy::ecs::system::{SystemMeta, SystemParam};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use clap::{CommandFactory, FromArgMatches};

use crate::prelude::*;

pub trait AppCommand: CommandFactory + FromArgMatches + Sized + Resource {
    fn name() -> &'static str;
}

pub trait CommandSender: Send + Sync {
    fn send(&self, line: &str);
}

pub type BoxedSender = Arc<dyn CommandSender>;

pub struct ConsoleCommandSender;

impl CommandSender for ConsoleCommandSender {
    fn send(&self, line: &str) {
        info!("{}", line);
    }
}

pub struct CommandContext<C>(Vec<(C, BoxedSender)>);

impl<C> IntoIterator for CommandContext<C> {
    type Item = (C, BoxedSender);
    type IntoIter = std::vec::IntoIter<(C, BoxedSender)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// internals: implementing SystemParam

type CommandSentReader = EventReader<'static, 'static, CommandSent>;

unsafe impl<C: AppCommand> SystemParam for CommandContext<C> {
    type State = CommandContextState<C>;
    type Item<'w, 's> = CommandContext<C>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        let event_reader = CommandSentReader::init_state(world, system_meta);
        CommandContextState {
            event_reader,
            marker: PhantomData::default(),
        }
    }

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'w>,
        change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        let mut event_reader =
            CommandSentReader::get_param(&mut state.event_reader, system_meta, world, change_tick);

        let buf: Vec<(C, BoxedSender)> = event_reader
            .iter()
            .filter_map(|event| {
                if C::name() != event.name {
                    return None;
                }
                let command = C::command()
                    .no_binary_name(true)
                    .name(C::name())
                    .color(clap::ColorChoice::Never);
                
                fn send_error(sender: &BoxedSender, e: impl ToString) {
                    for line in e.to_string().lines() {
                        sender.send(line);
                    }
                }

                match command.try_get_matches_from(event.args.iter()) {
                    Ok(matches) => {
                        match C::from_arg_matches(&matches) {
                            Ok(c) => Some((c, event.sender.clone())),
                            Err(e) => {
                                send_error(&event.sender, e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        send_error(&event.sender, e);
                        None
                    },
                }
            })
            .collect();

        CommandContext(buf)
    }
}

pub struct CommandContextState<C> {
    event_reader: <CommandSentReader as SystemParam>::State,
    marker: PhantomData<C>,
}
