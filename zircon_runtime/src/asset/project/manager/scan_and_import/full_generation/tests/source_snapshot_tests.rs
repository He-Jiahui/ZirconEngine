use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use crate::asset::project::{
    AssetMetaDocument, AssetSourceUnit, ProjectManager, ProjectManifest, ProjectPaths,
};
use crate::asset::{AssetImportContext, AssetKind, AssetUri, AssetUuid, ImportedAsset};
use crate::core::resource::ResourceState;

// 两个测试钩子按线程隔离；完整代在上下文携带源快照后调用观察器，以模拟随后发生的文件变化。
// Fixture::drop 清除观察器与预算覆盖，避免钩子泄漏到同线程的后续测试。
thread_local! {
    static SOURCE_OBSERVER: RefCell<Option<Box<dyn FnMut(&AssetImportContext)>>> = RefCell::new(None);
    static BYTE_LIMIT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

pub(in crate::asset::project::manager::scan_and_import) fn byte_limit(default: u64) -> u64 {
    BYTE_LIMIT.with(|limit| limit.get().unwrap_or(default))
}

pub(super) fn observe(context: &AssetImportContext) {
    SOURCE_OBSERVER.with(|observer| {
        if let Some(observer) = observer.borrow_mut().as_mut() {
            observer(context);
        }
    });
}

struct Fixture {
    root: PathBuf,
    assets: PathBuf,
    paths: ProjectPaths,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-snapshot-product-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let paths = ProjectPaths::from_root(&root).unwrap();
        let asset_root = zircon_runtime_interface::project::RelPath::project_assets();
        paths.ensure_layout(&[asset_root.clone()]).unwrap();
        ProjectManifest::new(
            "Snapshot Regression",
            AssetUri::parse("res://model.gltf").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        Self {
            root,
            assets: paths.asset_root(&asset_root),
            paths,
        }
    }

    fn model(&self, x: f32) {
        fs::write(self.assets.join("model.gltf"), br#"{
            "asset":{"version":"2.0"},
            "buffers":[{"uri":"aaa.bin","byteLength":36}],
            "bufferViews":[{"buffer":0,"byteLength":36}],
            "accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[10,1,0]}],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}]
        }"#).unwrap();
        self.buffer(x);
    }

    fn buffer(&self, x: f32) {
        let bytes = [x, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(self.assets.join("aaa.bin"), bytes).unwrap();
    }

    fn digest(&self, name: &str) -> String {
        AssetMetaDocument::load(self.assets.join(name))
            .unwrap()
            .source_digest
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        SOURCE_OBSERVER.with(|observer| *observer.borrow_mut() = None);
        BYTE_LIMIT.with(|limit| limit.set(None));
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn first_position(manager: &ProjectManager) -> f32 {
    let ImportedAsset::Model(model) = manager
        .load_artifact(&AssetUri::parse("res://model.gltf").unwrap())
        .unwrap()
    else {
        panic!("model artifact required")
    };
    // BUG: [CR-ASSET-PROJECT-SNAPSHOT-0001] 默认 glTF 根 primitive 仅保存 mesh 引用、vertices 为空；工件恢复仍保留空数组，此首顶点索引会 panic。
    model.primitives[0].vertices[0].position[0]
}

#[test]
fn project_gltf_external_only_change_reimports_and_repeats_digest() {
    let fixture = Fixture::new("gltf-change");
    fixture.model(0.0);
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();
    let original = fixture.digest("model.gltf.zmeta");
    assert_eq!(first_position(&manager), 0.0);
    fixture.buffer(0.5);
    manager.scan_and_import().unwrap();
    let changed = fixture.digest("model.gltf.zmeta");
    assert_ne!(original, changed);
    assert_eq!(first_position(&manager), 0.5);
    manager.scan_and_import().unwrap();
    assert_eq!(fixture.digest("model.gltf.zmeta"), changed);
}

#[test]
fn project_gltf_auxiliary_snapshot_respects_the_generation_cumulative_budget() {
    let fixture = Fixture::new("gltf-cumulative-budget");
    fixture.model(0.0);
    let source_bytes = fs::metadata(fixture.assets.join("model.gltf"))
        .unwrap()
        .len();
    let auxiliary_bytes = fs::metadata(fixture.assets.join("aaa.bin")).unwrap().len();
    BYTE_LIMIT.with(|limit| limit.set(Some(source_bytes + auxiliary_bytes - 1)));

    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();

    let record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://model.gltf").unwrap())
        .unwrap();
    assert_eq!(record.state, ResourceState::Error);
    assert!(record
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("limit")));
}

#[test]
fn project_gltf_import_uses_bytes_captured_before_live_deletion() {
    let fixture = Fixture::new("gltf-race");
    fixture.model(0.25);
    #[cfg(windows)]
    {
        let path = fixture.assets.join("model.gltf");
        let source = fs::read_to_string(&path)
            .unwrap()
            .replace("aaa.bin", "AAA.BIN");
        fs::write(path, source).unwrap();
    }
    let buffer = fixture.assets.join("aaa.bin");
    SOURCE_OBSERVER.with(|observer| {
        *observer.borrow_mut() = Some(Box::new(move |context| {
            if context.uri.path() == "model.gltf" {
                fs::remove_file(&buffer).unwrap();
            }
        }));
    });
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();
    assert_eq!(first_position(&manager), 0.25);
}

#[test]
fn project_external_snapshot_change_rolls_back_on_commit_failure() {
    let fixture = Fixture::new("gltf-rollback");
    fixture.model(0.0);
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();
    let meta_path = fixture.assets.join("model.gltf.zmeta");
    let registry_path = fixture.paths.registry_root().join("asset-registry.json");
    let meta_before = fs::read(&meta_path).unwrap();
    let registry_before = fs::read(&registry_path).unwrap();
    fixture.buffer(0.75);
    assert!(manager
        .scan_and_import_with_commit_failure_before_registry(None)
        .is_err());
    assert_eq!(fs::read(meta_path).unwrap(), meta_before);
    assert_eq!(fs::read(registry_path).unwrap(), registry_before);
    assert_eq!(first_position(&manager), 0.0);
    manager.scan_and_import().unwrap();
    assert_eq!(first_position(&manager), 0.75);
}

#[test]
fn project_invalid_gltf_preflight_records_error_and_keeps_other_assets_ready() {
    let fixture = Fixture::new("invalid-preflight");
    fs::write(fixture.assets.join("model.gltf"), b"invalid gltf").unwrap();
    fs::write(fixture.assets.join("good.json"), b"{\"answer\":42}").unwrap();
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();
    for (uri, state) in [
        ("res://model.gltf", ResourceState::Error),
        ("res://good.json", ResourceState::Ready),
    ] {
        assert_eq!(
            manager
                .registry()
                .get_by_locator(&AssetUri::parse(uri).unwrap())
                .unwrap()
                .state,
            state
        );
    }
}

#[test]
#[cfg(any(feature = "graphics", feature = "target-server"))]
fn project_shader_parent_source_change_reimports_and_compound_digest_stabilizes() {
    let fixture = Fixture::new("shader-parent");
    let package = fixture.assets.join("shaders/package");
    fs::create_dir_all(&package).unwrap();
    fs::create_dir_all(fixture.assets.join("shared")).unwrap();
    let uri = AssetUri::parse("res://shaders/package").unwrap();
    let meta_path = fixture.assets.join("shaders/package.zmeta");
    let mut meta = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Shader);
    meta.unit = AssetSourceUnit::Compound;
    meta.save(&meta_path).unwrap();
    fs::write(package.join("package.zshader"), "kind = \"include\"\nversion = 2\nimport_path = \"project::snapshot\"\nwgsl_files = [\"../../shared/source.wgsl\"]\n").unwrap();
    let wgsl = fixture.assets.join("shared/source.wgsl");
    fs::write(&wgsl, "fn snapshot_value() -> f32 { return 1.0; }").unwrap();
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();
    let original = fixture.digest("shaders/package.zmeta");
    fs::write(&wgsl, "fn snapshot_value() -> f32 { return 2.0; }").unwrap();
    manager.scan_and_import().unwrap();
    let changed = fixture.digest("shaders/package.zmeta");
    assert_ne!(original, changed);
    let ImportedAsset::Shader(shader) = manager.load_artifact(&uri).unwrap() else {
        panic!("shader artifact required")
    };
    assert!(shader.wgsl_source.contains("return 2.0"));
    manager.scan_and_import().unwrap();
    assert_eq!(fixture.digest("shaders/package.zmeta"), changed);
}

#[test]
#[cfg(any(feature = "graphics", feature = "target-server"))]
fn project_large_compound_retains_one_member_copy_and_enforces_cumulative_budget() {
    let fixture = Fixture::new("compound-budget");
    let package = fixture.assets.join("package");
    fs::create_dir_all(&package).unwrap();
    let uri = AssetUri::parse("res://package").unwrap();
    let mut meta = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Shader);
    meta.unit = AssetSourceUnit::Compound;
    meta.save(fixture.assets.join("package.zmeta")).unwrap();
    fs::write(package.join("package.zshader"), "kind = \"include\"\nversion = 2\nimport_path = \"project::large\"\nwgsl_files = [\"main.wgsl\"]\n").unwrap();
    fs::write(
        package.join("main.wgsl"),
        "fn large_value() -> f32 { return 1.0; }",
    )
    .unwrap();
    const MEMBER_BYTES: usize = 256 * 1024;
    const MEMBERS: usize = 16;
    for index in 0..MEMBERS {
        fs::write(
            package.join(format!("padding-{index:02}.bin")),
            vec![index as u8; MEMBER_BYTES],
        )
        .unwrap();
    }
    let observed = std::rc::Rc::new(std::cell::Cell::new(false));
    let captured = observed.clone();
    SOURCE_OBSERVER.with(|observer| {
        *observer.borrow_mut() = Some(Box::new(move |context| {
            if context.uri.path() == "package" && context.has_source_file_snapshots() {
                let retained = context
                    .source_file_snapshot_paths()
                    .map(|path| context.source_file_snapshot(path).unwrap().len())
                    .sum::<usize>();
                assert!(retained >= MEMBERS * MEMBER_BYTES);
                assert!(retained < MEMBERS * MEMBER_BYTES + 4096);
                assert!(
                    context.source_bytes.len() < 64 * 1024,
                    "primary descriptor must not contain an aggregate member copy"
                );
                captured.set(true);
            }
        }));
    });
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager.scan_and_import().unwrap();
    assert!(observed.get());
    let first = fixture.digest("package.zmeta");
    manager.scan_and_import().unwrap();
    assert_eq!(fixture.digest("package.zmeta"), first);
    BYTE_LIMIT.with(|limit| limit.set(Some(2 * 1024 * 1024)));
    manager.scan_and_import().unwrap();
    let record = manager.registry().get_by_locator(&uri).unwrap();
    assert_eq!(record.state, ResourceState::Error);
    assert!(record
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("limit")));
}
