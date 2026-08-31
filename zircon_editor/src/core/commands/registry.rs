use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};

use crate::core::editing::operation::{
    OperationCommandFactoryError, OperationCommandFactoryRegistration,
};
use crate::core::editor_event::EditorEvent;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale};

use super::{
    defaults::default_workbench_commands, menu::menu_bar_model, menu::menu_model,
    AssetWriteTargetDescriptor, CommandEvalCtx, EditorCommandAction, EditorCommandDescriptor,
    EditorCommandExecutorRegistry, EditorCommandExecutorRegistryError, EditorCommandPaletteCatalog,
    NativePluginEditorCommandBinding,
};

/// The only registry for editor command metadata, invocation, discovery, and extensions.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EditorCommandRegistry {
    commands: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    #[serde(skip)]
    operation_factories: BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
    generation: u64,
    #[serde(skip)]
    palette_catalog: OnceLock<Arc<EditorCommandPaletteCatalog>>,
    #[serde(skip)]
    executors: EditorCommandExecutorRegistry,
}

impl Clone for EditorCommandRegistry {
    fn clone(&self) -> Self {
        Self {
            commands: self.commands.clone(),
            operation_factories: self.operation_factories.clone(),
            generation: self.generation,
            palette_catalog: OnceLock::new(),
            executors: EditorCommandExecutorRegistry::default(),
        }
    }
}

impl PartialEq for EditorCommandRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.commands == other.commands
            && self.operation_factories == other.operation_factories
            && self.generation == other.generation
    }
}

impl EditorCommandRegistry {
    pub fn new(commands: Vec<EditorCommandDescriptor>) -> Result<Self, EditorCommandRegistryError> {
        let mut registry = Self::default();
        for command in commands {
            registry.register(command)?;
        }
        Ok(registry)
    }

    pub fn default_workbench() -> Self {
        Self::new(default_workbench_commands()).expect("default editor command ids are unique")
    }

    pub fn register(
        &mut self,
        command: EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        if self.commands.contains_key(command.id()) {
            return Err(EditorCommandRegistryError::DuplicateCommand(
                command.id().clone(),
            ));
        }
        Self::validate_descriptor(&command)?;
        self.ensure_headless_commandlet_route_available(&command)?;
        self.ensure_headless_commandlet_name_available(&command)?;
        self.commands.insert(command.id().clone(), command);
        self.advance_generation();
        Ok(())
    }

