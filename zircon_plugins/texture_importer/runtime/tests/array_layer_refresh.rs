#![cfg(windows)]

use std::os::windows::fs::MetadataExt;
use std::path::{Component, Path, PathBuf, Prefix};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs, process};

use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder, ImageFormat};
use zircon_plugin_texture_importer_runtime::plugin_registration;
use zircon_runtime::asset::artifact::ArtifactStore;
use zircon_runtime::asset::project::{
    AssetMetaDocument, PreviewState, ProjectManifest, ProjectPaths,
};
use zircon_runtime::asset::{
    AssetId, AssetKind, AssetManager, AssetUri, ImportedAsset, ProjectAssetManager, TextureAsset,
    TexturePayload, RGBA8_UNORM_SRGB_FORMAT,
};
use zircon_runtime::core::framework::render::RenderImageDimension;
use zircon_runtime::core::resource::{ResourceScheme, ResourceState};

const EDITED: &str = "a.png";
const UNCHANGED: &str = "b.png";
const ARRAY: &str = "t.zarray";
const SOURCES: [(&str, &str, u32); 3] = [
    (EDITED, "texture_importer.image", 1),
    (UNCHANGED, "texture_importer.image", 1),
    (ARRAY, "texture_importer.array", 2),
];
const MANIFEST: &[u8] = b"sources = [\"b.png\", \"a.png\"]\n";
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];

#[test]
#[ignore = "requires an owned ZIRCON_TEXTURE_TEST_ROOT; retains disk specimens"]
fn array_layer_source_changes_refresh_the_baked_texture() {
    let run = new_run();
    let [red, blue, green] = [RED, BLUE, GREEN].map(encode_png);
    assert_ne!(red, blue);

    let (warm, assets) = open_project(&run.join("w"), &red, &green);
    let active = warm.current_project_manager().expect("warm active project");
    let guid = active.manifest().project_guid;
    let activation = active.activation_identity().clone();
    let manifest = active.paths().manifest_path().to_owned();
    let project_bytes = fs::read(&manifest).expect("warm project manifest");
    drop(active);
    let before = read_bakes(&warm, "warm-first");
    assert_colors(&before, RED);
    assert_eq!(full_scan(&warm, "warm-unchanged"), before);

    let (cold, cold_assets) = open_project(&run.join("c"), &blue, &green);
    let cold_project = cold.current_project_manager().expect("cold active project");
    assert_ne!(cold_project.manifest().project_guid, guid);
    drop(cold_project);
    check_png(&cold_assets.join(EDITED), &blue, BLUE);
    let reference = read_bakes(&cold, "cold-new");
    assert_colors(&reference, BLUE);
    for (old, new) in before.iter().zip(&reference) {
        assert_ne!(old.meta.uuid, new.meta.uuid, "independent asset UUIDs");
    }
    drop(cold);

    let source = assets.join(EDITED);
    check_png(&source, &red, RED);
    // This is the only edit to the warm project. Do not rewrite the .zarray or its sidecar.
    fs::write(&source, &blue).expect("replace the edited PNG at the same path");
    let after = full_scan(&warm, "warm-only-png-edited");
    check_png(&source, &blue, BLUE);
    assert_eq!(fs::read(assets.join(UNCHANGED)).unwrap(), green);
    assert_eq!(fs::read(assets.join(ARRAY)).unwrap(), MANIFEST);
    assert_eq!(fs::read(&manifest).unwrap(), project_bytes);
    let active = warm.current_project_manager().unwrap();
    assert_eq!(active.manifest().project_guid, guid);
    assert_eq!(active.activation_identity(), &activation);
    for (old, new) in before.iter().zip(&after) {
        assert_eq!(old.meta.uuid, new.meta.uuid, "persistent asset UUID");
        assert_eq!(old.meta.url, new.meta.url);
    }

    let [old_png, old_other, old_array] = &before;
    let [new_png, new_other, new_array] = &after;
    let [cold_png, _, cold_array] = &reference;
    assert_eq!(new_png.texture, cold_png.texture, "edited PNG rebaked");
    assert_ne!(old_png.meta.source_digest, new_png.meta.source_digest);
    assert_eq!(new_png.meta.source_digest, cold_png.meta.source_digest);
    assert_ne!(old_png.digest, new_png.digest);
    assert_eq!(old_other, new_other);
    assert_eq!(&new_array.texture.rgba[..4], &GREEN, "keep layer 0");
    eprintln!("array-layer controls passed; compare edited warm array to cold-new array");
    assert_eq!(
        new_array.texture.rgba, cold_array.texture.rgba,
        "full scan after only a.png changed must rebake the array's layer 1"
    );
    assert_ne!(old_array.digest, new_array.digest);
}

fn assert_colors([png, other, array]: &[Bake; 3], edited: [u8; 4]) {
    assert_eq!(png.texture.rgba, edited);
    assert_eq!(other.texture.rgba, GREEN);
    assert_eq!(array.texture.rgba, [GREEN, edited].concat());
}

