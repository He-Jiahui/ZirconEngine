use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::asset::AssetContextCommandAccess;
use crate::core::editing::operation::{
    OperationCommandFactoryError, OperationCommandFactoryRegistration,
};
use crate::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use crate::core::editor_extension::{EditorExtensionRegistryError, ViewDescriptor};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{ContributionBatch, ContributionStore};

use super::{
    AssetWriteTargetDescriptor, EditorCommandAction, EditorCommandDescriptor,
    EditorCommandExecutorRegistryError, EditorCommandMenuPath, EditorCommandMenuProjection,
    EditorCommandRegistry, EditorCommandRegistryError,
};

/// One-shot command descriptors plus the stable ids retained after registration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandContributionSet {
    command_ids: BTreeSet<EditorOperationPath>,
    pending: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    #[serde(skip)]
    pending_factories: BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
}

impl EditorCommandContributionSet {
    pub fn register(
        &mut self,
        descriptor: EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        EditorCommandRegistry::validate_descriptor(&descriptor)?;
        let command_id = self.claim_command_id(descriptor.id())?;
        self.pending.insert(command_id, descriptor);
        Ok(())
    }

    pub fn register_operation(
        &mut self,
        descriptor: EditorCommandDescriptor,
        factory: OperationCommandFactoryRegistration,
    ) -> Result<(), EditorCommandRegistryError> {
        if descriptor.id() != factory.operation() {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::OperationMismatch {
                    descriptor_operation: descriptor.id().clone(),
                    factory_operation: factory.operation().clone(),
                },
            ));
        }
        if !matches!(descriptor.action(), EditorCommandAction::Operation) {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DescriptorIsEvent {
                    operation: descriptor.id().clone(),
                },
            ));
        }
        EditorCommandRegistry::validate_descriptor(&descriptor)?;
        let operation = self.claim_command_id(descriptor.id())?;
        if self.pending_factories.contains_key(factory.operation()) {
            self.command_ids.remove(&operation);
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DuplicateFactory {
                    operation: factory.operation().clone(),
                },
            ));
        }
        self.pending.insert(operation.clone(), descriptor);
        self.pending_factories.insert(operation, factory);
        Ok(())
    }

    pub fn command_ids(&self) -> impl Iterator<Item = &EditorOperationPath> {
        self.command_ids.iter()
    }

    pub fn pending_command(&self, id: &EditorOperationPath) -> Option<&EditorCommandDescriptor> {
        self.pending.get(id)
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.pending.values()
    }

    pub fn pending_factory(
        &self,
        id: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.pending_factories.get(id)
    }

    pub fn take_pending(&mut self) -> Vec<EditorCommandDescriptor> {
        std::mem::take(&mut self.pending).into_values().collect()
    }

    pub fn take_pending_factories(&mut self) -> Vec<OperationCommandFactoryRegistration> {
        std::mem::take(&mut self.pending_factories)
            .into_values()
            .collect()
    }

    pub(crate) fn record_registered_id(&mut self, id: EditorOperationPath) {
        self.command_ids.insert(id);
    }

    fn claim_command_id(
        &mut self,
        id: &EditorOperationPath,
    ) -> Result<EditorOperationPath, EditorCommandRegistryError> {
        let command_id = id.clone();
        if !self.command_ids.insert(command_id.clone()) {
            return Err(EditorCommandRegistryError::DuplicateCommand(command_id));
        }
        Ok(command_id)
    }
}

/// Rebuilds the executable command registry from Store-owned contribution batches.
///
/// The Store remains the only contribution lifetime authority. Callers publish the returned
/// registry only after the surrounding host transaction has prepared every other projection.
pub(crate) fn project_command_registry_from_contributions(
    contributions: &ContributionStore,
    previous_generation: u64,
) -> Result<EditorCommandRegistry, EditorExtensionRegistryError> {
    let mut command_registry = EditorCommandRegistry::default_workbench();
    for extension in contributions.active_batches() {
        project_extension_commands(&mut command_registry, extension)?;
    }
    command_registry.publish_projection_after(previous_generation);
    Ok(command_registry)
}

