#[path = "management_policy/mod.rs"]
mod management_policy;
#[path = "state_migration.rs"]
mod state_migration;
#[path = "vm_plugin_instance.rs"]
mod vm_plugin_instance;
#[path = "vm_plugin_manifest.rs"]
mod vm_plugin_manifest;
#[path = "vm_plugin_package.rs"]
mod vm_plugin_package;
#[path = "vm_plugin_package_discovery.rs"]
mod vm_plugin_package_discovery;
#[path = "vm_plugin_package_source.rs"]
mod vm_plugin_package_source;
#[path = "vm_state_blob.rs"]
mod vm_state_blob;

pub use management_policy::{
    VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy, VmPluginHotReloadPolicy,
    VmPluginManagementPolicy, VmPluginManagementPolicyError, VmPluginManagementPolicyResult,
    VmPluginMemoryPolicy,
};
pub use state_migration::{
    migrate_vm_state_blob, VmStateMigrationError, VmStateSchema, VmStateTypeSchema,
};
pub use vm_plugin_instance::VmPluginInstance;
pub use vm_plugin_manifest::VmPluginManifest;
pub use vm_plugin_package::{VmPluginPackage, ZrVmExecutionMode, ZrVmPluginProjectSource};
pub use vm_plugin_package_discovery::{
    discover_vm_plugin_package, discover_vm_plugin_package_with_limits,
    discover_vm_plugin_packages, discover_vm_plugin_packages_with_limits,
    DiscoveredVmPluginPackage, VmPluginDiscoveryLimits, VmPluginDiscoveryRequest,
};
pub(crate) use vm_plugin_package_discovery::{VmPluginDiscoveryWorker, VmPluginPayloadCache};
pub use vm_plugin_package_source::VmPluginPackageSource;
pub use vm_state_blob::{
    VmStateBlob, VmStateFieldValue, VmStateObject, VmStateTypeIdentity, VM_STATE_SCHEMA_VERSION_V3,
};
