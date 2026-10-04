// 经公共预设应用入口拒绝未知定位符，核对 InvalidLocator 错误。
use zircon_runtime::core::framework::sound::{SoundError, SoundMixerGraphManager};

use super::super::DefaultSoundManager;

#[test]
fn applying_unknown_mixer_preset_returns_typed_locator_error() {
    let sound = DefaultSoundManager::default();

    assert!(matches!(
        sound
            .apply_mixer_preset("sound://mixer/missing")
            .unwrap_err(),
        SoundError::InvalidLocator { .. }
    ));
}
