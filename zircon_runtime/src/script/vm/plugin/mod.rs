mod management_policy;
mod state_migration;
mod vm_plugin_instance;
mod vm_plugin_manifest;
mod vm_plugin_package;
mod vm_plugin_package_discovery;
mod vm_plugin_package_source;
mod vm_state_blob;

pub use management_policy::{
    VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy, VmPluginHotReloadPolicy,
    VmPluginManagementPolicy, VmPluginManagementPolicyError, VmPluginManagementPolicyResult,
    VmPluginMemoryPolicy,
};
pub use state_migration::{
    migrate_vm_state_blob, VmStateFieldRename, VmStateMigrationError, VmStateSchema,
    VmStateTypeSchema,
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
    VmStateBlob, VmStateObject, VmStateTypeIdentity, VM_STATE_SCHEMA_VERSION_V2,
};
