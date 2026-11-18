//! Check the published material option bit budget through actual import, cache read and reopen.

use super::*;
use crate::asset::{project::PreviewState, ArtifactStore};
use crate::core::framework::render::RenderShaderDefinitionValue;
use std::fmt::Write as _;

const DETAIL: &str = "detail_mode";
const DEFINE: &str = "ZR_OPT_DETAIL_MODE";
const IMPORTER: &str = "zircon.builtin.shader.zshader_package";

fn with_project(
    label: &str,
    flags: usize,
    check: impl FnOnce(&Path, &ProjectPaths, &ProjectManager, &AssetUri),
) {
    let root = unique_temp_project_root(label);
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    fs::create_dir(&root).expect("exclusive test project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let asset_root = zircon_runtime_interface::project::RelPath::project_assets();
    paths
        .ensure_layout(std::slice::from_ref(&asset_root))
        .unwrap();
    let uri = AssetUri::parse("res://shaders/budget").unwrap();
    ProjectManifest::new("OptionBudget", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let dir = paths.asset_root(&asset_root).join("shaders").join("budget");
    fs::create_dir_all(&dir).unwrap();
    let mut meta = AssetMetaDocument::new(
        AssetUuid::from_stable_label("option-budget-root"),
        uri.clone(),
        AssetKind::Shader,
    );
    meta.unit = AssetSourceUnit::Compound;
    meta.save(dir.with_extension("zmeta")).unwrap();
    let mut source = String::from("version=2\nkind=\"surface\"\nshading_model=\"unlit\"\nimport_path=\"material_test::budget\"\nwgsl_files=[\"budget.wgsl\"]\n");
    for index in 0..flags {
        writeln!(
            source,
            "[[options]]\nname=\"flag_{index:02}\"\nkind=\"bool\"\ndefault=false"
        )
        .unwrap();
    }
    writeln!(source, "[[options]]\nname=\"{DETAIL}\"\nkind=\"enum\"\ndefault=\"off\"\neditor={{values=\"off,uv,triplanar\"}}").unwrap();
    fs::write(dir.join("budget.zshader"), source).unwrap();
    fs::write(dir.join("budget.wgsl"), "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }\n@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }\n").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .scan_and_import()
        .expect("publish the scan, including any per-source import error");
    let record = manager
        .registry()
        .get_by_locator(&uri)
        .expect("registered shader source");
    assert_eq!(record.importer_id, IMPORTER);
    assert_eq!(record.id(), AssetId::from_asset_uuid(meta.uuid));
    check(&root, &paths, &manager, &uri);
    drop(manager);
    fs::remove_dir_all(root).unwrap();
}

fn load_shader(manager: &ProjectManager, uri: &AssetUri) -> ShaderAsset {
    let ImportedAsset::Shader(asset) = manager
        .load_artifact(uri)
        .expect("read persisted Shader payload")
    else {
        panic!("wrong payload type")
    };
    assert_eq!(asset.kind, ShaderAssetKind::Surface);
    assert!(
        asset.validation_diagnostics.is_empty(),
        "unexpected WGSL/provider diagnostics: {:?}",
        asset.validation_diagnostics
    );
    asset
}

fn observe(shader: &ShaderAsset) -> ([u32; 3], [RenderShaderDefinitionValue; 3]) {
    let option = shader
        .material_option_table
        .option(DETAIL)
        .expect("declared enum");
    let pairs = ["off", "uv", "triplanar"].map(|value| {
        let mut material = material_for_shader(&shader.uri);
        material
            .options
            .insert(DETAIL.into(), toml::Value::String(value.into()));
        let bits = material.material_option_bits_for_shader(shader);
        let define = shader
            .material_option_table
            .definition_values_for_bits(bits)
            .into_iter()
            .find(|d| d.name() == DEFINE)
            .unwrap();
        (bits, define)
    });
    let bits = pairs.each_ref().map(|p| p.0);
    let defines = pairs.map(|p| p.1);
    println!(
        "option-budget table={} offset={} width={} bits={bits:?} defines={defines:?}",
        shader.material_option_table.total_bits, option.bit_offset, option.bit_width
    );
    (bits, defines)
}

fn assert_boundary(shader: &ShaderAsset) {
    assert_eq!(shader.options.len(), 31);
    assert_eq!(shader.material_option_table.total_bits, 32);
    assert_eq!(
        observe(shader),
        (
            [0, 1 << 30, 1 << 31],
            [0, 1, 2].map(|v| RenderShaderDefinitionValue::uint(DEFINE, v))
        )
    );
}

fn named_budget_fault(message: &str) -> bool {
    message.contains(DETAIL) && message.contains("32")
}

#[test]
fn material_budget_non_material_document_options_are_not_a_surface_table() {
    for (kind, stage) in [("compute", "compute"), ("fullscreen", "fragment")] {
        let mut source = format!(
            "kind=\"{kind}\"\nversion=2\nentry_points=[{{name=\"main\",stage=\"{stage}\"}}]\n"
        );
        for i in 0..33 {
            writeln!(
                source,
                "[[options]]\nname=\"flag_{i:02}\"\nkind=\"bool\"\ndefault=false"
            )
            .unwrap();
        }
        let doc =
            ZShaderDocumentV2::from_toml_str(&source).expect("not a surface material permutation");
        assert!(!doc.kind().participates_in_material_variants());
        assert_eq!(doc.options().len(), 33);
    }
}

#[test]
fn material_budget_exact32_roundtrips_through_the_registered_importer() {
    with_project("budget32", 30, |_, _, manager, uri| {
        let record = manager.registry().get_by_locator(uri).unwrap();
        assert_eq!(record.state, ResourceState::Ready);
        assert!(record.artifact_locator().is_some());
        assert_boundary(&load_shader(manager, uri));
    });
}

#[test]
fn material_budget_overflow_is_not_a_ready_shader() {
    with_project("budget33", 31, |_, paths, manager, uri| {
        let record = manager.registry().get_by_locator(uri).unwrap();
        let meta = AssetMetaDocument::load(
            paths
                .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                .join("shaders/budget.zmeta"),
        )
        .unwrap();
        println!(
            "option-budget author state={:?} preview={:?} artifact={} diagnostics={:?}",
            record.state,
            meta.preview_state,
            record.artifact_locator().is_some(),
            record.diagnostics
        );
        if record.state == ResourceState::Ready {
            let shader = load_shader(manager, uri);
            observe(&shader);
        }
        assert_eq!(
            record.state,
            ResourceState::Error,
            "an unrepresentable named enum must not be published as Ready"
        );
        assert_eq!(meta.preview_state, PreviewState::Error);
        assert!(record.artifact_locator().is_none() && meta.artifact_locator.is_none());
        assert!(
            record
                .diagnostics
                .iter()
                .any(|d| named_budget_fault(&d.message)),
            "name the unrepresentable option and the budget: {:?}",
            record.diagnostics
        );
    });
}

// The source and source digests remain valid. Only the persisted root payload has 31 flags
// plus an enum, while its stale derived table still describes 30 flags plus an enum.
fn install_stale_payload(
    manager: &ProjectManager,
    paths: &ProjectPaths,
    uri: &AssetUri,
) -> (AssetUri, String) {
    let mut shader = load_shader(manager, uri);
    assert_boundary(&shader);
    let mut extra = shader.options[0].clone();
    extra.name = "flag_30".to_string();
    shader.options.insert(30, extra);
    let record = manager.registry().get_by_locator(uri).unwrap();
    let store = ArtifactStore::default();
    let locator = store
        .write(paths, record, &ImportedAsset::Shader(shader))
        .unwrap();
    assert_eq!(Some(&locator), record.artifact_locator());
    let disk = store.open_chunk_inventory(paths, &locator).unwrap();
    println!(
        "option-budget stored declared_bits=33 cached_table_bits=32 hash={} bytes={} chunks={}",
        disk.content_hash(),
        disk.raw_bytes(),
        disk.len()
    );
    (locator, disk.content_hash().to_string())
}

#[test]
fn material_budget_cache_reader_rejects_oversized_declaration() {
    with_project("budget_cache", 30, |_, paths, manager, uri| {
        let (locator, _) = install_stale_payload(manager, paths, uri);
        match ArtifactStore::default().read(paths, &locator) {
            Ok(ImportedAsset::Shader(shader)) => {
                observe(&shader);
                panic!("cache reader exposed an unrepresentable material option");
            }
            Ok(_) => panic!("wrong cached payload type"),
            Err(e) => {
                println!("option-budget cache error={e}");
                assert!(
                    named_budget_fault(&e.to_string()),
                    "unexpected cache error: {e}"
                )
            }
        }
    });
}

#[test]
fn material_budget_reopen_reimports_valid_source_after_bad_cache() {
    with_project("budget_reopen", 30, |root, paths, manager, uri| {
        let (locator, stale) = install_stale_payload(manager, paths, uri);
        let mut reopened = ProjectManager::open(root).unwrap();
        reopened.scan_and_import().unwrap();
        let record = reopened.registry().get_by_locator(uri).unwrap();
        assert_eq!(record.state, ResourceState::Ready);
        assert_eq!(record.artifact_locator(), Some(&locator));
        let shader = load_shader(&reopened, uri);
        let disk = ArtifactStore::default()
            .open_chunk_inventory(paths, &locator)
            .unwrap();
        println!(
            "option-budget reopened state={:?} table={} hash={} stale={stale}",
            record.state,
            shader.material_option_table.total_bits,
            disk.content_hash()
        );
        assert_ne!(
            disk.content_hash(),
            stale,
            "the corrupt root payload survived a real project scan"
        );
        assert_boundary(&shader);
    });
}
