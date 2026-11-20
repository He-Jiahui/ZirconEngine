use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{project_asset_manager_handle, AssetManager};
use zircon_runtime::core::framework::sound::SoundError;
use zircon_runtime::core::manager::{resolve_manager_service, sound_manager_handle};

#[path = "../../../tests/support/project_clip_fixture.rs"]
mod fixture;

#[test]
fn cached_clip_load_reuses_active_project_and_package_clips() {
    const RES: &str = "res://audio/cached.wav";
    const PACKAGE: &str = "package://com.zircon.sound/audio/cached.wav";
    let scratch = fixture::ScratchRoot::new("cache-unit");
    let paths = fixture::create_project(&scratch.path().join("project"), "Clip Cache Unit");
    fixture::write_project_wav(&paths, RES, (8_000, &[0, 4096]));
    let package = scratch.path().join("package");
    std::fs::create_dir_all(package.join("audio")).unwrap();
    fixture::write_wav(
        &package.join("audio/cached.wav"),
        (16_000, &[8192, 0, -8192]),
    );

    let mut project = ProjectManager::open(paths.root()).unwrap();
    project
        .register_package_asset_root("com.zircon.sound", &package)
        .unwrap();
    let runtime = fixture::create_runtime(
        &scratch,
        crate::module_descriptor(),
        crate::SOUND_MODULE_NAME,
    );
    let core = runtime.handle();
    let assets =
        resolve_manager_service(&core, project_asset_manager_handle(&core).unwrap()).unwrap();
    let sound = resolve_manager_service(&core, sound_manager_handle(&core).unwrap()).unwrap();
    AssetManager::open_prepared_project(&*assets, project).unwrap();

    let mut loaded = Vec::new();
    for (locator, rate, frames) in [(RES, 8_000, 2), (PACKAGE, 16_000, 3)] {
        let status = assets.asset_status(locator).unwrap();
        assert!(status.imported);
        assert_eq!(status.importer_id, "audio_importer.wav");
        let first = sound.load_clip(locator).unwrap();
        assert_eq!(sound.load_clip(locator).unwrap(), first);
        let info = sound.clip_info(first).unwrap();
        assert_eq!(
            (info.locator.as_str(), info.sample_rate_hz, info.frame_count),
            (locator, rate, frames)
        );
        loaded.push(first);
    }
    assert_ne!(loaded[0], loaded[1]);
    for locator in [
        "res://audio/missing.wav",
        "package://com.zircon.sound/audio/missing.wav",
        "package://com.zircon.sound",
    ] {
        assert_eq!(
            sound.load_clip(locator),
            Err(SoundError::InvalidLocator {
                locator: locator.into()
            })
        );
    }
    AssetManager::close_project(&*assets).unwrap();
}