fn open_project(root: &Path, edited: &[u8], unchanged: &[u8]) -> (ProjectAssetManager, PathBuf) {
    fs::create_dir(root).expect("setup fresh project directory");
    let paths = ProjectPaths::from_root(root).expect("setup physical project paths");
    assert_eq!(paths.root(), root);
    let manifest = ProjectManifest::new("Array sweep", uri("main.scene.toml"), 1);
    paths.ensure_layout(&manifest.asset_roots).unwrap();
    manifest.save(paths.manifest_path()).unwrap();
    let assets = manifest.primary_asset_root_path(&paths).unwrap();
    for (name, bytes) in [(EDITED, edited), (UNCHANGED, unchanged), (ARRAY, MANIFEST)] {
        fs::write(assets.join(name), bytes).expect("setup authored fixture");
    }
    let empty_store = fs::read_dir(paths.asset_artifact_root())
        .unwrap()
        .next()
        .is_none();
    assert!(empty_store, "new project starts without artifacts");

    let report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let registry = report.extensions.asset_importers();
    let manager = ProjectAssetManager::default();
    for handler in registry.importers() {
        manager
            .register_asset_importer_arc(handler)
            .expect("attach actual texture handler");
    }
    AssetManager::open_project(&manager, root.to_str().expect("UTF-8 project path"))
        .expect("setup activate isolated project and run its first full import");
    // Isolate explicit full scans from watcher imports without retiring the active project.
    assert!(
        manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(5)),
        "setup could not stop and join the project watchers"
    );
    for (name, importer, _) in SOURCES {
        let path = assets.join(name);
        let registered = registry.capability_report_for_source(&path).unwrap();
        let selected = manager
            .asset_importer_capability_report_for_source(&path)
            .unwrap();
        assert_eq!(selected, registered, "actual project handler for {name}");
        assert_eq!(selected.descriptor.id, importer);
        assert_eq!(selected.descriptor.output_kind, AssetKind::Texture);
        assert!(selected.status.is_available(), "{selected:?}");
    }
    (manager, assets)
}

#[derive(Debug, PartialEq)]
struct Bake {
    meta: AssetMetaDocument,
    digest: String,
    texture: TextureAsset,
}

fn full_scan(manager: &ProjectAssetManager, stage: &str) -> [Bake; 3] {
    let rows = AssetManager::reimport_all(manager).unwrap_or_else(|e| panic!("{stage}: {e}"));
    assert_eq!(rows.len(), SOURCES.len(), "{stage} full scan");
    read_bakes(manager, stage)
}

fn read_bakes(manager: &ProjectAssetManager, stage: &str) -> [Bake; 3] {
    let project = manager.current_project_manager().expect("active project");
    let paths = project.paths();
    let assets = project.manifest().primary_asset_root_path(paths).unwrap();
    let store = ArtifactStore::default();
    eprintln!(
        "array-layer {stage} root={} project={}",
        ProjectPaths::display_path(paths.root()).display(),
        project.manifest().project_guid
    );
    SOURCES.map(|(name, importer, layers)| {
        let res = uri(name);
        let row = project.registry().get_by_locator(&res).expect("project row");
        assert_eq!(row.state, ResourceState::Ready, "{stage} {res}: {row:?}");
        let meta = AssetMetaDocument::load(&assets.join(format!("{name}.zmeta"))).unwrap();
        assert_eq!(meta.preview_state, PreviewState::Ready, "{stage} {res}");
        assert_eq!(meta.url, res);
        assert_eq!(meta.asset_kind, AssetKind::Texture);
        assert_eq!(row.kind, AssetKind::Texture);
        assert_eq!(row.id(), AssetId::from_asset_uuid(meta.uuid));
        assert_eq!(meta.importer_id, importer);
        assert_eq!(row.importer_id, meta.importer_id);
        assert_eq!(row.importer_version, meta.importer_version);
        assert_eq!(row.config_hash, meta.config_hash);
        assert_eq!(row.source_hash, meta.source_digest);
        assert!(meta.config_hash.starts_with("blake3:") && !meta.source_digest.is_empty());
        let lib = meta.artifact_locator.as_ref().expect("persisted artifact locator");
        assert_eq!(row.artifact_locator(), Some(lib));
        assert_eq!(lib.scheme(), ResourceScheme::Library);
        let inventory = store.open_chunk_inventory(paths, lib).expect("read disk inventory");
        assert_eq!(inventory.kind(), AssetKind::Texture);
        let digest = inventory.content_hash().to_owned();
        let physical = as_texture(store.read(paths, lib).expect("decode lib artifact"));
        let loaded = as_texture(project.load_artifact(&res).expect("decode via res URI"));
        eprintln!(
            "array-layer {stage} {res} uuid={} importer={}@{} source={} key={} {lib} digest={digest} lib_rgba={:02x?} res_rgba={:02x?}",
            meta.uuid, meta.importer_id, meta.importer_version, meta.source_digest, meta.config_hash,
            physical.rgba, loaded.rgba,
        );
        assert_eq!(physical, loaded, "{stage} {res} resolves to actual disk texture");
        assert_eq!(physical.uri, res);
        assert_eq!(physical.payload, TexturePayload::Rgba8);
        let descriptor = physical.texture_descriptor();
        assert_eq!((physical.width, physical.height, descriptor.mip_count), (1, 1, 1));
        assert_eq!(descriptor.dimension, RenderImageDimension::D2);
        assert_eq!(descriptor.format, RGBA8_UNORM_SRGB_FORMAT);
        assert_eq!(descriptor.depth_or_array_layers, layers);
        assert_eq!(physical.rgba.len(), 4 * layers as usize);
        Bake { meta, digest, texture: physical }
    })
}

