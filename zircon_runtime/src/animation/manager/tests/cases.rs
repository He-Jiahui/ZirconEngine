use std::panic::{self, AssertUnwindSafe};

use crate::core::framework::animation::{AnimationManager, AnimationPlaybackSettings};

use super::DefaultAnimationManager;

#[test]
fn animation_manager_playback_settings_recover_poisoned_lock() {
    let manager = DefaultAnimationManager::default();
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.lock_playback_settings();
        panic!("poison animation playback settings");
    }));

    let mut playback_settings = AnimationPlaybackSettings::default();
    playback_settings.enabled = false;
    manager
        .store_playback_settings(playback_settings.clone())
        .expect("store playback settings after poisoned lock");

    assert_eq!(manager.playback_settings(), playback_settings);
}
