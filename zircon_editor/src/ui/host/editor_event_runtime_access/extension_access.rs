use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::editor_extension::{
    AssetImporterDescriptor, EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSnapshot,
};
use crate::core::extension::{
    CapabilitySet, ContributionSource, InspectTargetType, InspectorCustomization,
};
use crate::ui::activity::ActivityViewDescriptor;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::view::ViewInstanceId;

fn template_pane_view_is_current(
    manager: &crate::ui::host::editor_manager::EditorManager,
    instance_id: &ViewInstanceId,
    template_id: &str,
) -> bool {
    let registry = manager.host.lock_view_registry();
    let Some(instance) = registry.instance(instance_id) else {
        return false;
    };
    registry
        .descriptor(&instance.descriptor_id)
        .is_some_and(|descriptor| {
            registry.descriptor_capability_error(descriptor).is_none()
                && descriptor
                    .pane_template
                    .as_ref()
                    .is_some_and(|template| template.body.document_id == template_id)
        })
}

impl EditorHostEventController {
    pub fn activity_view_descriptor(&self, view_id: &str) -> Option<ActivityViewDescriptor> {
        self.shell()
            .lock()
            .control_service
            .activity_view(view_id)
            .cloned()
    }

    pub fn inspector_customization(
        &self,
        component_type: &str,
    ) -> Result<Option<Arc<dyn InspectorCustomization>>, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        Ok({
            let Ok(target_type) = InspectTargetType::new(component_type) else {
                return Ok(None);
            };
            let inner = self.shell().lock();
            let enabled_capabilities = manager
                .capability_snapshot()
                .enabled_capabilities()
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            let customization = inner
                .contributions
                .snapshot()
                .inspector_customizations(&enabled_capabilities)
                .find(|customization| customization.can_handle(&target_type));
            customization
        })
    }

    pub fn ui_template_descriptor(
        &self,
        id: &str,
    ) -> Result<Option<EditorUiTemplateDescriptor>, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        Ok({
            let inner = self.shell().lock();
            let enabled_capabilities = manager
                .capability_snapshot()
                .enabled_capabilities()
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            let descriptor = inner
                .contributions
                .snapshot()
                .ui_templates(&enabled_capabilities)
                .find(|descriptor| descriptor.id() == id)
                .cloned();
            descriptor
        })
    }

    pub(crate) fn extension_projection_is_current(
        &self,
        observed_generation: u64,
        observed_capabilities: &[String],
    ) -> Result<bool, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        let inner = self.shell().lock();
        Ok(inner.contributions.generation() == observed_generation
            && manager.enabled_capabilities_match(observed_capabilities))
    }

    pub(crate) fn extension_projection_revision(
        &self,
    ) -> Result<(u64, Vec<String>), crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        Ok({
            let inner = self.shell().lock();
            let enabled_capabilities = manager
                .capability_snapshot()
                .enabled_capabilities()
                .to_vec();
            (inner.contributions.generation(), enabled_capabilities)
        })
    }

    pub(crate) fn enabled_plugin_template_descriptors(
        &self,
    ) -> Result<
        (
            u64,
            Vec<String>,
            BTreeMap<String, Vec<EditorUiTemplateDescriptor>>,
        ),
        crate::ui::host::EditorError,
    > {
        let manager = self.enter_editor_manager()?;
        Ok({
            let inner = self.shell().lock();
            let enabled_capabilities = manager
                .capability_snapshot()
                .enabled_capabilities()
                .to_vec();
            let capabilities = enabled_capabilities
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            let templates_by_owner = inner
                .contributions
                .snapshot()
                .ui_templates_with_source(&capabilities)
                .filter_map(|(source, template)| match source {
                    ContributionSource::Plugin(plugin_id) => Some((plugin_id.as_str(), template)),
                    ContributionSource::Builtin => None,
                })
                .fold(
                    BTreeMap::<String, Vec<EditorUiTemplateDescriptor>>::new(),
                    |mut templates_by_owner, (owner_id, template)| {
                        templates_by_owner
                            .entry(owner_id.to_owned())
                            .or_default()
                            .push(template.clone());
                        templates_by_owner
                    },
                );
            (
                inner.contributions.generation(),
                enabled_capabilities,
                templates_by_owner,
            )
        })
    }

    pub(crate) fn ui_template_pane_data_snapshots(
        &self,
    ) -> Result<BTreeMap<String, EditorUiTemplatePaneDataSnapshot>, crate::ui::host::EditorError>
    {
        let manager = self.enter_editor_manager()?;
        Ok({
            let sources = {
                let inner = self.shell().lock();
                let enabled_capabilities = manager
                    .capability_snapshot()
                    .enabled_capabilities()
                    .iter()
                    .cloned()
                    .collect::<CapabilitySet>();
                inner
                    .contributions
                    .snapshot()
                    .ui_template_pane_data_sources(&enabled_capabilities)
                    .map(|(template_id, source)| (template_id.to_owned(), source))
                    .collect::<BTreeMap<_, _>>()
            };

            sources
                .into_iter()
                .map(|(template_id, source)| (template_id, source.snapshot()))
                .collect()
        })
    }

    pub(crate) fn ui_template_pane_data_snapshot(
        &self,
        template_id: &str,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<EditorUiTemplatePaneDataSnapshot>, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        if !template_pane_view_is_current(&manager, instance_id, template_id) {
            return Ok(None);
        }
        let (source, generation, enabled_capabilities) = {
            let inner = self.shell().lock();
            let enabled_capabilities = manager
                .capability_snapshot()
                .enabled_capabilities()
                .to_vec();
            let capabilities = enabled_capabilities
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            (
                inner
                    .contributions
                    .snapshot()
                    .ui_template_pane_data_source(template_id, &capabilities),
                inner.contributions.generation(),
                enabled_capabilities,
            )
        };
        let Some(source) = source else {
            return Ok(None);
        };
        // Extension callbacks run outside host locks and may retire their view or source.
        let snapshot = source.snapshot();
        if !template_pane_view_is_current(&manager, instance_id, template_id)
            || !self.extension_projection_is_current(generation, &enabled_capabilities)?
        {
            return Ok(None);
        }
        Ok(Some(snapshot))
    }

    pub fn asset_importers_for_extension(
        &self,
        extension: &str,
    ) -> Result<Vec<AssetImporterDescriptor>, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        Ok({
            let normalized = extension
                .trim()
                .trim_start_matches('.')
                .to_ascii_lowercase();
            let inner = self.shell().lock();
            let enabled_capabilities = manager
                .capability_snapshot()
                .enabled_capabilities()
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            inner
                .contributions
                .snapshot()
                .asset_importers(&enabled_capabilities)
                .filter(|descriptor| {
                    descriptor
                        .source_extensions()
                        .iter()
                        .any(|candidate| candidate == &normalized)
                })
                .cloned()
                .collect()
        })
    }
}

#[cfg(test)]
#[path = "tests/pane_data.rs"]
mod template_pane_data_tests;
