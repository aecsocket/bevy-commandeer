use bevy::{prelude::*, utils::HashMap};

use crate::{AppCommand, CommandDispatch, CommandResponse};

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CommandMetaMap(HashMap::default()))
            .insert_resource(RespondToInvalidCommand(true))
            .add_event::<CommandBufInput>()
            .add_event::<CommandArgsInput>()
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
            .add_systems(Update, (parse_command_bufs).before(CommandSet::Dispatch))
            .add_systems(
                Update,
                (mark_invalid_commands)
                    .after(CommandSet::Dispatch)
                    .before(CommandSet::Response),
            )
            .add_systems(
                Update,
                (invalid_command_response)
                    .after(CommandSet::Response)
                    .run_if(respond_to_invalid_command),
            );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum CommandSet {
    Dispatch,
    Process,
    Response,
}

fn have_commands(cmds: EventReader<CommandBufInput>) -> bool {
    !cmds.is_empty()
}

fn have_responses(resps: EventReader<CommandResponse>) -> bool {
    !resps.is_empty()
}

#[derive(Resource)]
pub struct RespondToInvalidCommand(pub bool);

fn respond_to_invalid_command(res: Res<RespondToInvalidCommand>) -> bool {
    res.0
}

#[derive(Resource)]
pub struct CommandMetaMap(pub HashMap<&'static str, clap::Command>);

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
            if command_meta.0.insert(C::name(), command::<C>()).is_some() {
                warn!("Command '{}' already exists, overwriting", C::name());
            }
        };

        let dispatch_command =
            move |mut input: EventReader<CommandArgsInput>,
                  mut dispatch: EventWriter<CommandDispatch<C>>,
                  mut resps: EventWriter<CommandResponse>| {
                for input in input.iter().filter(|input| input.name == C::name()) {
                    debug!("Dispatching '{}' sent by {:?}", input.name, input.sender);
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
pub struct CommandBufInput {
    pub sender: Entity,
    pub buf: String,
}

#[derive(Event)]
pub struct CommandArgsInput {
    pub sender: Entity,
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Event)]
pub struct InvalidCommandInput {
    pub target: Entity,
    pub name: String,
}

fn parse_command_bufs(
    mut buf_input: EventReader<CommandBufInput>,
    mut args_input: EventWriter<CommandArgsInput>,
) {
    args_input.send_batch(buf_input.iter().filter_map(|input| {
        let mut args = shlex::split(&input.buf).unwrap_or_default();
        if args.is_empty() {
            return None;
        }
        let name = args.remove(0);
        Some(CommandArgsInput {
            sender: input.sender,
            name,
            args,
        })
    }));
}

fn mark_invalid_commands(
    mut input: EventReader<CommandArgsInput>,
    command_meta: Res<CommandMetaMap>,
    mut invalid: EventWriter<InvalidCommandInput>,
) {
    for input in input
        .iter()
        .filter(|input| !command_meta.0.contains_key(input.name.as_str()))
    {
        debug!(
            "Marking '{}' sent by {:?} as invalid",
            input.name, input.sender
        );
        invalid.send(InvalidCommandInput {
            target: input.sender,
            name: input.name.clone(),
        })
    }
}

fn invalid_command_response(
    mut events: EventReader<InvalidCommandInput>,
    mut resps: EventWriter<CommandResponse>,
) {
    for event in events.iter() {
        resps.send(CommandResponse::err(
            event.target,
            format!("No such command: {}", event.name),
        ));
    }
}
