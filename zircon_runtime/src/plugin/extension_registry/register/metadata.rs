#[cfg(feature = "graphics")]
use crate::core::framework::render::{
    GBufferChannelMask, GeometrySourceDescriptor, ShadingModelDescriptor,
    GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
};
#[cfg(feature = "ui")]
use crate::plugin::UiComponentDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginOptionManifest,
    RuntimeExtensionRegistryError,
};

#[cfg(feature = "ui")]
use super::super::validation::validate_ui_component_descriptor;
use super::super::validation::{
    validate_component_type_descriptor, validate_plugin_event_catalog_manifest,
    validate_plugin_option_manifest,
};
use super::super::RuntimeExtensionRegistry;

impl RuntimeExtensionRegistry {
    pub fn register_component(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_component_type_descriptor(&descriptor)?;
        if self.components.contains_key(&descriptor.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicateComponentType(
                descriptor.type_id,
            ));
        }
        let owner = self.intern_runtime_owner(&descriptor.plugin_id)?;
        self.components
            .register(owner, descriptor.type_id.clone(), descriptor)
            .expect("component duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "ui")]
    pub fn register_ui_component(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_ui_component_descriptor(&descriptor)?;
        if self.ui_components.contains_key(&descriptor.component_id) {
            return Err(RuntimeExtensionRegistryError::DuplicateUiComponent(
                descriptor.component_id,
            ));
        }
        let owner = self.intern_runtime_owner(&descriptor.plugin_id)?;
        self.ui_components
            .register(owner, descriptor.component_id.clone(), descriptor)
            .expect("ui component duplicate was prechecked");
        Ok(())
    }

    pub fn register_plugin_option(
        &mut self,
        descriptor: PluginOptionManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_option_manifest(&descriptor)?;
        if self.plugin_options.contains_key(&descriptor.key) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginOption(
                descriptor.key,
            ));
        }
        let owner = self.intern_owner_from_namespaced_key(&descriptor.key)?;
        self.plugin_options
            .register(owner, descriptor.key.clone(), descriptor)
            .expect("plugin option duplicate was prechecked");
        Ok(())
    }

    pub fn register_plugin_event_catalog(
        &mut self,
        descriptor: PluginEventCatalogManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_event_catalog_manifest(&descriptor)?;
        if self
            .plugin_event_catalogs
            .contains_key(&descriptor.namespace)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEventCatalog(
                descriptor.namespace,
            ));
        }
        let owner = self.intern_owner_from_namespaced_key(&descriptor.namespace)?;
        self.plugin_event_catalogs
            .register(owner, descriptor.namespace.clone(), descriptor)
            .expect("plugin event catalog duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_geometry_source(
        &mut self,
        plugin_id: impl AsRef<str>,
        descriptor: GeometrySourceDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let owner = self.intern_runtime_owner(plugin_id.as_ref())?;
        self.register_geometry_source_for_owner(owner, descriptor)
    }

    #[cfg(feature = "graphics")]
    pub(in crate::plugin) fn register_geometry_source_for_owner(
        &mut self,
        owner: crate::plugin::PluginModuleId,
        mut descriptor: GeometrySourceDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let key = validate_geometry_source_descriptor(&descriptor)?;
        descriptor.token = key.clone();
        if self.geometry_sources.contains_key(&key) {
            return Err(RuntimeExtensionRegistryError::DuplicateGeometrySource(key));
        }
        self.geometry_sources
            .register(owner, key, descriptor)
            .expect("geometry source duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_shading_model(
        &mut self,
        plugin_id: impl AsRef<str>,
        descriptor: ShadingModelDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let owner = self.intern_runtime_owner(plugin_id.as_ref())?;
        self.register_shading_model_for_owner(owner, descriptor)
    }

    #[cfg(feature = "graphics")]
    pub(in crate::plugin) fn register_shading_model_for_owner(
        &mut self,
        owner: crate::plugin::PluginModuleId,
        mut descriptor: ShadingModelDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let key = validate_shading_model_descriptor(&descriptor)?;
        descriptor.token = key.clone();
        if self.shading_models.contains_key(&key) {
            return Err(RuntimeExtensionRegistryError::DuplicateShadingModel(key));
        }
        self.shading_models
            .register(owner, key, descriptor)
            .expect("shading model duplicate was prechecked");
        Ok(())
    }
}