fn project_extension_commands(
    command_registry: &mut EditorCommandRegistry,
    source_extension: &ContributionBatch,
) -> Result<(), EditorExtensionRegistryError> {
    let mut extension = source_extension.clone();
    let required_capabilities = extension.required_capabilities().to_vec();
    let views = extension.views().into_iter().cloned().collect::<Vec<_>>();
    let menu_capabilities = extension
        .menu_items()
        .into_iter()
        .filter(|item| !item.required_capabilities().is_empty())
        .fold(
            BTreeMap::<EditorOperationPath, Vec<String>>::new(),
            |mut capabilities, item| {
                capabilities
                    .entry(item.operation().clone())
                    .or_default()
                    .extend(item.required_capabilities().iter().cloned());
                capabilities
            },
        );
    let pending_command_ids = extension
        .pending_commands()
        .map(|command| command.id().clone())
        .collect::<BTreeSet<_>>();
    let native_command_ids = extension
        .native_command_bindings()
        .map(|(command_id, _)| command_id.clone())
        .collect::<BTreeSet<_>>();
    let asset_write_targets = asset_write_targets(&extension)?;
    let view_operation_ids = views
        .iter()
        .map(ViewDescriptor::open_operation_path)
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(EditorExtensionRegistryError::OperationPath)?;
    if let Some(command_id) = menu_capabilities.keys().find(|command_id| {
        !pending_command_ids.contains(*command_id) && !view_operation_ids.contains(*command_id)
    }) {
        return Err(
            EditorExtensionRegistryError::MenuCapabilitiesRequireContributedCommand {
                command_id: command_id.clone(),
            },
        );
    }
    let commands = extension.take_command_contributions();
    let mut operation_factories = extension
        .take_operation_factories()
        .into_iter()
        .map(|factory| (factory.operation().clone(), factory))
        .collect::<BTreeMap<_, _>>();
    let explicit_view_commands = commands
        .iter()
        .map(|command| (command.id().clone(), command.event().cloned()))
        .collect::<BTreeMap<_, _>>();
    for command in commands {
        if matches!(command.action(), EditorCommandAction::NativeEndpoint)
            && !native_command_ids.contains(command.id())
        {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::Executor(
                    EditorCommandExecutorRegistryError::MissingExecutor {
                        command_id: command.id().clone(),
                    },
                ),
            ));
        }
        let command_capabilities = menu_capabilities
            .get(command.id())
            .into_iter()
            .flatten()
            .cloned();
        let command = command
            .with_required_capabilities(required_capabilities.iter().cloned())
            .with_required_capabilities(command_capabilities);
        if let Some(factory) = operation_factories.remove(command.id()) {
            command_registry
                .register_operation(command, factory)
                .map_err(EditorExtensionRegistryError::Command)?;
        } else {
            command_registry
                .register(command)
                .map_err(EditorExtensionRegistryError::Command)?;
        }
    }
    for (command_id, binding) in extension.native_command_bindings() {
        command_registry
            .register_native_executor(command_id, binding.clone())
            .map_err(|error| {
                EditorExtensionRegistryError::Command(EditorCommandRegistryError::Executor(error))
            })?;
    }
    if let Some(operation) = operation_factories.keys().next().cloned() {
        return Err(EditorExtensionRegistryError::Command(
            EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::OrphanFactory { operation },
            ),
        ));
    }
    for view in &views {
        let operation_path = view
            .open_operation_path()
            .map_err(EditorExtensionRegistryError::OperationPath)?;
        let expected_event = extension_view_open_event(view);
        if let Some(explicit_event) = explicit_view_commands.get(&operation_path) {
            if explicit_event.as_ref() != Some(&expected_event) {
                return Err(EditorExtensionRegistryError::CommandViewTargetConflict {
                    command_id: operation_path,
                    view_id: view.id().to_string(),
                });
            }
        } else if command_registry.command(operation_path.as_str()).is_some() {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::DuplicateCommand(operation_path),
            ));
        } else {
            let mut view_capabilities = required_capabilities.clone();
            view_capabilities.extend(
                menu_capabilities
                    .get(&operation_path)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
            command_registry
                .register(extension_view_open_operation(
                    view,
                    operation_path.clone(),
                    &view_capabilities,
                ))
                .map_err(EditorExtensionRegistryError::Command)?;
        }
    }
    for (operation, target) in asset_write_targets {
        command_registry
            .attach_asset_write_target(&operation, target)
            .map_err(EditorExtensionRegistryError::Command)?;
    }
    let available_operations = command_registry
        .commands()
        .map(|descriptor| descriptor.id().clone())
        .collect::<BTreeSet<_>>();
    validate_menu_item_operation_bindings(&extension, &available_operations)?;
    validate_inspector_customization_operation_bindings(&extension, &available_operations)?;
    validate_asset_importer_operation_bindings(&extension, &available_operations)?;
    validate_asset_type_operation_bindings(&extension, &available_operations)
}

