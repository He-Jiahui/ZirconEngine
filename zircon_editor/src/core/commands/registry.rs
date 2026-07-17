use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::component::UiValue;

use crate::core::editing::operation::{
    OperationCommandFactoryError, OperationCommandFactoryRegistration,
};
use crate::core::editor_event::EditorEvent;
use crate::core::editor_operation::EditorOperationPath;

use super::{
    defaults::default_workbench_commands, menu::menu_bar_model, menu::menu_model,
    AssetWriteTargetDescriptor, CommandEvalCtx, EditorCommandAction, EditorCommandDescriptor,
    EditorCommandPaletteEntry,
};

/// The only registry for editor command metadata, invocation, discovery, and extensions.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandRegistry {
    commands: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    #[serde(skip)]
    operation_factories: BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
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
        self.commands.insert(command.id().clone(), command);
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
        let operation = command.id().clone();
        self.commands.insert(operation.clone(), command);
        self.operation_factories.insert(operation, factory);
        Ok(())
    }

    pub fn validate_descriptor(
        command: &EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        validate_menu_path(command)?;
        validate_payload_schema_id(command)?;
        validate_asset_write_target(command)
    }

    pub(crate) fn attach_asset_write_target(
        &mut self,
        command_id: &EditorOperationPath,
        target: AssetWriteTargetDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
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
        command.set_asset_write_target(target);
        Self::validate_descriptor(command)
    }

    pub fn commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.commands.values()
    }

    pub fn command<Q>(&self, id: &Q) -> Option<&EditorCommandDescriptor>
    where
        EditorOperationPath: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.commands.get(id)
    }

    pub fn operation_factory(
        &self,
        operation: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.operation_factories.get(operation)
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

    pub fn command_palette_entries(
        &self,
        context: &CommandEvalCtx,
    ) -> Vec<EditorCommandPaletteEntry> {
        self.commands()
            .filter(|descriptor| descriptor.is_enabled(context))
            .map(EditorCommandPaletteEntry::from_descriptor)
            .collect()
    }

    pub fn command_palette_value(&self, context: &CommandEvalCtx) -> UiValue {
        UiValue::Array(
            self.command_palette_entries(context)
                .into_iter()
                .map(|entry| entry.to_ui_value())
                .collect(),
        )
    }

    pub fn menu_bar_model(&self, context: &CommandEvalCtx) -> super::MenuBarModel {
        menu_bar_model(self, context)
    }

    pub fn menu_model(&self, label: &str, context: &CommandEvalCtx) -> Option<super::MenuModel> {
        menu_model(self, label, context)
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandRegistryError {
    DuplicateCommand(EditorOperationPath),
    OperationFactory(OperationCommandFactoryError),
    InvalidCommandMenuPath(String),
    InvalidCommandPayloadSchemaId(String),
    InvalidAssetWriteTargetArgument {
        command_id: EditorOperationPath,
        argument: String,
    },
    ConflictingAssetWriteTarget(EditorOperationPath),
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
            Self::InvalidCommandMenuPath(path) => {
                write!(formatter, "editor command menu path `{path}` is invalid")
            }
            Self::InvalidCommandPayloadSchemaId(schema_id) => write!(
                formatter,
                "editor command payload schema id `{schema_id}` is invalid"
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

fn validate_menu_path(
    descriptor: &EditorCommandDescriptor,
) -> Result<(), EditorCommandRegistryError> {
    if let Some(menu_path) = descriptor.menu_path() {
        let mut segment_count = 0;
        let invalid_segment = menu_path.split('/').any(|segment| {
            segment_count += 1;
            segment.trim().is_empty() || segment.trim() != segment
        });
        if invalid_segment || segment_count < MIN_MENU_PATH_SEGMENTS {
            return Err(EditorCommandRegistryError::InvalidCommandMenuPath(
                menu_path.to_string(),
            ));
        }
    }
    Ok(())
}

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

const MIN_MENU_PATH_SEGMENTS: usize = 2;
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
mod tests {
    #[test]
    fn command_descriptor_validation_streams_path_segments() {
        let source = include_str!("registry.rs");
        let menu_collect = ["split('/')", ".collect::<Vec<_>>()"].concat();
        let schema_collect = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(!source.contains(&menu_collect));
        assert!(!source.contains(&schema_collect));
    }
}