fn as_texture(asset: ImportedAsset) -> TextureAsset {
    let ImportedAsset::Texture(texture) = asset else {
        panic!("expected decoded Texture, got {asset:?}");
    };
    texture
}

fn uri(name: &str) -> AssetUri {
    AssetUri::parse(&format!("res://{name}")).unwrap()
}

fn encode_png(color: [u8; 4]) -> Vec<u8> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(&color, 1, 1, ExtendedColorType::Rgba8)
        .unwrap();
    bytes
}

fn check_png(path: &Path, bytes: &[u8], color: [u8; 4]) {
    let disk = fs::read(path).expect("read physical source PNG");
    assert_eq!(disk, bytes, "original fixture encoding at {path:?}");
    let png = image::load_from_memory_with_format(&disk, ImageFormat::Png)
        .unwrap()
        .into_rgba8();
    assert_eq!(png.dimensions(), (1, 1));
    assert_eq!(png.into_raw(), color);
}

fn new_run() -> PathBuf {
    const REPARSE_POINT: u32 = 0x400;
    let requested = PathBuf::from(
        env::var_os("ZIRCON_TEXTURE_TEST_ROOT").expect("set an owned E:/F: fixture root"),
    );
    let mut parts = requested.components();
    assert!(
        matches!(parts.next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(b'E' | b'F')))
            && parts.next() == Some(Component::RootDir)
            && matches!(parts.next(), Some(Component::Normal(_)))
            && parts.all(|part| matches!(part, Component::Normal(_))),
        "setup needs an absolute E:/F: subdirectory: {requested:?}"
    );
    let base = ProjectPaths::resolve_existing(&requested).expect("resolve existing fixture root");
    assert_eq!(
        base.display_path(),
        requested.as_path(),
        "setup requires a physical path, not an alias"
    );
    for path in requested.ancestors() {
        let entry = fs::symlink_metadata(path).expect("inspect fixture root ancestry");
        assert!(entry.is_dir() && entry.file_attributes() & REPARSE_POINT == 0);
        assert!(
            !path.join(".git").exists(),
            "fixture root must not be in a checkout"
        );
    }
    let marker = fs::symlink_metadata(base.operation_path().join(".zr-array-test-root"))
        .expect("runner must precreate an empty .zr-array-test-root marker");
    assert!(marker.is_file() && marker.file_attributes() & REPARSE_POINT == 0 && marker.len() == 0);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = base
        .operation_path()
        .join(format!("zra-{:x}-{stamp:x}", process::id()));
    fs::create_dir(&run).expect("create exclusive fixture child; never reuse a project");
    eprintln!(
        "array-layer retained fixture root: {}",
        ProjectPaths::display_path(&run).display()
    );
    run
}

mod array_source_contract {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;
    use zircon_runtime::asset::AssetImportContext;
    use zircon_runtime::core::diagnostics::profiling;

    const PNG_IMPORTER: &str = "texture_importer.image";
    const ARRAY_IMPORTER: &str = "texture_importer.array";

    #[test]
    #[ignore = "requires ZIRCON_TEXTURE_TEST_ROOT and zircon_runtime/profiling; retains disk specimens"]
    fn array_source_contract_restores_then_reopens_after_offline_png_edit() {
        require_profiling();
        let root = new_run().join("p");
        let [red, green, blue] = [RED, GREEN, BLUE].map(encode_png);
        let (paths, assets) = fixture(
            &root,
            &[(EDITED, &red), (UNCHANGED, &green), (ARRAY, MANIFEST)],
        );
        let project_bytes = fs::read(paths.manifest_path()).unwrap();
        let activate = || open_existing(&root, &assets, &[EDITED, UNCHANGED, ARRAY]);
        let identity = |manager: &ProjectAssetManager| {
            let project = manager.current_project_manager().unwrap();
            assert_eq!(project.paths().root(), root.as_path());
            (
                project.manifest().project_guid,
                project.activation_identity().clone(),
            )
        };

        let (first, work) = observe("cold-fixture-first-open", activate);
        assert_eq!(work, [3.0, 0.0, 3.0]);
        let (guid, first_activation) = identity(&first);
        let before = read_bakes(&first, "qualification-first");
        assert_colors(&before, RED);
        assert_eq!(scan(&first, "same-manager-unchanged", 3), [3.0, 3.0, 0.0]);
        assert_eq!(read_bakes(&first, "qualification-unchanged-scan"), before);
        close(first, &root);

        let (unchanged, work) = observe("same-root-unchanged-open", activate);
        assert_eq!(work, [3.0, 3.0, 0.0]);
        let (unchanged_guid, unchanged_activation) = identity(&unchanged);
        assert_eq!(unchanged_guid, guid);
        assert_ne!(unchanged_activation, first_activation);
        assert_eq!(
            read_bakes(&unchanged, "qualification-unchanged-open"),
            before
        );
        close(unchanged, &root);

        // Close retires and joins its watchers. No manager can update this cache before open.
        let meta_path = assets.join(format!("{ARRAY}.zmeta"));
        let old_sidecar = fs::read(&meta_path).unwrap();
        let source = assets.join(EDITED);
        check_png(&source, &red, RED);
        fs::write(&source, &blue).unwrap();
        check_png(&source, &blue, BLUE);
        assert_eq!(fs::read(&meta_path).unwrap(), old_sidecar);
        assert_eq!(fs::read(assets.join(ARRAY)).unwrap(), MANIFEST);
        assert_eq!(fs::read(assets.join(UNCHANGED)).unwrap(), green);
        assert_eq!(fs::read(paths.manifest_path()).unwrap(), project_bytes);

        let (changed, work) = observe("same-root-after-offline-png-edit", activate);
        let (changed_guid, changed_activation) = identity(&changed);
        assert_eq!(changed_guid, guid);
        assert_ne!(changed_activation, unchanged_activation);
        let after = read_bakes(&changed, "qualification-post-edit-open");
        close(changed, &root);
        assert_eq!(after[0].texture.rgba, BLUE);
        assert_rebaked(&before[0], &after[0]);
        assert_eq!(after[1], before[1]);
        assert_eq!(after[2].meta.uuid, before[2].meta.uuid);
        eprintln!(
            "array-source contract offline-edit controls passed; compare same-root parent layers"
        );
        assert_eq!(
            after[2].texture.rgba,
            [GREEN, BLUE].concat(),
            "same disk project must restore the unchanged layer and rebake the offline edit"
        );
        assert_rebaked(&before[2], &after[2]);
        assert_eq!(work, [3.0, 1.0, 2.0]);
    }