#[cfg(feature = "graphics")]
fn validate_geometry_source_descriptor(
    descriptor: &GeometrySourceDescriptor,
) -> Result<String, RuntimeExtensionRegistryError> {
    let token = descriptor.token.trim().to_ascii_lowercase();
    if !token.starts_with("custom:") || token.len() == "custom:".len() {
        return Err(RuntimeExtensionRegistryError::InvalidGeometrySource(
            format!(
                "geometry source token `{}` must use custom:<name>",
                descriptor.token
            ),
        ));
    }
    if !descriptor.id.is_plugin_range() {
        return Err(RuntimeExtensionRegistryError::InvalidGeometrySource(
            format!(
                "geometry source `{token}` id {} must be >= {GEOMETRY_SOURCE_PLUGIN_ID_START}",
                descriptor.id.value()
            ),
        ));
    }
    if descriptor.wgsl_include.trim().is_empty()
        || descriptor.wgsl_include.trim() != descriptor.wgsl_include
    {
        return Err(RuntimeExtensionRegistryError::InvalidGeometrySource(
            format!("geometry source `{token}` wgsl_include must be non-empty and trimmed"),
        ));
    }
    if descriptor.vertex_attributes.is_empty() {
        return Err(RuntimeExtensionRegistryError::InvalidGeometrySource(
            format!("geometry source `{token}` must declare vertex attributes"),
        ));
    }
    for binding in &descriptor.required_bindings {
        if binding.slot_token.trim().is_empty() || binding.slot_token.trim() != binding.slot_token {
            return Err(RuntimeExtensionRegistryError::InvalidGeometrySource(
                format!(
                    "geometry source `{token}` binding slot_token must be non-empty and trimmed"
                ),
            ));
        }
    }
    for define in &descriptor.shader_defines {
        let name = define.name();
        if name.trim().is_empty() || name.trim() != name {
            return Err(RuntimeExtensionRegistryError::InvalidGeometrySource(
                format!(
                    "geometry source `{token}` shader define name must be non-empty and trimmed"
                ),
            ));
        }
    }
    Ok(token)
}

#[cfg(feature = "graphics")]
fn validate_shading_model_descriptor(
    descriptor: &ShadingModelDescriptor,
) -> Result<String, RuntimeExtensionRegistryError> {
    let token = descriptor.token.trim().to_ascii_lowercase();
    if !token.starts_with("custom:") || token.len() == "custom:".len() {
        return Err(RuntimeExtensionRegistryError::InvalidShadingModel(format!(
            "shading model token `{}` must use custom:<name>",
            descriptor.token
        )));
    }
    if !descriptor.id.is_plugin_range() {
        return Err(RuntimeExtensionRegistryError::InvalidShadingModel(format!(
            "shading model `{token}` id {} must be >= {SHADING_MODEL_PLUGIN_ID_START}",
            descriptor.id
        )));
    }
    for (field, value) in [
        ("forward_include", descriptor.forward_include.as_str()),
        (
            "gbuffer_encode_include",
            descriptor.gbuffer_encode_include.as_str(),
        ),
        ("deferred_include", descriptor.deferred_include.as_str()),
    ] {
        if value.trim().is_empty() || value.trim() != value {
            return Err(RuntimeExtensionRegistryError::InvalidShadingModel(format!(
                "shading model `{token}` {field} must be non-empty and trimmed"
            )));
        }
    }
    if descriptor.required_channels == GBufferChannelMask::EMPTY {
        return Err(RuntimeExtensionRegistryError::InvalidShadingModel(format!(
            "shading model `{token}` must declare required G-buffer channels"
        )));
    }
    Ok(token)
}
