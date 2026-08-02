use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

use crate::asset::project::{AssetMetaDocument, AssetSourceUnit, ProjectManifest, ProjectPaths};
use crate::asset::{AssetKind, AssetUri, AssetUuid};
use crate::core::framework::render::{
    shader_ide_preview_relative_path, shader_ide_preview_segments_relative_path,
    ShaderIdeModuleMap, ShaderIdePreviewMap, ShaderIdePreviewVariant, ShaderPassType,
    SHADER_IDE_PREVIEW_DEFAULT_VARIANT,
};

use super::*;

#[test]
fn shader_ide_preview_matrix_builds_the_shader_index_once() {
    let source = include_str!("../ide_env_generation.rs");
    let start = source
        .find("fn shader_preview_files(")
        .expect("preview batch builder");
    let end = start
        + source[start..]
            .find("fn shader_preview_file(")
            .expect("single preview projection");
    let batch = &source[start..end];

    assert_eq!(
        batch
            .matches("shader_include_index(shaders.iter())")
            .count(),
        1
    );
    assert!(batch.contains("assemble_shader_ide_surface_preview_with_index"));
    assert!(!batch.contains(concat!(
        "assemble_shader_ide_surface_preview(shader, ",
        "shaders.iter(), variant)"
    )));
}

#[test]
fn shader_ide_stub_validation_rejects_circular_dependencies() {
    let stubs = [
        shader_ide_dependency_test_stub("project::a", "project::b"),
        shader_ide_dependency_test_stub("project::b", "project::a"),
    ];

    let error = shader_ide_stub_validation_source(&stubs[0], &stubs)
        .expect_err("circular Shader IDE dependencies must fail validation");

    assert!(error.contains("circular shader IDE dependency"), "{error}");
    assert!(
        error.contains("project::a -> project::b -> project::a"),
        "{error}"
    );
}

#[test]
fn shader_ide_lightmap_stub_resolves_irradiance_volume_dependency() {
    let stubs = builtin_stubs();
    let lightmap = stubs
        .iter()
        .find(|stub| stub.entry.import_path == "zr_lightmap.wgsl")
        .expect("builtin lightmap stub should exist");
    let source = shader_ide_stub_validation_source(lightmap, &stubs)
        .expect("lightmap Shader IDE dependencies should resolve");

    assert!(
        source.contains("// Zircon shader IDE validation dependency: zr_irradiance_volume.wgsl")
    );
    parse_shader_ide_wgsl_module(&lightmap.entry.import_path, &source)
        .expect("lightmap stub should parse with irradiance-volume dependency");
}

fn shader_ide_dependency_test_stub(import_path: &str, dependency: &str) -> ShaderIdeStub {
    let stub_path = format!("modules/{}.wgsl", import_path.replace("::", "/"));
    ShaderIdeStub {
        relative_path: PathBuf::from(&stub_path),
        source: format!(
            "fn {}_value() -> f32 {{ return 1.0; }}",
            import_path.replace("::", "_")
        ),
        include_paths: vec![dependency.to_string()],
        validation_defines: Vec::new(),
        entry: ShaderIdeModuleMapEntry {
            import_path: import_path.to_string(),
            scope_uri: None,
            kind: ShaderAssetKind::Include,
            stub_path,
            source_uri: None,
            source_files: Vec::new(),
            content_hash: import_path.to_string(),
            generated: false,
        },
    }
}

#[test]
fn shader_ide_standard_pbr_stub_resolves_registered_advanced_lighting_dependencies() {
    let stubs = builtin_stubs();
    let standard_pbr = stubs
        .iter()
        .find(|stub| stub.entry.import_path == "zr_shading_standard_pbr.wgsl")
        .expect("builtin Standard PBR stub should exist");
    let source = shader_ide_stub_validation_source(standard_pbr, &stubs)
        .expect("Standard PBR Shader IDE dependencies should resolve");

    for dependency in ["zr_pbr_extras.wgsl", "zr_light_cookie.wgsl"] {
        assert!(source.contains(&format!(
            "// Zircon shader IDE validation dependency: {dependency}"
        )));
    }
    parse_shader_ide_wgsl_module(&standard_pbr.entry.import_path, &source)
        .expect("Standard PBR stub should parse with transitive advanced-lighting dependencies");
}