    #[test]
    #[ignore = "requires ZIRCON_TEXTURE_TEST_ROOT and zircon_runtime/profiling; retains disk specimens"]
    fn array_source_contract_tracks_reordered_members_and_drops_removed_inputs() {
        require_profiling();
        const ADDED: &str = "c.png";
        const YELLOW: [u8; 4] = [255, 255, 0, 255];
        const WHITE: [u8; 4] = [255, 255, 255, 255];
        const MEMBERS: &[u8] = b"sources = [\"c.png\", \"a.png\", \"a.png\"]\n";
        let root = new_run().join("p");
        let ((manager, assets), work) = observe("list-first-open", || {
            open_project(&root, &encode_png(RED), &encode_png(GREEN))
        });
        assert_eq!(work, [3.0, 0.0, 3.0]);
        let initial = read_bakes(&manager, "qualification-list-first");
        assert_colors(&initial, RED);
        let manifest = assets.join(ARRAY);
        let array = |stage: &str, colors: &[[u8; 4]]| bake(&manager, stage, ARRAY, colors);

        fs::write(&manifest, b"sources = [\"a.png\", \"b.png\", \"a.png\"]\n").unwrap();
        assert_eq!(scan(&manager, "reorder-and-repeat", 3), [3.0, 2.0, 1.0]);
        let reordered = array("reorder-and-repeat", &[RED, GREEN, RED]);
        assert_rebaked(&initial[2], &reordered);

        fs::write(assets.join(ADDED), encode_png(YELLOW)).unwrap();
        fs::write(&manifest, MEMBERS).unwrap();
        assert_eq!(scan(&manager, "add-c-remove-b", 4), [4.0, 2.0, 2.0]);
        let membership = array("add-c-remove-b", &[YELLOW, RED, RED]);
        assert_rebaked(&reordered, &membership);
        let added = bake(&manager, "c-added", ADDED, &[YELLOW]);

        let edit = |stage: &str, name: &str, color, previous: &Bake| {
            let path = assets.join(name);
            let bytes = encode_png(color);
            fs::write(&path, &bytes).unwrap();
            check_png(&path, &bytes, color);
            let work = scan(&manager, stage, 4);
            let png = bake(&manager, stage, name, &[color]);
            assert_rebaked(previous, &png);
            (png, work)
        };

        let (b, work) = edit("removed-b-only", UNCHANGED, BLUE, &initial[1]);
        assert_eq!(array("removed-b-only", &[YELLOW, RED, RED]), membership);
        assert_eq!(bake(&manager, "c-unchanged", ADDED, &[YELLOW]), added);
        assert_eq!(work, [4.0, 3.0, 1.0], "only standalone b may import");

        let (c, work) = edit("added-c-only", ADDED, WHITE, &added);
        let changed_c = array("added-c-only", &[WHITE, RED, RED]);
        assert_rebaked(&membership, &changed_c);
        assert_eq!(work, [4.0, 2.0, 2.0]);

        let (_, work) = edit("retained-a-only", EDITED, BLUE, &initial[0]);
        let changed_a = array("retained-a-only", &[WHITE, BLUE, BLUE]);
        assert_rebaked(&changed_c, &changed_a);
        assert_eq!(work, [4.0, 2.0, 2.0]);
        assert_eq!(bake(&manager, "b-unchanged", UNCHANGED, &[BLUE]), b);
        assert_eq!(bake(&manager, "c-still-white", ADDED, &[WHITE]), c);
        assert_eq!(fs::read(manifest).unwrap(), MEMBERS);
    }

    #[test]
    #[ignore = "requires ZIRCON_TEXTURE_TEST_ROOT and zircon_runtime/profiling; retains disk specimens"]
    fn array_source_contract_row_slices_use_captured_png_before_changed_disk() {
        require_profiling();
        const SLICED: &str = "a.zarray";
        const SHEET: &str = "sheet.png";
        const ROWS: &[u8] = b"source = \"sheet.png\"\nrow_count = 2\n";
        let sheet = |top: [u8; 4]| {
            let mut bytes = Vec::new();
            PngEncoder::new(&mut bytes)
                .write_image(&[top, GREEN].concat(), 1, 2, ExtendedColorType::Rgba8)
                .unwrap();
            bytes
        };
        let root = new_run().join("p");
        let old_png = sheet(RED);
        let (_, assets) = fixture(&root, &[(SLICED, ROWS), (SHEET, &old_png)]);
        let (manager, work) = observe("row-slice-first-open", || {
            open_existing(&root, &assets, &[SLICED, SHEET])
        });
        assert_eq!(work, [2.0, 0.0, 2.0]);
        assert!(manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(5)));
        let before = bake(&manager, "row-slice-first-open", SLICED, &[RED, GREEN]);

