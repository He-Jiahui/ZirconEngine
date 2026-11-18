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
