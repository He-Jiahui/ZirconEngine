use super::super::super::super::*;
use super::support::cpal_backend;

#[test]
fn cpal_backend_is_listed_when_feature_is_enabled() {
    let sound = DefaultSoundManager::default();
    let backend = cpal_backend(&sound);

    assert_eq!(backend.backend, "cpal");
}