        let array = assets.join(SLICED);
        let source = assets.join(SHEET);
        let context = AssetImportContext::new(
            array.clone(),
            uri(SLICED),
            fs::read(&array).unwrap(),
            Default::default(),
        );
        let captured = fs::read(&source).unwrap();
        assert_eq!(captured, old_png);
        let new_png = sheet(BLUE);
        fs::write(&source, &new_png).unwrap();
        assert_eq!(fs::read(&source).unwrap(), new_png);

        // Deliberately supply the preceding PNG bytes via the public context setter.
        // This tests the registered decoder's input seam, not mid-scan filesystem timing.
        let report = plugin_registration();
        assert!(report.is_success(), "{:?}", report.diagnostics);
        let handler = report.extensions.asset_importers().select(&array).unwrap();
        assert_eq!(handler.descriptor().id, ARRAY_IMPORTER);
        let read = |context: &AssetImportContext| {
            let outcome = handler.import(context).unwrap();
            as_texture(outcome.root_entry().unwrap().asset.clone())
        };
        pixels(
            &read(&context),
            &uri(SLICED),
            &[BLUE, GREEN],
            "disk control",
        );
        let context = context.with_source_file_snapshots(BTreeMap::from([(source, captured)]));
        pixels(
            &read(&context),
            &uri(SLICED),
            &[RED, GREEN],
            "captured bytes",
        );