    pub fn register_operation(
        &mut self,
        command: EditorCommandDescriptor,
        factory: OperationCommandFactoryRegistration,
    ) -> Result<(), EditorCommandRegistryError> {
        if command.id() != factory.operation() {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::OperationMismatch {
                    descriptor_operation: command.id().clone(),
                    factory_operation: factory.operation().clone(),
                },
            ));
        }
        if !matches!(command.action(), EditorCommandAction::Operation) {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DescriptorIsEvent {
                    operation: command.id().clone(),
                },
            ));
        }
        if self.commands.contains_key(command.id()) {
            return Err(EditorCommandRegistryError::DuplicateCommand(
                command.id().clone(),
            ));
        }
        if self.operation_factories.contains_key(factory.operation()) {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DuplicateFactory {
                    operation: factory.operation().clone(),
                },
            ));
        }
        Self::validate_descriptor(&command)?;
        self.ensure_headless_commandlet_route_available(&command)?;
        self.ensure_headless_commandlet_name_available(&command)?;
        let operation = command.id().clone();
        self.commands.insert(operation.clone(), command);
        self.operation_factories.insert(operation, factory);
        self.advance_generation();
        Ok(())
    }

    pub fn validate_descriptor(
        command: &EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        validate_payload_schema_id(command)?;
        validate_headless_commandlet(command)?;
        validate_asset_write_target(command)?;
        if let Some(contract) = command.execution_contract() {
            contract.validate().map_err(|error| {
                EditorCommandRegistryError::InvalidExecutionContract {
                    command_id: command.id().clone(),
                    detail: error.to_string(),
                }
            })?;
        } else if matches!(command.action(), EditorCommandAction::NativeEndpoint) {
            return Err(EditorCommandRegistryError::InvalidExecutionContract {
                command_id: command.id().clone(),
                detail: "native endpoint commands require an execution contract".to_owned(),
            });
        }
        Ok(())
    }

    pub(crate) fn attach_asset_write_target(
        &mut self,
        command_id: &EditorOperationPath,
        target: AssetWriteTargetDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        {
            let command = self
                .commands
                .get_mut(command_id)
                .ok_or_else(|| EditorCommandRegistryError::MissingCommand(command_id.clone()))?;
            if let Some(existing) = command.asset_write_target() {
                if existing != &target {
                    return Err(EditorCommandRegistryError::ConflictingAssetWriteTarget(
                        command_id.clone(),
                    ));
                }
                return Ok(());
            }
            let mut updated = command.clone();
            updated.set_asset_write_target(target);
            Self::validate_descriptor(&updated)?;
            *command = updated;
        }
        self.advance_generation();
        Ok(())
    }

    pub fn commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.commands.values()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Publishes a freshly materialized registry after the preceding live generation.
    ///
    /// Projection builders register descriptors into a private registry before publication, so
    /// their construction mutation count is not a stable catalog revision. Different projections
    /// with the same number of descriptors must still publish distinct, monotonic generations.
    pub(crate) fn publish_projection_after(&mut self, previous_generation: u64) {
        self.generation = previous_generation
            .checked_add(1)
            .expect("editor command registry generation overflowed");
        let _ = self.palette_catalog.take();
    }

    pub fn command<Q>(&self, id: &Q) -> Option<&EditorCommandDescriptor>
    where
        EditorOperationPath: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.commands.get(id)
    }

    pub fn command_for_headless_commandlet_route(
        &self,
        route: &EditorOperationPath,
    ) -> Option<&EditorCommandDescriptor> {
        self.commands()
            .find(|descriptor| descriptor.headless_commandlet_route() == Some(route))
    }

    pub fn command_for_headless_commandlet_name(
        &self,
        name: &str,
    ) -> Option<&EditorCommandDescriptor> {
        self.commands()
            .find(|descriptor| descriptor.headless_commandlet_name() == Some(name))
    }

    pub fn operation_factory(
        &self,
        operation: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.operation_factories.get(operation)
    }

    pub fn register_native_executor(
        &mut self,
        command_id: &EditorOperationPath,
        binding: NativePluginEditorCommandBinding,
    ) -> Result<(), EditorCommandExecutorRegistryError> {
        let descriptor = self.command(command_id).cloned().ok_or_else(|| {
            EditorCommandExecutorRegistryError::MissingCommand {
                command_id: command_id.clone(),
            }
        })?;
        self.executors.register_native(&descriptor, binding)
    }

    pub fn native_executor(
        &self,
        command_id: &EditorOperationPath,
    ) -> Option<&super::NativeCommandExecutorRegistration> {
        self.executors.get(command_id)
    }

    pub fn revoke_native_executor(&mut self, command_id: &EditorOperationPath) -> bool {
        self.executors.revoke(command_id)
    }

    pub fn native_executor_count(&self) -> usize {
        self.executors.len()
    }

    pub fn invoke_native_executor(
        &self,
        command_id: &EditorOperationPath,
        payload: &[u8],
    ) -> Result<super::EditorCommandExecutionReceipt, EditorCommandExecutorRegistryError> {
        self.executors
            .get(command_id)
            .ok_or_else(|| EditorCommandExecutorRegistryError::MissingExecutor {
                command_id: command_id.clone(),
            })
            .map(|executor| executor.invoke(payload))
    }

    pub fn descriptor_for_event(&self, event: &EditorEvent) -> Option<&EditorCommandDescriptor> {
        self.commands()
            .find(|descriptor| descriptor.event() == Some(event))
    }

    pub fn event_for_command(
        &self,
        id: &str,
        context: &CommandEvalCtx,
    ) -> Result<EditorEvent, EditorCommandDispatchError> {
        let descriptor = self
            .command(id)
            .ok_or_else(|| EditorCommandDispatchError::UnknownCommand(id.to_string()))?;
        Self::ensure_enabled(descriptor, context)?;
        descriptor.event().cloned().ok_or_else(|| {
            EditorCommandDispatchError::OperationRequiresInvocation {
                command_id: descriptor.id().clone(),
            }
        })
    }

    pub fn command_palette_catalog(&self) -> Arc<EditorCommandPaletteCatalog> {
        Arc::clone(self.palette_catalog.get_or_init(|| {
            Arc::new(EditorCommandPaletteCatalog::from_descriptors(
                self.generation,
                self.commands(),
            ))
        }))
    }

    pub fn menu_bar_model(
        &self,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        context: &CommandEvalCtx,
    ) -> super::MenuBarModel {
        menu_bar_model(self, i18n, locale, context)
    }

    pub fn menu_model(
        &self,
        root_id: &str,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        context: &CommandEvalCtx,
    ) -> Option<super::MenuModel> {
        menu_model(self, root_id, i18n, locale, context)
    }

    pub fn ensure_enabled(
        descriptor: &EditorCommandDescriptor,
        context: &CommandEvalCtx,
    ) -> Result<(), EditorCommandDispatchError> {
        if descriptor.is_enabled(context) {
            return Ok(());
        }
        let missing = descriptor.missing_required_capabilities(context);
        if missing.is_empty() {
            Err(EditorCommandDispatchError::DisabledByWhen {
                command_id: descriptor.id().clone(),
            })
        } else {
            Err(EditorCommandDispatchError::MissingCapabilities {
                command_id: descriptor.id().clone(),
                capabilities: missing,
            })
        }
    }

    pub fn missing_default_keymap_bindings<'a>(
        &'a self,
        keymap: &'a super::EditorKeymap,
    ) -> Vec<&'a str> {
        let keymap_commands = keymap
            .bindings()
            .iter()
            .map(|binding| binding.command_id())
            .collect::<BTreeSet<_>>();
        self.commands()
            .filter(|descriptor| descriptor.default_chord().is_some())
            .map(|descriptor| descriptor.id().as_str())
            .filter(|id| !keymap_commands.contains(id))
            .collect()
    }

    fn advance_generation(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("editor command registry generation overflowed");
        let _ = self.palette_catalog.take();
    }

    fn ensure_headless_commandlet_route_available(
        &self,
        command: &EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        let Some(route) = command.headless_commandlet_route() else {
            return Ok(());
        };
        if self
            .commands()
            .any(|existing| existing.headless_commandlet_route() == Some(route))
        {
            return Err(
                EditorCommandRegistryError::DuplicateHeadlessCommandletRoute(route.clone()),
            );
        }
        Ok(())
    }

    fn ensure_headless_commandlet_name_available(
        &self,
        command: &EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        let Some(name) = command.headless_commandlet_name() else {
            return Ok(());
        };
        if self
            .commands()
            .any(|existing| existing.headless_commandlet_name() == Some(name))
        {
            return Err(EditorCommandRegistryError::DuplicateHeadlessCommandletName(
                name.to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandRegistryError {
    DuplicateCommand(EditorOperationPath),
    OperationFactory(OperationCommandFactoryError),
    Executor(EditorCommandExecutorRegistryError),
    InvalidCommandPayloadSchemaId(String),
    HeadlessCommandletMissingRoute(EditorOperationPath),
    HeadlessCommandletMissingName(EditorOperationPath),
    HeadlessCommandletMissingPayloadSchema(EditorOperationPath),
    HeadlessCommandletNotCallableFromRemote(EditorOperationPath),
    HeadlessRouteOnNonHeadlessCommand(EditorOperationPath),
    HeadlessNameOnNonHeadlessCommand(EditorOperationPath),
    DuplicateHeadlessCommandletRoute(EditorOperationPath),
    DuplicateHeadlessCommandletName(String),
    InvalidHeadlessCommandletName(String),
    InvalidAssetWriteTargetArgument {
        command_id: EditorOperationPath,
        argument: String,
    },
    ConflictingAssetWriteTarget(EditorOperationPath),
    InvalidExecutionContract {
        command_id: EditorOperationPath,
        detail: String,
    },
    MissingCommand(EditorOperationPath),
    CommandNotCallableFromRemote(EditorOperationPath),
}

impl std::fmt::Display for EditorCommandRegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCommand(id) => {
                write!(formatter, "editor command {id} already registered")
            }
            Self::OperationFactory(error) => error.fmt(formatter),
            Self::Executor(error) => error.fmt(formatter),
            Self::InvalidCommandPayloadSchemaId(schema_id) => write!(
                formatter,
                "editor command payload schema id `{schema_id}` is invalid"
            ),
            Self::HeadlessCommandletMissingRoute(command_id) => write!(
                formatter,
                "headless editor commandlet {command_id} has no typed route"
            ),
            Self::HeadlessCommandletMissingName(command_id) => write!(
                formatter,
                "headless editor commandlet {command_id} has no CLI name"
            ),
            Self::HeadlessCommandletMissingPayloadSchema(command_id) => write!(
                formatter,
                "headless editor commandlet {command_id} has no payload schema"
            ),
            Self::HeadlessCommandletNotCallableFromRemote(command_id) => write!(
                formatter,
                "headless editor commandlet {command_id} is not callable from remote control"
            ),
            Self::HeadlessRouteOnNonHeadlessCommand(command_id) => write!(
                formatter,
                "editor command {command_id} declares a headless route without a headless action"
            ),
            Self::HeadlessNameOnNonHeadlessCommand(command_id) => write!(
                formatter,
                "editor command {command_id} declares a headless CLI name without a headless action"
            ),
            Self::DuplicateHeadlessCommandletRoute(route) => write!(
                formatter,
                "headless editor commandlet route {route} is already registered"
            ),
            Self::DuplicateHeadlessCommandletName(name) => write!(
                formatter,
                "headless editor commandlet name `{name}` is already registered"
            ),
            Self::InvalidHeadlessCommandletName(name) => write!(
                formatter,
                "headless editor commandlet name `{name}` is invalid"
            ),
            Self::InvalidAssetWriteTargetArgument {
                command_id,
                argument,
            } => write!(
                formatter,
                "editor command {command_id} has invalid asset write target argument `{argument}`"
            ),
            Self::ConflictingAssetWriteTarget(command_id) => write!(
                formatter,
                "editor command {command_id} has conflicting asset write targets"
            ),
            Self::InvalidExecutionContract { command_id, detail } => write!(
                formatter,
                "editor command {command_id} has an invalid execution contract: {detail}"
            ),
            Self::MissingCommand(id) => write!(formatter, "editor command {id} is not registered"),
            Self::CommandNotCallableFromRemote(id) => write!(
                formatter,
                "editor command {id} is not callable from remote control"
            ),
        }
    }
}

impl std::error::Error for EditorCommandRegistryError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandDispatchError {
    UnknownCommand(String),
    DisabledByWhen {
        command_id: EditorOperationPath,
    },
    MissingCapabilities {
        command_id: EditorOperationPath,
        capabilities: Vec<String>,
    },
    OperationRequiresInvocation {
        command_id: EditorOperationPath,
    },
}

impl std::fmt::Display for EditorCommandDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCommand(id) => write!(formatter, "editor command `{id}` does not exist"),
            Self::DisabledByWhen { command_id } => {
                write!(
                    formatter,
                    "editor command {command_id} is disabled by its when clause"
                )
            }
            Self::MissingCapabilities {
                command_id,
                capabilities,
            } => write!(
                formatter,
                "editor command {command_id} requires disabled capabilities: {}",
                capabilities.join(", ")
            ),
            Self::OperationRequiresInvocation { command_id } => write!(
                formatter,
                "editor command {command_id} must be invoked through the operation dispatcher"
            ),
        }
    }
}

