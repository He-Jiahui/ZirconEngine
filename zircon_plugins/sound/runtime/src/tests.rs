use zircon_runtime::core::framework::sound::SoundManager;
use zircon_runtime::RuntimePluginRegistrationReport;

use super::{runtime_plugin, DefaultSoundManager, SOUND_MODULE_NAME};

#[test]
fn sound_plugin_registration_contributes_runtime_module() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == SOUND_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ]
    );
}

#[test]
fn default_sound_manager_renders_silence_without_active_playback() {
    let sound = DefaultSoundManager::default();
    let mix = sound.render_mix(3).unwrap();

    assert_eq!(mix.sample_rate_hz, 48_000);
    assert_eq!(mix.channel_count, 2);
    assert_eq!(mix.samples, vec![0.0; 6]);
}