        let work = scan(&manager, "row-slice-new-capture", 2);
        let after = bake(&manager, "row-slice-new-capture", SLICED, &[BLUE, GREEN]);
        assert_rebaked(&before, &after);
        assert_eq!(work, [2.0, 0.0, 2.0]);
        assert_eq!(fs::read(array).unwrap(), ROWS);
    }

    fn fixture(root: &Path, sources: &[(&str, &[u8])]) -> (ProjectPaths, PathBuf) {
        fs::create_dir(root).expect("create new qualification project");
        let paths = ProjectPaths::from_root(root).unwrap();
        assert_eq!(paths.root(), root);
        let manifest =
            ProjectManifest::new("Array source qualification", uri("main.scene.toml"), 1);
        paths.ensure_layout(&manifest.asset_roots).unwrap();
        manifest.save(paths.manifest_path()).unwrap();
        let assets = manifest.primary_asset_root_path(&paths).unwrap();
        for &(name, bytes) in sources {
            fs::write(assets.join(name), bytes).unwrap();
        }
        assert!(fs::read_dir(paths.asset_artifact_root())
            .unwrap()
            .next()
            .is_none());
        (paths, assets)
    }

    fn open_existing(root: &Path, assets: &Path, sources: &[&str]) -> ProjectAssetManager {
        let report = plugin_registration();
        assert!(report.is_success(), "{:?}", report.diagnostics);
        let registry = report.extensions.asset_importers();
        let manager = ProjectAssetManager::default();
        for handler in registry.importers() {
            manager.register_asset_importer_arc(handler).unwrap();
        }
        AssetManager::open_project(&manager, root.to_str().expect("UTF-8 project path")).unwrap();
        for name in sources {
            let path = assets.join(name);
            let selected = manager
                .asset_importer_capability_report_for_source(&path)
                .unwrap();
            assert_eq!(
                selected,
                registry.capability_report_for_source(&path).unwrap()
            );
            assert_eq!(selected.descriptor.id, importer(name));
            assert_eq!(selected.descriptor.output_kind, AssetKind::Texture);
            assert!(selected.status.is_available(), "{selected:?}");
        }
        manager
    }

    fn close(manager: ProjectAssetManager, root: &Path) {
        assert_eq!(
            AssetManager::close_project(&manager).unwrap().as_deref(),
            Some(root)
        );
        assert!(manager.current_project_manager().is_none());
    }

    fn require_profiling() {
        assert!(
            profiling::feature_enabled(),
            "run with --features zircon_runtime/profiling"
        );
    }

    fn observe<T>(stage: &str, action: impl FnOnce() -> T) -> (T, [f64; 3]) {
        // The process-wide recorder is shared. The qualification runner also uses --test-threads 1.
        static CAPTURE: Mutex<()> = Mutex::new(());
        let _exclusive = CAPTURE.lock().unwrap_or_else(|err| err.into_inner());
        assert!(
            !profiling::capture_active(),
            "{stage}: recorder is already active"
        );
        let status = profiling::start_capture(profiling::ProfileCaptureConfig {
            session_id: stage.to_owned(),
            max_counters: 4096,
            ..Default::default()
        });
        assert!(status.active, "{stage}: {status:?}");
        struct ResetCapture;
        impl Drop for ResetCapture {
            fn drop(&mut self) {
                profiling::reset_capture();
            }
        }
        let _reset = ResetCapture;
        let result = action();
        profiling::stop_capture();
        let snapshot = profiling::snapshot();
        assert_eq!(snapshot.session_id, stage);
        let [retention] = snapshot.recorder_retention.as_slice() else {
            panic!("{stage}: expected a single recorder");
        };
        assert_eq!(
            retention.counters.overwritten, 0,
            "{stage}: lost counter samples"
        );
        let counter = |suffix| {
            let name = format!("asset.project_generation.{suffix}");
            let mut samples = snapshot
                .counters
                .iter()
                .filter(|s| s.stream == "asset" && s.name == name);
            let sample = samples
                .next()
                .unwrap_or_else(|| panic!("{stage}: missing {name}"));
            assert!(
                samples.next().is_none(),
                "{stage}: multiple samples for {name}"
            );
            sample.value
        };
        let work = [
            counter("source_count"),
            counter("restored_source_count"),
            counter("imported_source_count"),
        ];
        let artifacts = counter("artifact_count");
        eprintln!("array-source contract {stage} [sources, restored, imported]={work:?} artifacts={artifacts}");
        assert_eq!(
            counter("commit_succeeded"),
            1.0,
            "{stage}: generation not committed"
        );
        assert_eq!(
            counter("failed_source_count"),
            0.0,
            "{stage}: failed source"
        );
        assert_eq!(
            work[0],
            work[1] + work[2],
            "{stage}: incomplete generation accounting"
        );
        assert_eq!(
            artifacts, work[2],
            "{stage}: one artifact per freshly imported fixture source"
        );
        (result, work)
    }

    fn scan(manager: &ProjectAssetManager, stage: &str, sources: usize) -> [f64; 3] {
        let (rows, work) = observe(stage, || AssetManager::reimport_all(manager).unwrap());
        assert_eq!(rows.len(), sources, "{stage}: explicit full scan");
        work
    }

    fn importer(name: &str) -> &str {
        if name.ends_with(".zarray") {
            ARRAY_IMPORTER
        } else {
            PNG_IMPORTER
        }
    }

    fn bake(manager: &ProjectAssetManager, stage: &str, name: &str, expected: &[[u8; 4]]) -> Bake {
        let project = manager.current_project_manager().unwrap();
        let paths = project.paths();
        let assets = project.manifest().primary_asset_root_path(paths).unwrap();
        let res = uri(name);
        let row = project.registry().get_by_locator(&res).unwrap();
        let meta = AssetMetaDocument::load(assets.join(format!("{name}.zmeta"))).unwrap();
        assert_eq!(row.state, ResourceState::Ready, "{stage} {res}: {row:?}");
        assert_eq!(meta.preview_state, PreviewState::Ready);
        assert_eq!(meta.url, res);
        assert_eq!(meta.asset_kind, AssetKind::Texture);
        assert_eq!(row.id(), AssetId::from_asset_uuid(meta.uuid));
        assert_eq!(row.kind, meta.asset_kind);
        assert_eq!(meta.importer_id, importer(name));
        assert_eq!(row.importer_id, meta.importer_id);
        assert_eq!(row.importer_version, meta.importer_version);
        assert_eq!(row.source_hash, meta.source_digest);
        assert_eq!(row.config_hash, meta.config_hash);
        let lib = meta.artifact_locator.as_ref().unwrap();
        assert_eq!(row.artifact_locator(), Some(lib));
        assert_eq!(lib.scheme(), ResourceScheme::Library);
        let store = ArtifactStore::default();
        let inventory = store.open_chunk_inventory(paths, lib).unwrap();
        assert_eq!(inventory.kind(), AssetKind::Texture);
        let digest = inventory.content_hash().to_owned();
        let texture = as_texture(store.read(paths, lib).unwrap());
        assert_eq!(texture, as_texture(project.load_artifact(&res).unwrap()));
        eprintln!("array-source contract {stage} {res} uuid={} source={} key={} {lib} digest={digest} rgba={:02x?}", meta.uuid, meta.source_digest, meta.config_hash, texture.rgba);
        pixels(&texture, &res, expected, stage);
        Bake {
            meta,
            digest,
            texture,
        }
    }

    fn pixels(texture: &TextureAsset, res: &AssetUri, expected: &[[u8; 4]], stage: &str) {
        let descriptor = texture.texture_descriptor();
        assert_eq!(&texture.uri, res);
        assert_eq!(texture.payload, TexturePayload::Rgba8);
        assert_eq!(
            (texture.width, texture.height, descriptor.mip_count),
            (1, 1, 1)
        );
        assert_eq!(descriptor.dimension, RenderImageDimension::D2);
        assert_eq!(descriptor.format, RGBA8_UNORM_SRGB_FORMAT);
        assert_eq!(descriptor.depth_or_array_layers as usize, expected.len());
        assert_eq!(
            texture.rgba.as_slice(),
            expected.as_flattened(),
            "{stage} {res}: layer RGBA"
        );
    }

    fn assert_rebaked(before: &Bake, after: &Bake) {
        assert_eq!(before.meta.uuid, after.meta.uuid);
        assert_eq!(before.meta.url, after.meta.url);
        assert_eq!(before.meta.importer_id, after.meta.importer_id);
        assert_eq!(before.meta.importer_version, after.meta.importer_version);
        assert_ne!(before.meta.source_digest, after.meta.source_digest);
        assert_ne!(before.meta.config_hash, after.meta.config_hash);
        assert_ne!(before.digest, after.digest);
    }
}

