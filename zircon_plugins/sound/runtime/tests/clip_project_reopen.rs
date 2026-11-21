//! Device-free regression of the current-project `load_clip` contract.
use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::artifact::ArtifactStore;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::asset::{
    project_asset_manager_handle, AssetKind, AssetManager, AssetStatusRecord, AssetUri,
    ImportedAsset, ProjectAssetManager,
};
use zircon_runtime::core::framework::sound::SoundError;
use zircon_runtime::core::manager::{resolve_manager_service, sound_manager_handle};
#[path = "support/project_clip_fixture.rs"]
mod fixture;

use fixture::{create_project, create_runtime, write_project_wav, ScratchRoot};

const BEEP: &str = "res://audio/beep.wav";
const CONTROL: &str = "res://audio/finite.wav";
const FIRST: (u32, &[i16]) = (8_000, &[-8192, 4096, 8192]);
const REPLACEMENT: (u32, &[i16]) = (16_000, &[8192, 0, -16384, 4096]);
const FINITE: (u32, &[i16]) = (8_000, &[0, 4096, 8192]);

#[test]
fn wav_project_reopen_loads_current_pcm_at_the_same_locator() {
    let scratch = ScratchRoot::new("changed");
    let project = create_project(&scratch.path().join("project"), "Audio Reopen");
    write_project_wav(&project, BEEP, FIRST);
    write_project_wav(&project, CONTROL, FINITE);

    let runtime = create_runtime(
        &scratch,
        zircon_plugin_sound_runtime::module_descriptor(),
        zircon_plugin_sound_runtime::SOUND_MODULE_NAME,
    );
    let core = runtime.handle();
    let assets = resolve_manager_service(&core, project_asset_manager_handle(&core).unwrap())
        .expect("actual ProjectAssetManager");
    let sound = resolve_manager_service(&core, sound_manager_handle(&core).unwrap())
        .expect("actual neutral SoundManager");
    open(&assets, &project);
    let first_activation = assets
        .current_project_generation_snapshot()
        .unwrap()
        .project()
        .activation_identity()
        .clone();
    let (before, original_manifest) = assert_ready_pcm(&assets, &project, BEEP, FIRST);
    let (finite_before, _) = assert_ready_pcm(&assets, &project, CONTROL, FINITE);
    let old_clip = sound.load_clip(BEEP).unwrap();
    assert_eq!(sound.load_clip(BEEP).unwrap(), old_clip);
    assert_eq!(
        sound.clip_info(old_clip).unwrap().frame_count,
        FIRST.1.len()
    );
    assert_eq!(sound.clip_info(old_clip).unwrap().sample_rate_hz, FIRST.0);
    let finite_old_clip = sound.load_clip(CONTROL).unwrap();
    assert_eq!(
        sound.clip_info(finite_old_clip).unwrap().frame_count,
        FINITE.1.len()
    );

    assert_eq!(
        AssetManager::close_project(&*assets).unwrap().as_deref(),
        Some(project.root())
    );
    assert!(assets
        .resolve_asset_id(&AssetUri::parse(BEEP).unwrap())
        .is_none());
    // A real v1 artifact survives on disk; only the source WAV changes while closed.
    write_project_wav(&project, BEEP, REPLACEMENT);
    let (old_locator, old_manifest_path) = locate_artifact(&project, &before);
    assert_eq!(fs::read(old_manifest_path).unwrap(), original_manifest);
    assert_artifact_pcm(&project, &old_locator, BEEP, FIRST);

    // Reopen on the same Runtime and exact same resolved SoundManager.
    open(&assets, &project);
    let new_activation = assets.current_project_generation_snapshot().unwrap();
    assert_ne!(
        new_activation.project().activation_identity(),
        &first_activation
    );
    let (after, _) = assert_ready_pcm(&assets, &project, BEEP, REPLACEMENT);
    assert_eq!(after.id, before.id, "same .zmeta source identity");
    assert_eq!(after.artifact_uri, before.artifact_uri);
    assert_ne!(after.source_hash, before.source_hash);

    // Valid unchanged PCM remains available through both the disk and the Sound consumer.
    let (finite_after, _) = assert_ready_pcm(&assets, &project, CONTROL, FINITE);
    assert_eq!(finite_after.id, finite_before.id);
    assert_eq!(finite_after.source_hash, finite_before.source_hash);
    let control = sound.load_clip(CONTROL).unwrap();
    let info = sound.clip_info(control).unwrap();
    assert_eq!(
        (info.sample_rate_hz, info.frame_count),
        (FINITE.0, FINITE.1.len())
    );

    let current = sound.load_clip(BEEP).expect("current activated Wave");
    let info = sound.clip_info(current).unwrap();
    assert_eq!(info.locator, BEEP);
    assert_eq!(
        (info.sample_rate_hz, info.frame_count),
        (REPLACEMENT.0, REPLACEMENT.1.len()),
        "Runtime has already verified the new finite artifact; load_clip must not return the retired clip"
    );
    assert_eq!(sound.load_clip(BEEP).unwrap(), current);
    // Only discovery is retired. Explicit old handles keep their already-loaded clip.
    let old = sound.clip_info(old_clip).unwrap();
    assert_eq!(
        (old.sample_rate_hz, old.frame_count),
        (FIRST.0, FIRST.1.len())
    );
    AssetManager::close_project(&*assets).unwrap();
}