fn asset_write_targets(
    extension: &ContributionBatch,
) -> Result<BTreeMap<EditorOperationPath, AssetWriteTargetDescriptor>, EditorExtensionRegistryError>
{
    let mut targets = BTreeMap::new();
    for contribution in extension.asset_type_contributions() {
        for template in contribution.creation_templates() {
            insert_asset_write_target(
                &mut targets,
                template.operation().clone(),
                AssetWriteTargetDescriptor::new("asset_type", "target_folder"),
            )?;
        }
        for command in contribution
            .context_commands()
            .iter()
            .filter(|command| command.access() == AssetContextCommandAccess::Mutation)
        {
            insert_asset_write_target(
                &mut targets,
                command.operation().clone(),
                AssetWriteTargetDescriptor::new("asset_type", "asset_locator"),
            )?;
        }
    }
    Ok(targets)
}

fn insert_asset_write_target(
    targets: &mut BTreeMap<EditorOperationPath, AssetWriteTargetDescriptor>,
    operation: EditorOperationPath,
    target: AssetWriteTargetDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    if targets
        .get(&operation)
        .is_some_and(|existing| existing != &target)
    {
        return Err(EditorExtensionRegistryError::Command(
            EditorCommandRegistryError::ConflictingAssetWriteTarget(operation),
        ));
    }
    targets.insert(operation, target);
    Ok(())
}

fn extension_view_open_operation(
    view: &ViewDescriptor,
    operation_path: EditorOperationPath,
    required_capabilities: &[String],
) -> EditorCommandDescriptor {
    let menu_path = EditorCommandMenuPath::builtin(&operation_path, "view", &["extensions"]);
    EditorCommandDescriptor::operation(operation_path)
        .with_menu_path(menu_path)
        .with_menu_projection(EditorCommandMenuProjection::ExtensionRegistry)
        .with_required_capabilities(required_capabilities.iter().cloned())
        .with_event(extension_view_open_event(view))
}

fn extension_view_open_event(view: &ViewDescriptor) -> EditorEvent {
    EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view.id())))
}

fn validate_menu_item_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for menu_item in extension.menu_items() {
        if !available_operations.contains(menu_item.operation()) {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::MissingCommand(menu_item.operation().clone()),
            ));
        }
    }
    Ok(())
}

fn validate_inspector_customization_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for customization in extension.inspector_customizations() {
        let Some(surface) = customization.surface() else {
            continue;
        };
        for binding in surface.bindings() {
            let path = EditorOperationPath::parse(binding.clone())
                .map_err(EditorExtensionRegistryError::OperationPath)?;
            if !available_operations.contains(&path) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(path),
                ));
            }
        }
    }
    Ok(())
}

fn validate_asset_importer_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for importer in extension.asset_importers() {
        if !available_operations.contains(importer.operation()) {
            return Err(EditorExtensionRegistryError::Command(
                EditorCommandRegistryError::MissingCommand(importer.operation().clone()),
            ));
        }
    }
    Ok(())
}