#[test]
#[ignore = "requires a fresh owned ZIRCON_TEXTURE_TEST_ROOT with the retained R19 project staged"]
fn array_source_migration_reopens_the_retained_project() {
    const REPARSE_POINT: u32 = 0x400;
    let run = new_run();
    let root = run.parent().expect("validated audit root").join("upgrade_input");
    let entry = fs::symlink_metadata(&root).expect("executor-staged upgrade_input");
    assert!(entry.is_dir() && entry.file_attributes() & REPARSE_POINT == 0);
    let paths = ProjectPaths::from_root(&root).expect("physical upgrade_input project paths");
    assert_eq!(paths.root(), root);

    // Inspect the staged tree without following links, before any project open or recovery.
    let chunk_root = paths.asset_artifact_root().join("chunks");
    let mut pending = vec![root.clone()];
    let (mut files, mut chunks) = (0, 0);
    while let Some(path) = pending.pop() {
        let entry = fs::symlink_metadata(&path).expect("inspect staged project entry");
        assert!(
            entry.file_attributes() & REPARSE_POINT == 0 && !path.ends_with(".git"),
            "staged project must contain only ordinary entries: {path:?}"
        );
        if entry.is_dir() {
            for child in fs::read_dir(&path).expect("inspect staged project directory") {
                pending.push(child.expect("staged directory entry").path());
            }
        } else {
            assert!(entry.is_file(), "expected an ordinary staged file: {path:?}");
            files += 1;
            if path.parent() == Some(chunk_root.as_path())
                && path.extension().is_some_and(|ext| ext == "zchunk")
            {
                chunks += 1;
            }
        }
    }
    assert_eq!((files, chunks), (15, 4), "stage only the R19 whitelist");
    assert!(paths.registry_root().join("asset-registry.json").is_file());

    let project_bytes = fs::read(paths.manifest_path()).expect("copied project manifest");
    let manifest = ProjectManifest::load(paths.manifest_path()).expect("load copied manifest");
    let assets = manifest.primary_asset_root_path(&paths).unwrap();
    assert_eq!(manifest.asset_roots.len(), 1);
    assert_eq!(assets, root.join("assets"));
    let blue = encode_png(BLUE);
    let green = encode_png(GREEN);
    let check_inputs = || {
        check_png(&assets.join(EDITED), &blue, BLUE);
        check_png(&assets.join(UNCHANGED), &green, GREEN);
        assert_eq!(fs::read(assets.join(ARRAY)).unwrap(), MANIFEST);
        assert_eq!(fs::read(paths.manifest_path()).unwrap(), project_bytes);
    };
    check_inputs();

    let store = ArtifactStore::default();
    let before = SOURCES.map(|(name, importer, layers)| {
        let meta = AssetMetaDocument::load(&assets.join(format!("{name}.zmeta")))
            .expect("read persisted R19 sidecar without opening the project");
        assert_eq!(meta.preview_state, PreviewState::Ready, "before {name}");
        assert_eq!(meta.url, uri(name));
        assert_eq!(meta.asset_kind, AssetKind::Texture);
        assert_eq!(meta.importer_id, importer);
        assert!(meta.config_hash.starts_with("blake3:") && !meta.source_digest.is_empty());
        let lib = meta.artifact_locator.as_ref().expect("copied lib locator");
        assert_eq!(lib.scheme(), ResourceScheme::Library);
        assert!(lib.label().is_none());
        assert_eq!(
            lib.path(),
            format!("textures/{}.zasset", AssetId::from_asset_uuid(meta.uuid))
        );
        let inventory = store
            .open_chunk_inventory(&paths, lib)
            .expect("read copied inventory");
        let digest = inventory.content_hash().to_owned();
        let texture = as_texture(store.read(&paths, lib).expect("decode copied lib artifact"));
        assert_eq!(texture.uri, uri(name));
        assert_eq!(texture.payload, TexturePayload::Rgba8);
        assert_eq!((texture.width, texture.height), (1, 1));
        assert_eq!(texture.texture_descriptor().depth_or_array_layers, layers);
        Bake { meta, digest, texture }
    });
    assert_eq!(before[0].texture.rgba, BLUE);
    assert_eq!(before[1].texture.rgba, GREEN);
    let old_array = &before[2];
    assert_eq!(
        old_array.texture.rgba,
        [GREEN, RED].concat(),
        "staged old array"
    );

    let trace = |phase: &str, bake: &Bake| {
        eprintln!(
            "ARRAY_MIGRATION_V1 phase={phase} root={} project={} state={:?} uuid={} source={} action={} lib={} digest={} rgba={:02x?}",
            ProjectPaths::display_path(&root).display(), manifest.project_guid,
            bake.meta.preview_state, bake.meta.uuid, bake.meta.source_digest, bake.meta.config_hash,
            bake.meta.artifact_locator.as_ref().expect("array lib locator"), bake.digest,
            bake.texture.rgba,
        );
    };
    trace("before", old_array);

    let report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let registry = report.extensions.asset_importers();
    let manager = ProjectAssetManager::default();
    for handler in registry.importers() {
        manager
            .register_asset_importer_arc(handler)
            .expect("attach actual texture handler");
    }
    let check_handlers = || {
        for ((name, id, _), old) in SOURCES.into_iter().zip(&before) {
            let path = assets.join(name);
            let installed = manager
                .asset_importer_capability_report_for_source(&path)
                .unwrap();
            assert_eq!(installed, registry.capability_report_for_source(&path).unwrap());
            assert_eq!(installed.descriptor.id, id);
            assert_eq!(installed.descriptor.output_kind, AssetKind::Texture);
            assert_eq!(
                installed.descriptor.importer_version,
                old.meta.importer_version
            );
            assert!(installed.status.is_available(), "{installed:?}");
        }
    };
    check_handlers();
    AssetManager::open_project(&manager, root.to_str().expect("UTF-8 upgrade_input"))
        .expect("open the retained project and commit its first scan");
    assert!(
        manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(5)),
        "could not stop and join the project's watchers"
    );
    check_handlers();
    let active = manager.current_project_manager().expect("active migration input");
    assert_eq!(active.paths(), &paths);
    assert_eq!(active.manifest(), &manifest);
    drop(active);

    let after = read_bakes(&manager, "migration-after-open");
    check_inputs();
    for (old, new) in before.iter().zip(&after) {
        assert_eq!(old.meta.uuid, new.meta.uuid, "preserve each asset UUID");
        assert_eq!(old.meta.artifact_locator, new.meta.artifact_locator);
        assert_eq!(old.meta.format_version, new.meta.format_version);
        assert_eq!(old.meta.importer_version, new.meta.importer_version);
    }
    let [png, other, new_array] = &after;
    assert_eq!(png.texture.rgba, BLUE, "blue PNG was already current");
    assert_eq!(other.texture.rgba, GREEN, "green PNG remains current");
    trace("after-open", new_array);
    assert_eq!(
        new_array.texture.rgba, [GREEN, BLUE].concat(),
        "first project open must migrate the persisted array from green/red to green/blue"
    );
    assert_ne!(old_array.meta.source_digest, new_array.meta.source_digest);
    assert_ne!(old_array.meta.config_hash, new_array.meta.config_hash);
    assert_ne!(old_array.digest, new_array.digest);
    trace("migrated", new_array);

    assert_eq!(full_scan(&manager, "migration-next-unchanged"), after);
    check_inputs();
    trace("verified", new_array);
}