#[test]
fn wav_project_reopen_cannot_find_a_path_from_the_retired_project() {
    let scratch = ScratchRoot::new("missing");
    let first = create_project(&scratch.path().join("first"), "Has Beep");
    let second = create_project(&scratch.path().join("second"), "No Beep");
    write_project_wav(&first, BEEP, FIRST);
    write_project_wav(&second, CONTROL, FINITE);
    let runtime = create_runtime(
        &scratch,
        zircon_plugin_sound_runtime::module_descriptor(),
        zircon_plugin_sound_runtime::SOUND_MODULE_NAME,
    );
    let core = runtime.handle();
    let assets = resolve_manager_service(&core, project_asset_manager_handle(&core).unwrap())
        .expect("actual ProjectAssetManager");
    let sound = resolve_manager_service(&core, sound_manager_handle(&core).unwrap())
        .expect("actual neutral SoundManager");

    open(&assets, &first);
    assert_ready_pcm(&assets, &first, BEEP, FIRST);
    let old = sound.load_clip(BEEP).unwrap();
    assert_eq!(sound.clip_info(old).unwrap().sample_rate_hz, FIRST.0);
    AssetManager::close_project(&*assets).unwrap();
    // Read both states before checking results so the second activation is always exercised.
    let while_closed = sound.load_clip(BEEP);
    open(&assets, &second);
    assert_ready_pcm(&assets, &second, CONTROL, FINITE);
    assert!(assets
        .resolve_asset_id(&AssetUri::parse(BEEP).unwrap())
        .is_none());
    let missing = sound.load_clip(BEEP);
    let control = sound.load_clip(CONTROL).unwrap();
    assert_eq!(
        sound.clip_info(control).unwrap().frame_count,
        FINITE.1.len()
    );
    AssetManager::close_project(&*assets).unwrap();
    assert_eq!(
        (while_closed, missing),
        (
            Err(SoundError::NoProjectOpen),
            Err(SoundError::InvalidLocator {
                locator: BEEP.into()
            })
        ),
        "closed and missing-in-current-project are not valid cache hits",
    );
}

fn open(assets: &ProjectAssetManager, project: &ProjectPaths) {
    let root = project.root().to_str().unwrap();
    AssetManager::open_project(assets, root).expect("normal project asset activation");
}

fn assert_ready_pcm(
    assets: &ProjectAssetManager,
    project: &ProjectPaths,
    uri: &str,
    pcm: (u32, &[i16]),
) -> (AssetStatusRecord, Vec<u8>) {
    let status = assets.asset_status(uri).expect("published Sound record");
    assert!(status.imported);
    assert_eq!(status.kind, AssetKind::Sound);
    assert_eq!(status.importer_id, "audio_importer.wav");
    assert_eq!(status.importer_version, 1);
    let (locator, manifest) = locate_artifact(project, &status);
    assert_artifact_pcm(project, &locator, uri, pcm);
    let asset_id = assets
        .resolve_asset_id(&AssetUri::parse(uri).unwrap())
        .unwrap();
    let loaded = assets
        .load_sound_asset(asset_id)
        .expect("typed current PCM");
    assert_eq!(loaded.uri.to_string(), uri);
    assert_eq!(loaded.sample_rate_hz, pcm.0);
    assert_eq!(loaded.channel_count, 1);
    assert_eq!(loaded.samples, normalized(pcm.1));
    (status, fs::read(manifest).unwrap())
}

fn assert_artifact_pcm(project: &ProjectPaths, locator: &AssetUri, uri: &str, pcm: (u32, &[i16])) {
    let asset = ArtifactStore::default().read(project, locator).unwrap();
    let ImportedAsset::Sound(sound) = asset else {
        panic!("expected Sound artifact {locator}");
    };
    assert_eq!(sound.uri.to_string(), uri);
    assert_eq!(sound.sample_rate_hz, pcm.0);
    assert_eq!(sound.channel_count, 1);
    assert_eq!(sound.samples, normalized(pcm.1));
}

fn locate_artifact(project: &ProjectPaths, row: &AssetStatusRecord) -> (AssetUri, PathBuf) {
    let locator = AssetUri::parse(row.artifact_uri.as_ref().unwrap()).unwrap();
    let path = project.asset_artifact_root().join(locator.path());
    assert!(path.is_file(), "expected persisted artifact {path:?}");
    (locator, path)
}

fn normalized(samples: &[i16]) -> Vec<f32> {
    samples.iter().map(|n| f32::from(*n) / 32768.0).collect()
}
