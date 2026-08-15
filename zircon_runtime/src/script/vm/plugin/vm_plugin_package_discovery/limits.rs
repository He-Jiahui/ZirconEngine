use std::time::Duration;

pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_DEPTH: usize = 16;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_ENTRIES: usize = 16_384;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_MANIFESTS: usize = 1_024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_MANIFEST_BYTES: usize = 256 * 1024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_TOTAL_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_PATH_BYTES: usize = 32 * 1024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_TOTAL_PATH_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_BYTECODE_BYTES: usize = 64 * 1024 * 1024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_CACHED_BYTECODE_ENTRIES: usize = 1_024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_CACHED_BYTECODE_BYTES: usize = 256 * 1024 * 1024;
pub const DEFAULT_VM_PLUGIN_DISCOVERY_MAX_WALL_TIME: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmPluginDiscoveryLimits {
    pub max_depth: usize,
    pub max_entries: usize,
    pub max_manifests: usize,
    pub max_manifest_bytes: usize,
    pub max_total_manifest_bytes: usize,
    pub max_path_bytes: usize,
    pub max_total_path_bytes: usize,
    pub max_bytecode_bytes: usize,
    pub max_cached_bytecode_entries: usize,
    pub max_cached_bytecode_bytes: usize,
    pub max_wall_time: Duration,
}

impl Default for VmPluginDiscoveryLimits {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_DEPTH,
            max_entries: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_ENTRIES,
            max_manifests: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_MANIFESTS,
            max_manifest_bytes: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_MANIFEST_BYTES,
            max_total_manifest_bytes: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_TOTAL_MANIFEST_BYTES,
            max_path_bytes: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_PATH_BYTES,
            max_total_path_bytes: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_TOTAL_PATH_BYTES,
            max_bytecode_bytes: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_BYTECODE_BYTES,
            max_cached_bytecode_entries: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_CACHED_BYTECODE_ENTRIES,
            max_cached_bytecode_bytes: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_CACHED_BYTECODE_BYTES,
            max_wall_time: DEFAULT_VM_PLUGIN_DISCOVERY_MAX_WALL_TIME,
        }
    }
}
