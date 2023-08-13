use std::any::Any;
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

pub trait CommandSender: Any + Send + Sync {
    fn send_all<'a>(&self, lines: impl IntoIterator<Item = &'a str>);

    fn send(&self, line: &str) {
        self.send_all([line])
    }
}

pub struct CommandContext<C, S>(Vec<(C, Arc<S>)>);

impl<C, S> IntoIterator for CommandContext<C, S> {
    type Item = (C, Arc<S>);
    type IntoIter = std::vec::IntoIter<(C, Arc<S>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// internals: implementing SystemParam

type CommandSentReader<S> = EventReader<'static, 'static, CommandSent<S>>;

unsafe impl<C: AppCommand, S: CommandSender> SystemParam for CommandContext<C, S> {
    type State = CommandContextState<C, S>;
    type Item<'w, 's> = CommandContext<C, S>;

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

        let buf: Vec<(C, Arc<S>)> = event_reader
            .iter()
            .filter_map(|event| {
                if C::name() != event.name {
                    return None;
                }
                let command = C::command()
                    .no_binary_name(true)
                    .name(C::name())
                    .color(clap::ColorChoice::Never);
                
                fn send_error<S: CommandSender>(sender: &S, e: impl ToString) {
                    sender.send_all(e.to_string().lines());
                }

                match command.try_get_matches_from(event.args.iter()) {
                    Ok(matches) => {
                        match C::from_arg_matches(&matches) {
                            Ok(c) => Some((c, event.sender.clone())),
                            Err(e) => {
                                send_error(event.sender.as_ref(), e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        send_error(event.sender.as_ref(), e);
                        None
                    },
                }
            })
            .collect();

        CommandContext(buf)
    }
}

pub struct CommandContextState<C, S: CommandSender> {
    event_reader: <CommandSentReader<S> as SystemParam>::State,
    marker: PhantomData<C>,
}
