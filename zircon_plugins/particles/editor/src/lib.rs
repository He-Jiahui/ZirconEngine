mod authoring;
mod capability;
mod extension_ids;
mod plugin;

#[cfg(test)]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, PARTICLES_AUTHORING_CAPABILITY, PLUGIN_ID};
pub use extension_ids::{
    PARTICLES_AUTHORING_VIEW_ID, PARTICLES_COMPONENT_DRAWER_ID,
    PARTICLES_CPU_SPRITE_TEMPLATE_DOCUMENT, PARTICLES_CPU_SPRITE_TEMPLATE_ID, PARTICLES_DRAWER_ID,
    PARTICLES_PREVIEW_TEMPLATE_ID, PARTICLES_PREVIEW_VIEW_ID, PARTICLES_SYSTEM_ASSET_KIND,
    PARTICLES_TEMPLATE_ID,
};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, ParticlesEditorPlugin,
};
