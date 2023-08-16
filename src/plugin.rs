use bevy::{prelude::*, utils::HashMap};

use crate::{AppCommand, CommandDispatch, CommandResponse};

pub struct CommandsPlugin {
    pub invalid_command_response: bool,
}

impl Default for CommandsPlugin {
    fn default() -> Self {
        Self {
            invalid_command_response: true,
        }
    }
}

impl CommandsPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CommandMetaMap(HashMap::default()))
            .add_event::<CommandInput>()
            .add_event::<CommandResponse>()
            .add_event::<InvalidCommandInput>()
            .configure_sets(
                Update,
                (
                    CommandSet::Dispatch,
                    CommandSet::Process,
                    CommandSet::Response,
                )
                    .chain(),
            )
            .configure_sets(
                Update,
                (
                    CommandSet::Process.run_if(have_commands),
                    CommandSet::Response.run_if(have_responses),
                ),
            )
            .add_systems(
                Update,
                (mark_invalid_commands)
                    .after(CommandSet::Dispatch)
                    .before(CommandSet::Response),
            );

        if self.invalid_command_response {
            app.add_systems(
                Update,
                (invalid_command_responses).in_set(CommandSet::Response),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum CommandSet {
    Dispatch,
    Process,
    Response,
}

fn have_commands(cmds: EventReader<CommandInput>) -> bool {
    !cmds.is_empty()
}

fn have_responses(resps: EventReader<CommandResponse>) -> bool {
    !resps.is_empty()
}

#[derive(Resource)]
pub struct CommandMetaMap(HashMap<&'static str, clap::Command>);

pub trait AddAppCommand {
    fn add_app_command<C: AppCommand, M>(&mut self, system: impl IntoSystemConfigs<M>)
        -> &mut Self;
}

impl AddAppCommand for App {
    fn add_app_command<C: AppCommand, M>(
        &mut self,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        fn command<C: AppCommand>() -> clap::Command {
            C::command().no_binary_name(true)
        }

        let setup_command_meta = move |mut command_meta: ResMut<CommandMetaMap>| {
            if let Some(_) = command_meta.0.insert(C::name(), command::<C>()) {
                warn!("Command '{}' already exists, overwriting", C::name());
            }
        };

        let dispatch_command =
            move |mut input: EventReader<CommandInput>,
                  mut dispatch: EventWriter<CommandDispatch<C>>,
                  mut resps: EventWriter<CommandResponse>| {
                for input in input.iter().filter(|input| input.name == C::name()) {
                    match command::<C>()
                        .clone()
                        .try_get_matches_from(input.args.iter())
                        .and_then(|matches| C::from_arg_matches(&matches))
                    {
                        Ok(data) => dispatch.send(CommandDispatch {
                            sender: input.sender,
                            data,
                        }),
                        Err(e) => {
                            resps.send_batch(
                                e.render()
                                    .to_string()
                                    .lines()
                                    .map(|s| CommandResponse::err(input.sender, s)),
                            );
                        }
                    }
                }
            };

        self.add_event::<CommandDispatch<C>>()
            .add_systems(Startup, setup_command_meta)
            .add_systems(Update, (dispatch_command).in_set(CommandSet::Dispatch))
            .add_systems(Update, systems.in_set(CommandSet::Process))
    }
}

#[derive(Event)]
pub struct CommandInput {
    pub sender: Entity,
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Event)]
pub struct InvalidCommandInput {
    pub target: Entity,
    pub name: String,
}

fn mark_invalid_commands(
    mut input: EventReader<CommandInput>,
    command_meta: Res<CommandMetaMap>,
    mut invalid: EventWriter<InvalidCommandInput>,
) {
    for input in input
        .iter()
        .filter(|input| !command_meta.0.contains_key(input.name.as_str()))
    {
        invalid.send(InvalidCommandInput {
            target: input.sender,
            name: input.name.clone(),
        })
    }
}

fn invalid_command_responses(
    mut events: EventReader<InvalidCommandInput>,
    mut resps: EventWriter<CommandResponse>,
) {
    for event in events.iter() {
        resps.send(CommandResponse::err(
            event.target,
            format!("no such command: {}", event.name),
        ));
    }
}
