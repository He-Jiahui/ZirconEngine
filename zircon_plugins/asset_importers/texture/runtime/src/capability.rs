pub const PLUGIN_ID: &str = "asset_importer.texture";
pub const IMPORTER_FAMILY: &str = "texture";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_asset_importer_texture_runtime";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.texture";
pub const CONTAINER_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.container";
pub const PSD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.psd";

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    RUNTIME_CAPABILITY,
    CONTAINER_IMPORTER_CAPABILITY,
    PSD_IMPORTER_CAPABILITY,
];
