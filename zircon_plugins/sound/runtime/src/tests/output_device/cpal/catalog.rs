use super::super::super::*;

#[cfg(feature = "cpal-backend")]
#[test]
fn cpal_backend_is_listed_when_feature_is_enabled() {
    let sound = DefaultSoundManager::default();
    let backend = sound
        .available_output_backends()
        .unwrap()
        .into_iter()
        .find(|backend| backend.backend == "cpal")
        .expect("cpal backend should be listed with cpal-backend feature");
    assert!(backend.realtime_capable);
    assert!(!backend.deterministic);
}
