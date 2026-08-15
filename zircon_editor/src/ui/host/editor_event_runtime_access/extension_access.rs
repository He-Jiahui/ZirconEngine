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
    ) -> Option<Arc<dyn InspectorCustomization>> {
        let target_type = InspectTargetType::new(component_type).ok()?;
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
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
    }

    pub fn ui_template_descriptor(&self, id: &str) -> Option<EditorUiTemplateDescriptor> {
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
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
    }

    pub(crate) fn plugin_template_revision(&self) -> (u64, Vec<String>) {
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        (inner.contributions.generation(), enabled_capabilities)
    }

    pub(crate) fn enabled_plugin_template_descriptors(
        &self,
    ) -> (
        u64,
        Vec<String>,
        BTreeMap<String, Vec<EditorUiTemplateDescriptor>>,
    ) {
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
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
    }

    pub(crate) fn ui_template_pane_data_snapshots(
        &self,
    ) -> BTreeMap<String, EditorUiTemplatePaneDataSnapshot> {
        let sources = {
            let inner = self.shell().lock();
            let enabled_capabilities = inner
                .manager
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
    }

    pub(crate) fn ui_template_pane_data_snapshot(
        &self,
        template_id: &str,
    ) -> Option<EditorUiTemplatePaneDataSnapshot> {
        let source = {
            let inner = self.shell().lock();
            let enabled_capabilities = inner
                .manager
                .capability_snapshot()
                .enabled_capabilities()
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            inner
                .contributions
                .snapshot()
                .ui_template_pane_data_source(template_id, &enabled_capabilities)
        };
        source.map(|source| source.snapshot())
    }

    pub fn asset_importers_for_extension(&self, extension: &str) -> Vec<AssetImporterDescriptor> {
        let normalized = extension
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        let inner = self.shell().lock();
        let enabled_capabilities = inner
            .manager
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
    }
}