#[test]
fn shader_ide_env_writes_module_map_stubs_and_generated_material() {
    let root = unique_temp_project_root("shader_ide_env_module_map");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Ide Sandbox",
        AssetUri::parse("res://shaders/hero").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_include_shader_package(&paths);
    write_surface_shader_package(&paths);

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let out_dir = root.join("ide-out");
    let report = write_shader_ide_env_for_project(&project, Some(&out_dir), &[]).unwrap();

    assert_eq!(report.shader_count, 2);
    assert_eq!(report.generated_material_count, 1);
    assert_eq!(report.naga_parsed_stub_count, report.module_count);
    assert_eq!(report.naga_validated_preview_count, 0);
    assert_eq!(report.managed_file_count, report.written_file_count);
    assert_eq!(report.removed_stale_file_count, 0);
    let module_map_text = fs::read_to_string(out_dir.join("module_map.json")).unwrap();
    let module_map: ShaderIdeModuleMap = serde_json::from_str(&module_map_text).unwrap();
    assert_eq!(module_map.project_name, "Shader Ide Sandbox");
    assert!(module_map.entries.iter().any(|entry| {
        entry.import_path == "shader_ide_sandbox::hero"
            && entry.stub_path == "modules/shader_ide_sandbox/hero.wgsl"
            && !entry.generated
    }));
    let generated = module_map
        .entries
        .iter()
        .find(|entry| entry.generated)
        .expect("generated material map entry");
    assert_eq!(generated.import_path, GENERATED_MATERIAL_MODULE_IMPORT_PATH);
    assert_eq!(
        generated.scope_uri.as_ref().map(ToString::to_string),
        Some("res://shaders/hero".to_string())
    );
    assert!(out_dir.join(&generated.stub_path).exists());
    assert!(fs::read_to_string(out_dir.join(&generated.stub_path))
        .unwrap()
        .contains("zr_mat_base_color"));
    assert!(
        fs::read_to_string(out_dir.join("modules/shader_ide_sandbox/hero.wgsl"))
            .unwrap()
            .contains("fn zr_material_surface")
    );

    let second_report = write_shader_ide_env_for_project(&project, Some(&out_dir), &[]).unwrap();
    assert_eq!(report.managed_file_count, second_report.managed_file_count);
    assert_eq!(
        second_report.naga_parsed_stub_count,
        second_report.module_count
    );
    assert_eq!(second_report.naga_validated_preview_count, 0);
    assert_eq!(second_report.written_file_count, 0);
    assert_eq!(second_report.removed_stale_file_count, 0);
    assert_eq!(
        module_map_text,
        fs::read_to_string(out_dir.join("module_map.json")).unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_ide_env_writes_default_preview_and_segments_when_variants_enabled() {
    let root = unique_temp_project_root("shader_ide_env_preview");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Ide Sandbox",
        AssetUri::parse("res://shaders/hero").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_include_shader_package(&paths);
    write_surface_shader_package(&paths);

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let out_dir = root.join("ide-out");
    let variants = [ShaderIdePreviewVariant::default_forward()];
    let report = write_shader_ide_env_for_project(&project, Some(&out_dir), &variants).unwrap();

    assert_eq!(report.shader_count, 2);
    assert_eq!(report.preview_count, 1);
    assert_eq!(report.naga_parsed_stub_count, report.module_count);
    assert_eq!(report.naga_validated_preview_count, 1);
    let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
    let preview_path =
        shader_ide_preview_relative_path(&shader_uri, SHADER_IDE_PREVIEW_DEFAULT_VARIANT);
    let segment_path =
        shader_ide_preview_segments_relative_path(&shader_uri, SHADER_IDE_PREVIEW_DEFAULT_VARIANT);
    let preview = fs::read_to_string(out_dir.join(&preview_path)).unwrap();
    assert!(preview.contains("// include: zr_template_forward.wgsl"));
    assert!(preview.contains("fn zr_material_surface"));

    let segment_map: ShaderIdePreviewMap =
        serde_json::from_str(&fs::read_to_string(out_dir.join(segment_path)).unwrap()).unwrap();
    assert_eq!(segment_map.shader_uri, shader_uri);
    assert_eq!(segment_map.variant, SHADER_IDE_PREVIEW_DEFAULT_VARIANT);
    assert!(segment_map
        .segments
        .iter()
        .any(|segment| segment.module_id == GENERATED_MATERIAL_MODULE_IMPORT_PATH));
    assert!(segment_map
        .segments
        .iter()
        .any(|segment| segment.module_id == "shader_ide_sandbox::hero"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_ide_env_writes_non_default_preview_variants_with_option_bits() {
    let root = unique_temp_project_root("shader_ide_env_non_default_preview");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Ide Sandbox",
        AssetUri::parse("res://shaders/hero").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_option_surface_shader_package(&paths);

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let out_dir = root.join("ide-out");
    let variants = [
        ShaderIdePreviewVariant::default_forward(),
        ShaderIdePreviewVariant::new(ShaderPassType::GBuffer, 1),
    ];
    let report = write_shader_ide_env_for_project(&project, Some(&out_dir), &variants).unwrap();

    assert_eq!(report.shader_count, 1);
    assert_eq!(report.preview_count, 2);
    assert_eq!(report.naga_parsed_stub_count, report.module_count);
    assert_eq!(report.naga_validated_preview_count, 2);
    let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
    let gbuffer_variant = &variants[1];
    let preview_path = shader_ide_preview_relative_path(&shader_uri, &gbuffer_variant.name);
    let segment_path =
        shader_ide_preview_segments_relative_path(&shader_uri, &gbuffer_variant.name);
    let preview = fs::read_to_string(out_dir.join(&preview_path)).unwrap();
    assert!(preview.contains("// include: zr_template_gbuffer.wgsl"));
    assert!(preview.contains("const ZR_OPT_ENABLE_RIM: bool = true;"));

    let segment_map: ShaderIdePreviewMap =
        serde_json::from_str(&fs::read_to_string(out_dir.join(segment_path)).unwrap()).unwrap();
    assert_eq!(segment_map.variant, "gbuffer_options_0x00000001");
    assert_eq!(
        segment_map.wgsl_path,
        "preview/res_shaders_hero.gbuffer_options_0x00000001.wgsl"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_ide_env_batches_preview_matrix_for_all_surface_shaders() {
    let root = unique_temp_project_root("shader_ide_env_preview_matrix");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Ide Sandbox",
        AssetUri::parse("res://shaders/hero").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_option_surface_shader_package(&paths);
    write_named_option_surface_shader_package(
        &paths,
        "rival",
        "res://shaders/rival",
        "0.2, 0.6, 1.0, 1.0",
    );

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let out_dir = root.join("ide-out");
    let variants = [
        ShaderIdePreviewVariant::default_forward(),
        ShaderIdePreviewVariant::new(ShaderPassType::GBuffer, 1),
        ShaderIdePreviewVariant::new(ShaderPassType::DepthPrepass, 0),
        ShaderIdePreviewVariant::new(ShaderPassType::Shadow, 0),
        ShaderIdePreviewVariant::new(ShaderPassType::Velocity, 0),
        ShaderIdePreviewVariant::new(ShaderPassType::TaaReactiveMask, 1),
    ];
    let report = write_shader_ide_env_for_project(&project, Some(&out_dir), &variants).unwrap();

    assert_eq!(report.shader_count, 2);
    assert_eq!(report.preview_count, 12);
    assert_eq!(report.naga_validated_preview_count, report.preview_count);
    for shader_uri in [
        AssetUri::parse("res://shaders/hero").unwrap(),
        AssetUri::parse("res://shaders/rival").unwrap(),
    ] {
        for variant in &variants {
            let preview_path = shader_ide_preview_relative_path(&shader_uri, &variant.name);
            let segment_path =
                shader_ide_preview_segments_relative_path(&shader_uri, &variant.name);
            assert!(
                out_dir.join(&preview_path).exists(),
                "missing preview {}",
                preview_path.display()
            );
            let segment_map: ShaderIdePreviewMap =
                serde_json::from_str(&fs::read_to_string(out_dir.join(segment_path)).unwrap())
                    .unwrap();
            assert_eq!(segment_map.shader_uri, shader_uri);
            assert_eq!(segment_map.variant, variant.name);
        }
    }
    let taa_preview = fs::read_to_string(out_dir.join(shader_ide_preview_relative_path(
        &AssetUri::parse("res://shaders/rival").unwrap(),
        &variants[5].name,
    )))
    .unwrap();
    assert!(taa_preview.contains("// include: zr_template_taa_reactive_mask.wgsl"));
    assert!(taa_preview.contains("const ZR_OPT_ENABLE_RIM: bool = true;"));

    let reduced_variants = [ShaderIdePreviewVariant::default_forward()];
    let reduced_report =
        write_shader_ide_env_for_project(&project, Some(&out_dir), &reduced_variants).unwrap();

    assert_eq!(reduced_report.preview_count, 2);
    assert_eq!(reduced_report.naga_validated_preview_count, 2);
    assert_eq!(
        reduced_report.removed_stale_file_count,
        (variants.len() - reduced_variants.len()) * 2 * 2
    );
    assert!(!out_dir
        .join(shader_ide_preview_relative_path(
            &AssetUri::parse("res://shaders/hero").unwrap(),
            &variants[1].name
        ))
        .exists());
    assert!(out_dir
        .join(shader_ide_preview_relative_path(
            &AssetUri::parse("res://shaders/rival").unwrap(),
            SHADER_IDE_PREVIEW_DEFAULT_VARIANT
        ))
        .exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_ide_env_rejects_duplicate_preview_variant_names() {
    let variants = [
        ShaderIdePreviewVariant::default_forward(),
        ShaderIdePreviewVariant::default_forward(),
    ];
    let error = validate_shader_ide_preview_variants(&variants).expect_err("duplicate variant");

    assert!(error.contains("duplicate shader IDE preview variant default"));
}

#[test]
fn shader_ide_env_rewrites_only_changed_module_stub_and_map() {
    let root = unique_temp_project_root("shader_ide_env_incremental");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Shader Ide Sandbox",
        AssetUri::parse("res://shaders/hero").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_include_shader_package(&paths);
    write_surface_shader_package(&paths);

    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();

    let out_dir = root.join("ide-out");
    let first_report = write_shader_ide_env_for_project(&project, Some(&out_dir), &[]).unwrap();
    assert_eq!(
        first_report.managed_file_count,
        first_report.written_file_count
    );

    let shared_stub = out_dir.join("modules/shader_ide_sandbox/shared.wgsl");
    let hero_stub = out_dir.join("modules/shader_ide_sandbox/hero.wgsl");
    let generated_stub = out_dir.join("generated/res_shaders_hero.material.wgsl");
    let module_map = out_dir.join("module_map.json");
    let before_shared = fs::read_to_string(&shared_stub).unwrap();
    let before_hero = fs::read_to_string(&hero_stub).unwrap();
    let before_generated = fs::read_to_string(&generated_stub).unwrap();
    let before_module_map = fs::read_to_string(&module_map).unwrap();
    let before_hero_modified = modified_time(&hero_stub);
    let before_generated_modified = modified_time(&generated_stub);

    std::thread::sleep(Duration::from_millis(1100));
    fs::write(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("shared")
            .join("shared.wgsl"),
        r#"
fn shared_tint() -> vec4f {
return vec4f(0.9, 0.2, 0.1, 1.0);
}
"#,
    )
    .unwrap();

    project.scan_and_import().unwrap();
    let second_report = write_shader_ide_env_for_project(&project, Some(&out_dir), &[]).unwrap();

    assert_eq!(second_report.written_file_count, 2);
    assert_eq!(second_report.removed_stale_file_count, 0);
    assert_eq!(
        second_report.naga_parsed_stub_count,
        second_report.module_count
    );
    assert_eq!(second_report.naga_validated_preview_count, 0);
    assert_ne!(before_shared, fs::read_to_string(&shared_stub).unwrap());
    assert_ne!(before_module_map, fs::read_to_string(&module_map).unwrap());
    assert_eq!(before_hero, fs::read_to_string(&hero_stub).unwrap());
    assert_eq!(
        before_generated,
        fs::read_to_string(&generated_stub).unwrap()
    );
    assert_eq!(before_hero_modified, modified_time(&hero_stub));
    assert_eq!(before_generated_modified, modified_time(&generated_stub));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_ide_env_report_serializes_without_nested_objects() {
    let report = ShaderIdeEnvReport {
        output_dir: "out".to_string(),
        module_map: "out/module_map.json".to_string(),
        shader_count: 1,
        module_count: 2,
        generated_material_count: 1,
        preview_count: 0,
        naga_parsed_stub_count: 2,
        naga_validated_preview_count: 0,
        managed_file_count: 3,
        written_file_count: 3,
        removed_stale_file_count: 0,
    };

    let value = serde_json::to_value(report).unwrap();

    assert_eq!(value["shader_count"], Value::from(1));
    assert_eq!(value["naga_parsed_stub_count"], Value::from(2));
    assert_eq!(value["naga_validated_preview_count"], Value::from(0));
    assert_eq!(value["managed_file_count"], Value::from(3));
    assert_eq!(value["written_file_count"], Value::from(3));
}

#[test]
fn shader_ide_env_rejects_invalid_stub_wgsl_with_module_context() {
    let stub = ShaderIdeStub {
        relative_path: PathBuf::from("modules/project/broken.wgsl"),
        source: "fn broken( {".to_string(),
        include_paths: Vec::new(),
        validation_defines: Vec::new(),
        entry: ShaderIdeModuleMapEntry {
            import_path: "project::broken".to_string(),
            scope_uri: None,
            kind: ShaderAssetKind::Include,
            stub_path: "modules/project/broken.wgsl".to_string(),
            source_uri: None,
            source_files: Vec::new(),
            content_hash: "invalid".to_string(),
            generated: false,
        },
    };

    let error = parse_shader_ide_stubs(&[stub]).expect_err("invalid stub should fail");

    assert!(error.contains("project::broken"), "{error}");
    assert!(error.contains("modules/project/broken.wgsl"), "{error}");
}

#[test]
fn shader_ide_env_parses_stub_with_builtin_and_generated_context() {
    let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
    let mut stubs = builtin_stubs();
    stubs.push(ShaderIdeStub {
        relative_path: PathBuf::from("generated/res_shaders_hero.material.wgsl"),
        source: r#"
fn zr_mat_base_color() -> vec4<f32> {
    return vec4<f32>(0.8, 0.4, 0.2, 1.0);
}
"#
        .to_string(),
        include_paths: Vec::new(),
        validation_defines: Vec::new(),
        entry: ShaderIdeModuleMapEntry {
            import_path: GENERATED_MATERIAL_MODULE_IMPORT_PATH.to_string(),
            scope_uri: Some(shader_uri.clone()),
            kind: ShaderAssetKind::Surface,
            stub_path: "generated/res_shaders_hero.material.wgsl".to_string(),
            source_uri: Some(shader_uri.clone()),
            source_files: Vec::new(),
            content_hash: "generated".to_string(),
            generated: true,
        },
    });
    stubs.push(ShaderIdeStub {
        relative_path: PathBuf::from("modules/shader_ide_sandbox/hero.wgsl"),
        source: r#"
fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
    var surface = zr_surface_default(input);
    surface.base_color = zr_mat_base_color();
    return surface;
}
"#
        .to_string(),
        include_paths: vec![GENERATED_MATERIAL_MODULE_IMPORT_PATH.to_string()],
        validation_defines: Vec::new(),
        entry: ShaderIdeModuleMapEntry {
            import_path: "shader_ide_sandbox::hero".to_string(),
            scope_uri: None,
            kind: ShaderAssetKind::Surface,
            stub_path: "modules/shader_ide_sandbox/hero.wgsl".to_string(),
            source_uri: Some(shader_uri),
            source_files: Vec::new(),
            content_hash: "hero".to_string(),
            generated: false,
        },
    });

    let generated = stubs
        .iter()
        .find(|stub| stub.entry.generated)
        .expect("generated material stub should exist");
    let generated_source = shader_ide_stub_validation_source(generated, &stubs)
        .expect("generated material should not depend on itself");
    assert!(
        !generated_source.contains("// Zircon shader IDE validation dependency: self::material"),
        "generated material stub must not append itself as a validation dependency"
    );
    assert_eq!(parse_shader_ide_stubs(&stubs).unwrap(), stubs.len());
}

pub(crate) fn write_surface_shader_package(paths: &ProjectPaths) {
    let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("hero.zmeta");
    let mut shader_meta = AssetMetaDocument::new(AssetUuid::new(), shader_uri, AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("hero");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("hero.zshader"),
        r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["hero.wgsl"]

[[imports]]
source = "shader_ide_sandbox::shared"
redirect = { uuid = "22222222-2222-4222-8222-222222222222", url = "res://shaders/shared" }

[[properties]]
name = "base_color"
kind = "vec4"
default = [0.8, 0.4, 0.2, 1.0]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("hero.wgsl"),
        r#"
#include <self::material>

fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
var surface = zr_surface_default(input);
surface.base_color = zr_mat_base_color();
return surface;
}
"#,
    )
    .unwrap();
}

pub(crate) fn write_include_shader_package(paths: &ProjectPaths) {
    let shader_uri = AssetUri::parse("res://shaders/shared").unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("shared.zmeta");
    let mut shader_meta = AssetMetaDocument::new(AssetUuid::new(), shader_uri, AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("shared");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("shared.zshader"),
        r#"
kind = "include"
version = 2
wgsl_files = ["shared.wgsl"]
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("shared.wgsl"),
        r#"
fn shared_tint() -> vec4f {
return vec4f(0.8, 0.4, 0.2, 1.0);
}
"#,
    )
    .unwrap();
}

pub(crate) fn write_option_surface_shader_package(paths: &ProjectPaths) {
    write_named_option_surface_shader_package(
        paths,
        "hero",
        "res://shaders/hero",
        "0.8, 0.4, 0.2, 1.0",
    );
}

fn write_named_option_surface_shader_package(
    paths: &ProjectPaths,
    stem: &str,
    uri: &str,
    base_color: &str,
) {
    let shader_uri = AssetUri::parse(uri).unwrap();
    let shader_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join(format!("{stem}.zmeta"));
    let mut shader_meta = AssetMetaDocument::new(AssetUuid::new(), shader_uri, AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join(stem);
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join(format!("{stem}.zshader")),
        format!(
            r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["{stem}.wgsl"]

[[options]]
name = "ENABLE_RIM"
kind = "bool"
default = false

[[properties]]
name = "base_color"
kind = "vec4"
default = [{base_color}]
"#
        ),
    )
    .unwrap();
    fs::write(
        shader_dir.join(format!("{stem}.wgsl")),
        r#"
#include <self::material>

fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
var surface = zr_surface_default(input);
surface.base_color = zr_mat_base_color();
if ZR_OPT_ENABLE_RIM {
    surface.base_color = vec4f(1.0, 0.2, 0.1, 1.0);
}
return surface;
}
"#,
    )
    .unwrap();
}

fn modified_time(path: &Path) -> std::time::SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

pub(crate) fn unique_temp_project_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
