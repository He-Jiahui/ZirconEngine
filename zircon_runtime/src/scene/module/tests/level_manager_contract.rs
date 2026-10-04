use std::sync::atomic::Ordering;

use crate::core::framework::scene::{LevelManager, LevelManagerError};

use super::DefaultLevelManager;

const CONTRACT_SOURCE: &str = include_str!("../level_manager_contract.rs");

#[test]
fn level_manager_asset_io_uses_the_active_project_generation_without_a_scan() {
    assert!(CONTRACT_SOURCE.contains(concat!("current_project_", "snapshot()")));
    assert!(!CONTRACT_SOURCE.contains(concat!("ProjectManager", "::open")));
    assert!(!CONTRACT_SOURCE.contains(concat!("scan_and_", "import")));
}

#[test]
fn level_manager_contract_maps_kernel_handle_exhaustion_to_its_domain_error() {
    let manager = DefaultLevelManager::default();
    manager.next_handle.store(u64::MAX, Ordering::Relaxed);

    assert_eq!(
        LevelManager::create_default_level_handle(&manager),
        Err(LevelManagerError::HandleSpaceExhausted)
    );
}
