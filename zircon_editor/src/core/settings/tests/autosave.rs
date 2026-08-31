use std::time::Duration;

use super::{SettingValue, SettingsAuthority, SettingsScope, key};
use crate::core::settings::EDITOR_AUTOSAVE_INTERVAL_SECS_KEY;

#[test]
fn autosave_interval_user_setting_publishes_a_typed_hot_snapshot() {
    let authority = SettingsAuthority::with_defaults();
    assert_eq!(
        authority.snapshot().autosave_interval(),
        Duration::from_secs(300)
    );

    authority
        .set(
            SettingsScope::User,
            &key(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY),
            SettingValue::Int(60),
        )
        .unwrap()
        .expect("a changed autosave interval should publish a settings generation");

    assert_eq!(
        authority.snapshot().autosave_interval(),
        Duration::from_secs(60)
    );
}
