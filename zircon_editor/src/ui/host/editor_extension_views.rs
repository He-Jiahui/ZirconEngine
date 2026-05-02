use crate::core::editor_extension::ViewDescriptor as ExtensionViewDescriptor;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId, ViewKind};

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
        let mut registry = self.view_registry.lock().unwrap();
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
        let registry = self.view_registry.lock().unwrap();
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
    view.required_capabilities = required_capabilities.to_vec();
    view
}

fn validate_extension_view_descriptors(
    registry: &crate::ui::workbench::view::ViewRegistry,
    views: &[ViewDescriptor],
) -> Result<(), EditorError> {
    let mut pending = std::collections::HashSet::new();
    for view in views {
        if registry.descriptor(&view.descriptor_id).is_some()
            || !pending.insert(view.descriptor_id.clone())
        {
            return Err(EditorError::Registry(format!(
                "view descriptor {} already registered",
                view.descriptor_id.0
            )));
        }
    }
    Ok(())
}
