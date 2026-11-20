//! Shared device-free fixture for real WAV import and Sound service tests.
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths, ResolvedProjectPathIdentity};
use zircon_runtime::asset::{AssetModule, AssetUri, ASSET_MODULE_NAME};
use zircon_runtime::core::runtime::modules::{TasksModule, TASKS_MODULE_NAME};
use zircon_runtime::core::{CoreRuntime, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::foundation::{self, FOUNDATION_MODULE_NAME};
use zircon_runtime_interface::project::RelPath;

pub(super) struct ScratchRoot(ResolvedProjectPathIdentity);

impl ScratchRoot {
    pub(super) fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        // Compiler storage and managed-runner TEMP admission are external to this test.
        let base = ProjectPaths::resolve_identity(std::env::temp_dir()).unwrap();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = base.operation_path().join(format!(
            "zr-wav-reopen-{label}-{}-{nanos}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create new fixture; never adopt an existing root");
        let owned = ProjectPaths::resolve_identity(&path).unwrap();
        assert!(owned.is_within(&base));
        assert_eq!(owned.operation_path(), path.as_path());
        Self(owned)
    }

    pub(super) fn path(&self) -> &Path {
        self.0.operation_path()
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        // Declaring the fixture first lets service guards and watcher owners drop first.
        let path = self.path();
        if std::env::var_os("ZIRCON_KEEP_CLIP_REOPEN_FIXTURES").is_some_and(|value| value == "1") {
            eprintln!("retained clip reopen fixture: {}", path.display());
            return;
        }
        if !ProjectPaths::resolve_identity(path).is_ok_and(|current| current == self.0) {
            eprintln!(
                "refusing cleanup of a changed clip fixture: {}",
                path.display()
            );
            return;
        }
        if let Err(error) = fs::remove_dir_all(path) {
            eprintln!("could not clean clip fixture {}: {error}", path.display());
        }
    }
}

pub(super) fn create_runtime(
    scratch: &ScratchRoot,
    sound: ModuleDescriptor,
    sound_module_name: &str,
) -> CoreRuntime {
    let report = zircon_plugin_audio_importer_runtime::plugin_registration();
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let asset = AssetModule::with_asset_importers(report.extensions.asset_importers().clone());
    let runtime = CoreRuntime::new();
    let mut foundation = foundation::module_descriptor();
    foundation::bind_config_file_path(&mut foundation, scratch.path().join("config.json")).unwrap();
    for descriptor in [
        foundation,
        TasksModule.descriptor(),
        asset.descriptor(),
        sound,
    ] {
        runtime.register_module(descriptor).unwrap();
    }
    for name in [
        FOUNDATION_MODULE_NAME,
        TASKS_MODULE_NAME,
        ASSET_MODULE_NAME,
        sound_module_name,
    ] {
        runtime.activate_module(name).unwrap();
    }
    runtime
}

pub(super) fn create_project(root: &Path, name: &str) -> ProjectPaths {
    fs::create_dir(root).unwrap();
    let paths = ProjectPaths::from_root(root).unwrap();
    let assets = RelPath::project_assets();
    paths.ensure_layout(std::slice::from_ref(&assets)).unwrap();
    fs::create_dir(paths.asset_root(&assets).join("audio")).unwrap();
    ProjectManifest::new(
        name,
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    paths
}

pub(super) fn write_project_wav(project: &ProjectPaths, uri: &str, pcm: (u32, &[i16])) {
    let path = project
        .asset_root(&RelPath::project_assets())
        .join(AssetUri::parse(uri).unwrap().path());
    write_wav(&path, pcm);
}

pub(super) fn write_wav(path: &Path, (rate, pcm): (u32, &[i16])) {
    let data_bytes = u32::try_from(pcm.len() * 2).unwrap();
    let mut wav = b"RIFF".to_vec();
    wav.extend((36 + data_bytes).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes()); // PCM
    wav.extend(1u16.to_le_bytes()); // Mono
    wav.extend(rate.to_le_bytes());
    wav.extend((rate * 2).to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(data_bytes.to_le_bytes());
    for sample in pcm {
        wav.extend(sample.to_le_bytes());
    }
    fs::write(path, wav).unwrap();
}
