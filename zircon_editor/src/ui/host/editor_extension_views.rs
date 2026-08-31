use crate::core::editor_extension::ViewDescriptor as ExtensionViewDescriptor;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewKind,
};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use super::editor_ui_host::EditorUiHost;

impl EditorManager {
    pub fn register_extension_view(
        &self,
        descriptor: &ExtensionViewDescriptor,
    ) -> Result<(), EditorError> {
        self.host
            .register_extension_view_with_required_capabilities(descriptor, &[])
    }

    pub fn register_extension_view_with_required_capabilities(
        &self,
        descriptor: &ExtensionViewDescriptor,
        required_capabilities: &[String],
    ) -> Result<(), EditorError> {
        self.host
            .register_extension_view_with_required_capabilities(descriptor, required_capabilities)
    }

    pub fn register_extension_views_with_required_capabilities(
        &self,
        descriptors: &[ExtensionViewDescriptor],
        required_capabilities: &[String],
    ) -> Result<(), EditorError> {
        self.host
            .register_extension_views_with_required_capabilities(descriptors, required_capabilities)
    }

    pub fn validate_extension_views(
        &self,
        descriptors: &[ExtensionViewDescriptor],
    ) -> Result<(), EditorError> {
        self.host.validate_extension_views(descriptors)
    }
}

impl EditorUiHost {
    pub(super) fn retire_extension_views(
        &self,
        descriptor_ids: &[ViewDescriptorId],
    ) -> Result<(), EditorError> {
        if descriptor_ids.is_empty() {
            return Ok(());
        }

        let descriptor_ids = descriptor_ids
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let instance_ids = self
            .current_view_instances()
            .into_iter()
            .filter(|instance| descriptor_ids.contains(&instance.descriptor_id))
            .map(|instance| instance.instance_id)
            .collect::<Vec<_>>();

        // Preflight all document closes before changing layout state. Each lease rolls back on
        // drop, so a busy toolkit rejects the complete owner retirement.
        for instance_id in &instance_ids {
            if self.non_closeable_instance(instance_id) {
                return Err(EditorError::Registry(format!(
                    "extension view {} is not closeable",
                    instance_id.0
                )));
            }
            let _close = self.begin_document_close(instance_id)?;
        }
        for instance_id in &instance_ids {
            if !self.close_view(instance_id)? {
                return Err(EditorError::Registry(format!(
                    "extension view instance {} could not be retired",
                    instance_id.0
                )));
            }
        }

        let mut registry = self.lock_view_registry();
        for descriptor_id in descriptor_ids {
            registry
                .unregister_view(&descriptor_id)
                .map_err(EditorError::Registry)?;
        }
        Ok(())
    }

    pub(super) fn register_extension_view_with_required_capabilities(
        &self,
        descriptor: &ExtensionViewDescriptor,
        required_capabilities: &[String],
    ) -> Result<(), EditorError> {
        self.register_extension_views_with_required_capabilities(
            std::slice::from_ref(descriptor),
            required_capabilities,
        )
    }

    pub(super) fn register_extension_views_with_required_capabilities(
        &self,
        descriptors: &[ExtensionViewDescriptor],
        required_capabilities: &[String],
    ) -> Result<(), EditorError> {
        let views = descriptors
            .iter()
            .map(|descriptor| extension_view_descriptor(descriptor, required_capabilities))
            .collect::<Vec<_>>();
        let mut registry = self.lock_view_registry();
        validate_extension_view_descriptors(&registry, &views)?;
        for view in views {
            registry
                .register_view(view)
                .map_err(EditorError::Registry)?;
        }
        Ok(())
    }

    pub(super) fn validate_extension_views(
        &self,
        descriptors: &[ExtensionViewDescriptor],
    ) -> Result<(), EditorError> {
        let views = descriptors
            .iter()
            .map(|descriptor| extension_view_descriptor(descriptor, &[]))
            .collect::<Vec<_>>();
        let registry = self.lock_view_registry();
        validate_extension_view_descriptors(&registry, &views)
    }
}

fn extension_view_descriptor(
    descriptor: &ExtensionViewDescriptor,
    required_capabilities: &[String],
) -> ViewDescriptor {
    let mut view = ViewDescriptor::new(
        ViewDescriptorId::new(descriptor.id()),
        ViewKind::ActivityView,
        descriptor.display_name(),
    )
    .with_icon_key(descriptor.id());
    view.document_kind = descriptor.document_kind().cloned();
    if let Some(template_id) = descriptor.ui_template_id() {
        view = view.with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
            template_id,
            PanePayloadKind::TemplateV2,
            PaneRouteNamespace::Template,
            PaneInteractionMode::TemplateOnly,
        )));
    }
    view.required_capabilities = required_capabilities.to_vec();
    view
}

fn validate_extension_view_descriptors(
    registry: &crate::ui::workbench::view::ViewRegistry,
    views: &[ViewDescriptor],
) -> Result<(), EditorError> {
    let mut pending = std::collections::HashSet::<&ViewDescriptorId>::with_capacity(views.len());
    for view in views {
        if registry.descriptor(&view.descriptor_id).is_some()
            || !pending.insert(&view.descriptor_id)
        {
            return Err(EditorError::Registry(format!(
                "view descriptor {} already registered",
                view.descriptor_id.0
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind, ViewRegistry};

    use super::validate_extension_view_descriptors;

    fn view(id: &str) -> ViewDescriptor {
        ViewDescriptor::new(
            ViewDescriptorId::new(id),
            ViewKind::ActivityView,
            format!("View {id}"),
        )
    }

    #[test]
    fn borrowed_view_validation_accepts_unique_ids() {
        let registry = ViewRegistry::default();
        let views = [view("plugin.example.first"), view("plugin.example.second")];

        validate_extension_view_descriptors(&registry, &views).unwrap();
    }

    #[test]
    fn borrowed_view_validation_rejects_a_batch_duplicate() {
        let registry = ViewRegistry::default();
        let views = [view("plugin.example.same"), view("plugin.example.same")];

        assert!(validate_extension_view_descriptors(&registry, &views).is_err());
    }
}