fn validate_asset_type_operation_bindings(
    extension: &ContributionBatch,
    available_operations: &BTreeSet<EditorOperationPath>,
) -> Result<(), EditorExtensionRegistryError> {
    for contribution in extension.asset_type_contributions() {
        if let Some(toolkit) = contribution.toolkit() {
            if !available_operations.contains(toolkit.open_operation()) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(toolkit.open_operation().clone()),
                ));
            }
        }
        for template in contribution.creation_templates() {
            if !available_operations.contains(template.operation()) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(template.operation().clone()),
                ));
            }
        }
        for command in contribution.context_commands() {
            if !available_operations.contains(command.operation()) {
                return Err(EditorExtensionRegistryError::Command(
                    EditorCommandRegistryError::MissingCommand(command.operation().clone()),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const ADMISSION_COMMAND_COUNT: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;

    fn descriptor(id: &str) -> EditorCommandDescriptor {
        EditorCommandDescriptor::operation(EditorOperationPath::parse(id).unwrap())
    }

    fn admission_ids() -> Vec<EditorOperationPath> {
        (0..ADMISSION_COMMAND_COUNT)
            .map(|index| {
                EditorOperationPath::parse(format!("editor.performance.command_{index:05}"))
                    .unwrap()
            })
            .collect()
    }

    fn legacy_admit(ids: &[EditorOperationPath]) -> BTreeSet<EditorOperationPath> {
        let mut admitted = BTreeSet::new();
        for id in ids {
            if !admitted.contains(id) {
                admitted.insert(id.clone());
            }
        }
        admitted
    }

    fn single_pass_admit(ids: &[EditorOperationPath]) -> BTreeSet<EditorOperationPath> {
        let mut admitted = BTreeSet::new();
        for id in ids {
            admitted.insert(id.clone());
        }
        admitted
    }

    fn elapsed_micros(run: impl FnOnce()) -> u128 {
        let started = Instant::now();
        run();
        started.elapsed().as_micros()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * 95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn optimization_batch_20260826c_editor08_contribution_admission_uses_one_tree_traversal() {
        let source = include_str!("contribution.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;

        assert!(source.contains("fn claim_command_id("));
        assert!(source.contains("self.command_ids.insert(command_id.clone())"));
        assert!(!source.contains("self.command_ids.contains("));
    }

    #[test]
    fn optimization_batch_20260826c_editor08_contribution_admission_keeps_seen_ids_after_take() {
        let mut contributions = EditorCommandContributionSet::default();
        contributions
            .register(descriptor("editor.test.unique"))
            .unwrap();
        assert_eq!(contributions.take_pending().len(), 1);

        assert_eq!(
            contributions.register(descriptor("editor.test.unique")),
            Err(EditorCommandRegistryError::DuplicateCommand(
                EditorOperationPath::parse("editor.test.unique").unwrap()
            ))
        );
        assert!(contributions.take_pending().is_empty());
        assert_eq!(contributions.command_ids().count(), 1);
    }

    #[test]
    #[ignore = "release performance evidence for the managed validation coordinator"]
    fn optimization_batch_20260826c_editor08_contribution_admission_performance_evidence() {
        let ids = admission_ids();

        for _ in 0..3 {
            assert_eq!(black_box(legacy_admit(&ids)).len(), ADMISSION_COMMAND_COUNT);
            assert_eq!(
                black_box(single_pass_admit(&ids)).len(),
                ADMISSION_COMMAND_COUNT
            );
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            let measure_legacy = || {
                elapsed_micros(|| {
                    black_box(legacy_admit(black_box(&ids)));
                })
            };
            let measure_optimized = || {
                elapsed_micros(|| {
                    black_box(single_pass_admit(black_box(&ids)));
                })
            };
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
        println!(
            "EDITOR08_CONTRIBUTION_SINGLE_PASS_ADMISSION_BENCH_V1 sample_pairs={} command_ids={} legacy_membership_tree_traversals={} optimized_membership_tree_traversals={} legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
            SAMPLE_PAIRS,
            ADMISSION_COMMAND_COUNT,
            ADMISSION_COMMAND_COUNT * 2,
            ADMISSION_COMMAND_COUNT,
            legacy_p95,
            optimized_p95,
            legacy_samples,
            optimized_samples,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
            "single-pass contribution admission p95 must be at least 20% below contains-plus-insert: legacy={legacy_p95}us optimized={optimized_p95}us"
        );
    }
}