impl std::error::Error for EditorCommandDispatchError {}

fn validate_payload_schema_id(
    descriptor: &EditorCommandDescriptor,
) -> Result<(), EditorCommandRegistryError> {
    if let Some(schema_id) = descriptor.payload_schema_id() {
        let mut segment_count = 0;
        let valid = schema_id.split('.').all(|segment| {
            segment_count += 1;
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|value| value.is_ascii_alphanumeric() || value == '_' || value == '-')
        });
        if !valid || segment_count < MIN_PAYLOAD_SCHEMA_SEGMENTS {
            return Err(EditorCommandRegistryError::InvalidCommandPayloadSchemaId(
                schema_id.to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_headless_commandlet(
    descriptor: &EditorCommandDescriptor,
) -> Result<(), EditorCommandRegistryError> {
    let headless_action = matches!(
        descriptor.action(),
        EditorCommandAction::HeadlessAssetMigration
            | EditorCommandAction::HeadlessPluginList
            | EditorCommandAction::HeadlessAuthoringAutomation
    );
    if !headless_action {
        if descriptor.headless_commandlet_route().is_some() {
            return Err(
                EditorCommandRegistryError::HeadlessRouteOnNonHeadlessCommand(
                    descriptor.id().clone(),
                ),
            );
        }
        if descriptor.headless_commandlet_name().is_some() {
            return Err(
                EditorCommandRegistryError::HeadlessNameOnNonHeadlessCommand(
                    descriptor.id().clone(),
                ),
            );
        }
        return Ok(());
    }
    if descriptor.headless_commandlet_route().is_none() {
        return Err(EditorCommandRegistryError::HeadlessCommandletMissingRoute(
            descriptor.id().clone(),
        ));
    }
    let Some(name) = descriptor.headless_commandlet_name() else {
        return Err(EditorCommandRegistryError::HeadlessCommandletMissingName(
            descriptor.id().clone(),
        ));
    };
    if !valid_headless_commandlet_name(name) {
        return Err(EditorCommandRegistryError::InvalidHeadlessCommandletName(
            name.to_string(),
        ));
    }
    if descriptor.payload_schema_id().is_none() {
        return Err(
            EditorCommandRegistryError::HeadlessCommandletMissingPayloadSchema(
                descriptor.id().clone(),
            ),
        );
    }
    if !descriptor.callable_from_remote() {
        return Err(
            EditorCommandRegistryError::HeadlessCommandletNotCallableFromRemote(
                descriptor.id().clone(),
            ),
        );
    }
    Ok(())
}

fn valid_headless_commandlet_name(name: &str) -> bool {
    name.split('-').all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
    })
}

const MIN_PAYLOAD_SCHEMA_SEGMENTS: usize = 3;

fn validate_asset_write_target(
    descriptor: &EditorCommandDescriptor,
) -> Result<(), EditorCommandRegistryError> {
    let Some(target) = descriptor.asset_write_target() else {
        return Ok(());
    };
    for argument in [target.asset_type_argument(), target.locator_argument()] {
        let valid = !argument.is_empty()
            && argument.chars().enumerate().all(|(index, value)| {
                value == '_' || value.is_ascii_lowercase() || (index > 0 && value.is_ascii_digit())
            });
        if !valid {
            return Err(
                EditorCommandRegistryError::InvalidAssetWriteTargetArgument {
                    command_id: descriptor.id().clone(),
                    argument: argument.to_owned(),
                },
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