#[test]
#[ignore = "requires an owned ZIRCON_TEXTURE_TEST_ROOT; retains disk specimens"]
fn array_source_generated_publication_keeps_old_generation_on_missing_layer() {
    use zircon_runtime::asset::pack::zrpack_content_hash;

    let run = new_run();
    let [red, blue, green] = [RED, BLUE, GREEN].map(encode_png);
    let (manager, assets) = open_project(&run.join("g"), &red, &green);
    let source = assets.join(ARRAY);
    let sidecar = assets.join(format!("{ARRAY}.zmeta"));
    let original = fs::read(&source).unwrap();
    assert_eq!(original, MANIFEST);
    let before = read_bakes(&manager, "generated-before");
    assert_colors(&before, RED);

    // The new manifest names a different layer set; the old disk manifest still names b.png.
    let blue_source = assets.join("c.png");
    fs::write(&blue_source, &blue).unwrap();
    check_png(&blue_source, &blue, BLUE);
    let generated: &[u8] = b"sources = [\"./c.png\", \"a.png\"]\n";
    let receipt = manager
        .publish_generated_project_source(uri(ARRAY), generated.to_vec())
        .expect("publish the prepared manifest, not the old disk manifest");
    let committed = fs::read(&source).expect("read the source after actual commit");
    assert_eq!(committed.as_slice(), generated);
    assert_eq!(receipt.source_uri(), &uri(ARRAY));
    assert_eq!(
        receipt.previous_source_hash().unwrap().as_bytes(),
        &zrpack_content_hash(&original)
    );
    assert_eq!(
        receipt.committed_source_hash().as_bytes(),
        &zrpack_content_hash(&committed)
    );
    let built = read_bakes(&manager, "generated-committed");
    assert_eq!(built[2].texture.rgba, [BLUE, RED].concat());
    assert_eq!(built[2].meta.uuid, before[2].meta.uuid);
    assert_ne!(built[2].meta.config_hash, before[2].meta.config_hash);
    let status = receipt.status().expect("published array status");
    assert!(status.imported);
    assert_eq!(status.config_hash, built[2].meta.config_hash);

    let generation = manager.current_project_generation_snapshot().unwrap();
    assert_eq!(
        receipt.generation(),
        generation.project().catalog_input_generation().sequence()
    );
    let sidecar_bytes = fs::read(&sidecar).unwrap();
    let registry = generation
        .project()
        .paths()
        .registry_root()
        .join("asset-registry.json");
    let registry_bytes = fs::read(&registry).unwrap();
    let missing = b"sources = [\"a.png\", \"absent-generated.png\"]\n";
    assert!(!assets.join("absent-generated.png").exists());
    let error = manager
        .publish_generated_project_source(uri(ARRAY), missing.to_vec())
        .expect_err("an unavailable layer must not return a successful source-publication receipt");
    assert!(error.to_string().contains("absent-generated.png"), "{error}");
    assert_eq!(fs::read(&source).unwrap(), committed);
    assert_eq!(fs::read(&sidecar).unwrap(), sidecar_bytes);
    assert_eq!(fs::read(&registry).unwrap(), registry_bytes);
    assert_eq!(
        manager.current_project_generation_snapshot().unwrap().into_parts().1,
        generation.into_parts().1
    );
    assert_eq!(read_bakes(&manager, "generated-rollback"), built);
}
