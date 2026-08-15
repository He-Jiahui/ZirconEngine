use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};

use crate::core::editing::operation::{
    OperationCommandFactoryError, OperationCommandFactoryRegistration,
};
use crate::core::editor_event::EditorEvent;
use crate::core::editor_operation::EditorOperationPath;

use super::{
    AssetWriteTargetDescriptor, CommandEvalCtx, EditorCommandAction, EditorCommandDescriptor,
    EditorCommandPaletteCatalog, defaults::default_workbench_commands, menu::menu_bar_model,
    menu::menu_model,
};

/// The only registry for editor command metadata, invocation, discovery, and extensions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EditorCommandRegistry {
    commands: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    #[serde(skip)]
    operation_factories: BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
    generation: u64,
    #[serde(skip)]
    palette_catalog: OnceLock<Arc<EditorCommandPaletteCatalog>>,
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
        validate_menu_path(command)?;
        validate_payload_schema_id(command)?;
        validate_headless_commandlet(command)?;
        validate_asset_write_target(command)
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
    InvalidCommandMenuPath(String),
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
    use std::sync::Arc;
    use std::time::Instant;

    use crate::core::asset::AssetWriteAccess;
    use crate::core::editor_event::{EditorEvent, EditorEventTransient};
    use crate::core::editor_operation::EditorOperationPath;

    use super::super::{
        CommandEvalCtx, EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
        EditorCommandPaletteMru, WhenClause,
    };
    use super::{EditorCommandRegistry, EditorCommandRegistryError};

    #[test]
    fn command_descriptor_validation_streams_path_segments() {
        let source = include_str!("registry.rs");
        let menu_collect = ["split('/')", ".collect::<Vec<_>>()"].concat();
        let schema_collect = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(!source.contains(&menu_collect));
        assert!(!source.contains(&schema_collect));
    }

    #[test]
    fn headless_commandlets_require_unique_typed_routes() {
        let descriptor = EditorCommandDescriptor::new(
            EditorOperationPath::parse("test.commandlet.plugin_list")
                .expect("test command id should be valid"),
            "List Plugins",
            EditorCommandCategory::Command,
            EditorCommandAction::HeadlessPluginList,
        )
        .with_payload_schema_id("editor.commandlet.plugin-list")
        .with_required_capabilities(["plugin.catalog.read"]);
        let mut registry = EditorCommandRegistry::default();

        assert!(matches!(
            registry.register(descriptor.clone()),
            Err(EditorCommandRegistryError::HeadlessCommandletMissingRoute(
                _
            ))
        ));

        let route = EditorOperationPath::parse("commandlet.route.plugin_list")
            .expect("test commandlet route should be valid");
        let descriptor = descriptor.with_headless_commandlet_route(route.clone());
        assert!(matches!(
            registry.register(descriptor.clone()),
            Err(EditorCommandRegistryError::HeadlessCommandletMissingName(_))
        ));
        let descriptor = descriptor.with_headless_commandlet_name("plugin-list");
        registry
            .register(descriptor.clone())
            .expect("a routed headless commandlet should register");
        assert_eq!(
            registry
                .command_for_headless_commandlet_route(&route)
                .map(EditorCommandDescriptor::id),
            Some(descriptor.id())
        );

        let duplicate = EditorCommandDescriptor::new(
            EditorOperationPath::parse("test.commandlet.plugin_list_copy")
                .expect("test command id should be valid"),
            "List Plugins Copy",
            EditorCommandCategory::Command,
            EditorCommandAction::HeadlessPluginList,
        )
        .with_payload_schema_id("editor.commandlet.plugin-list-copy")
        .with_required_capabilities(["plugin.catalog.read"])
        .with_headless_commandlet_route(route.clone())
        .with_headless_commandlet_name("plugin-list-copy");

        assert!(matches!(
            registry.register(duplicate),
            Err(EditorCommandRegistryError::DuplicateHeadlessCommandletRoute(route_error))
                if route_error == route
        ));

        let duplicate_name = EditorCommandDescriptor::new(
            EditorOperationPath::parse("test.commandlet.plugin_list_name_copy")
                .expect("test command id should be valid"),
            "List Plugins Name Copy",
            EditorCommandCategory::Command,
            EditorCommandAction::HeadlessPluginList,
        )
        .with_payload_schema_id("editor.commandlet.plugin-list-name-copy")
        .with_required_capabilities(["plugin.catalog.read"])
        .with_headless_commandlet_route(
            EditorOperationPath::parse("commandlet.route.plugin_list_name_copy")
                .expect("test commandlet route should be valid"),
        )
        .with_headless_commandlet_name("plugin-list");

        assert!(matches!(
            registry.register(duplicate_name),
            Err(EditorCommandRegistryError::DuplicateHeadlessCommandletName(name))
                if name == "plugin-list"
        ));
    }

    #[test]
    fn stable_command_catalog_is_shared_until_registry_generation_changes() {
        let mut registry = EditorCommandRegistry::default_workbench();

        let first = registry.command_palette_catalog();
        let second = registry.command_palette_catalog();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.generation(), registry.generation());

        registry
            .register(test_command(10_000))
            .expect("new command should advance the catalog generation");
        let next = registry.command_palette_catalog();

        assert!(!Arc::ptr_eq(&first, &next));
        assert_eq!(next.generation(), first.generation() + 1);
        assert_eq!(next.len(), first.len() + 1);
    }

    #[test]
    fn palette_query_retains_only_the_requested_window_without_truncating_matches() {
        let mut registry = EditorCommandRegistry::default();
        for index in 0..1_000 {
            registry
                .register(test_command(index))
                .expect("generated command ids should be unique");
        }

        let catalog = registry.command_palette_catalog();
        let window =
            catalog.query_window(&CommandEvalCtx::interactive(), "palette command", 480, 24);

        assert_eq!(window.total_match_count(), 1_000);
        assert_eq!(window.offset(), 480);
        assert_eq!(window.len(), 24);
        assert_eq!(window.metrics().retained_handles, 24);
        assert_eq!(window.metrics().visited_entries, 1_000);
        assert_eq!(window.metrics().enablement_evaluations, 1_000);
        assert_eq!(window.metrics().candidate_handles, 504);
        assert_eq!(window.metrics().owned_buffers, 4);
        assert_eq!(
            window.entries().next().map(|entry| entry.id.as_str()),
            Some("test.palette.command_0480")
        );
        assert_eq!(
            window.entries().last().map(|entry| entry.id.as_str()),
            Some("test.palette.command_0503")
        );
    }

    #[test]
    fn palette_mru_precedes_catalog_order_and_breaks_fuzzy_score_ties() {
        let mut registry = EditorCommandRegistry::default();
        for index in 0..4 {
            registry
                .register(test_command(index))
                .expect("generated command ids should be unique");
        }
        let mru = EditorCommandPaletteMru::new([
            EditorOperationPath::parse("test.palette.command_0003")
                .expect("the recent command id should be valid"),
            EditorOperationPath::parse("test.palette.command_0001")
                .expect("the recent command id should be valid"),
        ])
        .expect("the bounded MRU list should be valid");
        let context = CommandEvalCtx::interactive();

        let catalog = registry.command_palette_catalog();
        let unfiltered = catalog.query_window_with_mru(&context, "", 0, 4, &mru);
        let fuzzy = catalog.query_window_with_mru(&context, "palette command", 0, 4, &mru);
        let expected = vec![
            "test.palette.command_0003",
            "test.palette.command_0001",
            "test.palette.command_0000",
            "test.palette.command_0002",
        ];

        assert_eq!(
            unfiltered
                .entries()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            fuzzy
                .entries()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn palette_query_index_visits_only_documents_with_the_rarest_query_byte() {
        let mut commands = (0..1_000).map(test_command).collect::<Vec<_>>();
        commands.push(EditorCommandDescriptor::new(
            EditorOperationPath::parse("test.palette.unique_zanzibar")
                .expect("unique command id should be valid"),
            "Unique Zanzibar Action",
            EditorCommandCategory::Command,
            EditorCommandAction::Emit(EditorEvent::Transient(
                EditorEventTransient::OpenCommandPalette,
            )),
        ));
        let registry =
            EditorCommandRegistry::new(commands).expect("generated command ids should be unique");

        let window = registry.command_palette_catalog().query_window(
            &CommandEvalCtx::interactive(),
            "zanzibar",
            0,
            12,
        );

        assert_eq!(window.total_match_count(), 1);
        assert_eq!(window.metrics().visited_entries, 1);
        assert_eq!(window.metrics().enablement_evaluations, 1);
        assert_eq!(
            window.entries().next().map(|entry| entry.id.as_str()),
            Some("test.palette.unique_zanzibar")
        );
    }

    #[test]
    fn palette_catalog_enablement_slot_preserves_descriptor_requirements() {
        let descriptor = test_command(0)
            .with_when(WhenClause::SelectionNonEmpty)
            .with_required_capabilities(["palette.execute"])
            .with_asset_write_target_arguments("asset_type", "asset_locator");
        let registry = EditorCommandRegistry::new(vec![descriptor])
            .expect("the descriptor should satisfy the registry contract");
        let selected = CommandEvalCtx::interactive()
            .with_selection_count(1)
            .with_capabilities(["palette.execute"]);

        let catalog = registry.command_palette_catalog();
        assert!(catalog.query_window(&selected, "palette", 0, 16).is_empty());
        assert_eq!(
            catalog
                .query_window(
                    &selected.with_asset_write_access(AssetWriteAccess::Writable),
                    "palette",
                    0,
                    16,
                )
                .total_match_count(),
            1
        );
    }

    #[test]
    fn one_thousand_query_updates_emit_current_source_burst_metrics() {
        let mut registry = EditorCommandRegistry::default();
        for index in 0..1_000 {
            registry
                .register(test_command(index))
                .expect("generated command ids should be unique");
        }
        let context = CommandEvalCtx::interactive();
        let catalog = registry.command_palette_catalog();

        let mut elapsed_micros = Vec::with_capacity(1_000);
        let mut maximum_visited_entries = 0;
        let mut maximum_document_byte_visits = 0;
        let mut maximum_text_comparisons = 0;
        let mut maximum_enablement_evaluations = 0;
        let mut maximum_candidate_handles = 0;
        let mut maximum_retained_handles = 0;
        let mut maximum_owned_buffers = 0;
        for index in 0..1_000 {
            let query = format!("command {:02}", index % 100);
            let started_at = Instant::now();
            let metrics = catalog.query_window(&context, &query, 0, 16).metrics();
            elapsed_micros.push(started_at.elapsed().as_micros());
            maximum_visited_entries = maximum_visited_entries.max(metrics.visited_entries);
            maximum_document_byte_visits =
                maximum_document_byte_visits.max(metrics.document_byte_visits);
            maximum_text_comparisons = maximum_text_comparisons.max(metrics.text_comparisons);
            maximum_enablement_evaluations =
                maximum_enablement_evaluations.max(metrics.enablement_evaluations);
            maximum_candidate_handles = maximum_candidate_handles.max(metrics.candidate_handles);
            maximum_retained_handles = maximum_retained_handles.max(metrics.retained_handles);
            maximum_owned_buffers = maximum_owned_buffers.max(metrics.owned_buffers);
        }
        elapsed_micros.sort_unstable();
        let p95_index = elapsed_micros
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        let p95_micros = elapsed_micros[p95_index];

        println!(
            "EDITOR08_PALETTE_QUERY_BURST samples=1000 p95_us={p95_micros} max_visits={maximum_visited_entries} max_document_byte_visits={maximum_document_byte_visits} max_text_comparisons={maximum_text_comparisons} max_enablement_evaluations={maximum_enablement_evaluations} max_candidate_handles={maximum_candidate_handles} max_retained_handles={maximum_retained_handles} max_owned_buffers={maximum_owned_buffers}"
        );
        assert_eq!(maximum_visited_entries, 1_000);
        assert!(maximum_document_byte_visits > 0);
        assert!(maximum_text_comparisons > 0);
        assert_eq!(maximum_enablement_evaluations, 1_000);
        assert!(maximum_candidate_handles <= 16);
        assert!(maximum_retained_handles <= 16);
        assert_eq!(maximum_owned_buffers, 4);
    }

    fn test_command(index: usize) -> EditorCommandDescriptor {
        EditorCommandDescriptor::new(
            EditorOperationPath::parse(&format!("test.palette.command_{index:04}"))
                .expect("generated command id should be valid"),
            format!("Palette Command {index:04}"),
            EditorCommandCategory::Command,
            EditorCommandAction::Emit(EditorEvent::Transient(
                EditorEventTransient::OpenCommandPalette,
            )),
        )
    }
}
