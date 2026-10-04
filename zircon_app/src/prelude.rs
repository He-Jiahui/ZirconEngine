//! Stable convenience imports for Zircon process entry and plugin composition.

pub use crate::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile, DefaultPlugins, DevPlugins,
    EntryConfig, EntryModuleSelection, EntryModuleSelectionReport, EntryProfile, EntryRunMode,
    EntryRunner, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
    PluginGroupError, ProductComposition, ProductCompositionRequest, ProductConfigSource,
    ProductConfigSourceSet, ProductExitClass, ProductHostConfigError, ProductHostConfigProvenance,
    ProductProcessExitCode, ProductRoleRequest, ResolvedPluginGroup, ResolvedProductHostConfig,
};
pub use crate::{retry_product_cleanup_until, ProductCloseError, ProductCompositionFailure};
pub use zircon_runtime::prelude::*;
