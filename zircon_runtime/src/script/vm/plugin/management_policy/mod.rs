mod garbage_collection;
mod hot_reload;
mod memory;
mod policy;

pub use garbage_collection::{VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy};
pub use hot_reload::VmPluginHotReloadPolicy;
pub use memory::VmPluginMemoryPolicy;
pub use policy::VmPluginManagementPolicy;

#[cfg(test)]
mod tests {
    use super::{
        VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy, VmPluginHotReloadPolicy,
        VmPluginManagementPolicy, VmPluginMemoryPolicy,
    };

    #[test]
    fn default_management_policy_preserves_state_and_defers_gc_to_backend() {
        let policy = VmPluginManagementPolicy::default();

        assert_eq!(policy.hot_reload, VmPluginHotReloadPolicy::PreserveState);
        assert_eq!(
            policy.garbage_collection.mode,
            VmPluginGarbageCollectionMode::BackendManaged
        );
        assert_eq!(policy.garbage_collection.interval_frames, None);
        assert_eq!(policy.memory.soft_limit_bytes, None);
        assert_eq!(policy.memory.hard_limit_bytes, None);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn management_policy_rejects_incoherent_gc_and_memory_limits() {
        let disabled_gc_with_interval = VmPluginGarbageCollectionPolicy {
            mode: VmPluginGarbageCollectionMode::Disabled,
            interval_frames: Some(60),
        };
        let error = VmPluginManagementPolicy::default()
            .with_garbage_collection(disabled_gc_with_interval)
            .validate()
            .unwrap_err();
        assert!(error.contains("disabled garbage collection"));

        let error = VmPluginManagementPolicy::default()
            .with_memory(VmPluginMemoryPolicy::with_limits(Some(2048), Some(1024)))
            .validate()
            .unwrap_err();
        assert!(error.contains("soft_limit_bytes 2048 exceeds hard_limit_bytes 1024"));
    }
}
