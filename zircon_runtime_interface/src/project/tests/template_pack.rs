//! 模板包测试覆盖 descriptor 防伪、嵌入源完整性、清单身份和默认资源引用契约。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::{
    project_template_descriptor, render_project_template, ProjectNameError,
    ProjectTemplateCapability, ProjectTemplateId, ProjectTemplatePackError,
    PROJECT_MANIFEST_FORMAT_VERSION,
};

// 不同项目名只影响渲染后的清单身份，不能改变模板 qualified ID 或内容摘要。
#[test]
fn render_exposes_a_stable_qualified_template_descriptor() {
    let first = render_project_template(ProjectTemplateId::RenderableEmpty, "First")
        .expect("render first template");
    let second = render_project_template(ProjectTemplateId::RenderableEmpty, "Second")
        .expect("render second template");

    assert_eq!(first.descriptor, second.descriptor);
    assert_eq!(first.descriptor.qualified_id(), "zircon/renderable-empty@1");
    assert_eq!(first.descriptor.version(), 1);
    assert_eq!(first.descriptor.target_requirements().len(), 2);
    let editor = &first.descriptor.target_requirements()[0];
    assert_eq!(
        editor.target(),
        crate::runtime_build_set::ZrRuntimeModuleCompositionTargetV1::EditorHost
    );
    assert_eq!(
        editor.module_profile(),
        crate::runtime_build_set::ZrRuntimeModuleProfileV1::Editor
    );
    assert_eq!(
        editor.required_capabilities(),
        &[
            ProjectTemplateCapability::Scene3d,
            ProjectTemplateCapability::ObjModelImport,
            ProjectTemplateCapability::PbrMaterialImport,
            ProjectTemplateCapability::WgslShaderImport,
            ProjectTemplateCapability::NativeWindow,
        ]
    );
    assert_eq!(
        editor.required_runtime_providers().collect::<Vec<_>>(),
        vec!["rendering", "obj_importer", "shader_wgsl_importer"]
    );
    let client = &first.descriptor.target_requirements()[1];
    assert_eq!(
        client.target(),
        crate::runtime_build_set::ZrRuntimeModuleCompositionTargetV1::ClientRuntime
    );
    assert_eq!(
        client.module_profile(),
        crate::runtime_build_set::ZrRuntimeModuleProfileV1::Client3d
    );
    assert_eq!(first.descriptor.entries().len(), 17);
    assert!(first
        .descriptor
        .entries()
        .iter()
        .all(|entry| entry.byte_len() > 0));
    assert_eq!(
        first.descriptor.engine_version_req(),
        Some(">=0.1.0, <0.2.0")
    );
    assert_eq!(
        first.descriptor.content_digest().to_string(),
        "05e89a48fdf1499bbf94aed42dba19dcf7923ca058294a10a0bc469dd838e21b"
    );
    assert_eq!(
        project_template_descriptor(ProjectTemplateId::RenderableEmpty),
        first.descriptor
    );
    assert_eq!(
        ProjectTemplateId::RenderableEmpty.descriptor(),
        first.descriptor
    );

    let compatible_engine = super::super::ProjectEngineVersion::parse("0.1.4").unwrap();
    assert!(first
        .descriptor
        .assess_engine_compatibility(&compatible_engine)
        .unwrap()
        .is_compatible());
    let older_required_engine = super::super::ProjectEngineVersion::parse("0.2.0").unwrap();
    assert!(!first
        .descriptor
        .assess_engine_compatibility(&older_required_engine)
        .unwrap()
        .is_compatible());
}

#[test]
fn template_descriptor_is_a_payload_free_registry_handle() {
    assert_eq!(
        std::mem::size_of::<super::super::ProjectTemplateDescriptor>(),
        std::mem::size_of::<ProjectTemplateId>()
    );
}

#[test]
fn template_descriptor_round_trips_as_a_persistable_receipt_identity() {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "Receipt")
        .expect("render template for descriptor receipt");
    let encoded = serde_json::to_string(&rendered.descriptor).expect("serialize descriptor");
    let decoded: super::super::ProjectTemplateDescriptor =
        serde_json::from_str(&encoded).expect("deserialize descriptor");

    assert_eq!(decoded, rendered.descriptor);
}

// descriptor 是注册表句柄而非可自由编辑 DTO；伪造字段必须回到 canonical 计算值。
#[test]
fn template_descriptor_rejects_forged_version_digest_and_capabilities() {
    let descriptor = project_template_descriptor(ProjectTemplateId::RenderableEmpty);
    let encoded = serde_json::to_value(descriptor).expect("serialize descriptor");

    for (field, replacement) in [
        ("version", serde_json::json!(2)),
        (
            "content_digest",
            serde_json::json!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        ),
        ("target_requirements", serde_json::json!([])),
        ("entries", serde_json::json!([])),
    ] {
        let mut forged = encoded.clone();
        forged[field] = replacement;
        assert!(
            serde_json::from_value::<super::super::ProjectTemplateDescriptor>(forged).is_err(),
            "accepted forged template descriptor field {field}"
        );
    }
}

#[test]
fn template_descriptor_rejects_unknown_entry_fields() {
    let descriptor = project_template_descriptor(ProjectTemplateId::RenderableEmpty);
    let mut encoded = serde_json::to_value(descriptor).expect("serialize descriptor");
    encoded["entries"][0]["unexpected"] = serde_json::json!(true);

    assert!(
        serde_json::from_value::<super::super::ProjectTemplateDescriptor>(encoded).is_err(),
        "accepted an unknown field inside a template entry descriptor"
    );
}

// 该集合比较把 include_bytes! 生成的包与版本控制模板树保持一一对应。
#[test]
fn embedded_pack_matches_every_versioned_template_file() {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "Pack Audit")
        .expect("render embedded project template");
    let source_root = template_source_root();
    let expected = collect_relative_files(&source_root);
    let actual = rendered
        .entries
        .iter()
        .map(|entry| entry.path.as_str().to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected);
}

// 创建端随后把 receipt 写入同一 manifest；其余插件、导出和资源引用仍来自包原文。
#[test]
fn render_rewrites_only_manifest_identity_and_preserves_current_schema() {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "My Game")
        .expect("render project template");
    let manifest = rendered
        .entries
        .iter()
        .find(|entry| entry.path.as_str() == "zircon-project.toml")
        .expect("manifest entry");
    let manifest = std::str::from_utf8(&manifest.bytes).unwrap();
    let summary = super::super::ProjectManifestSummary::parse_toml_str(manifest)
        .unwrap()
        .value;

    assert_eq!(summary.name, "My Game");
    assert_eq!(
        summary.engine_version_req.as_deref(),
        rendered.descriptor.engine_version_req()
    );
    assert_eq!(summary.format_version, PROJECT_MANIFEST_FORMAT_VERSION);
    assert_eq!(rendered.summary, summary);
    let manifest_value = toml::from_str::<toml::Value>(manifest).unwrap();
    let plugin_selections = manifest_value
        .get("plugins")
        .and_then(|plugins| plugins.get("selections"))
        .and_then(toml::Value::as_array)
        .expect("template manifest must declare its runtime providers");
    assert_eq!(plugin_selections.len(), 3);
    assert!(plugin_selections.iter().all(|selection| {
        selection.get("required").and_then(toml::Value::as_bool) == Some(true)
    }));
    assert!(rendered
        .entries
        .iter()
        .any(|entry| entry.path.as_str() == ".zircon/cache/.gitignore"));
    let preset = rendered
        .entries
        .iter()
        .find(|entry| entry.path.as_str() == "export/desktop_windows.zpreset")
        .expect("default desktop export preset");
    let preset = crate::serialization::load_versioned::<crate::export::ExportPreset>(
        &preset.bytes,
        crate::serialization::Format::Text,
    )
    .unwrap()
    .value;
    assert_eq!(preset.profile_ref, "desktop_windows");
    let shader = rendered
        .entries
        .iter()
        .find(|entry| entry.path.as_str().ends_with("pbr.zshader"))
        .unwrap();
    let shader = std::str::from_utf8(&shader.bytes).unwrap();
    assert!(shader.contains("version = 2"));
    assert!(!shader.contains("entry_points"));
    let wgsl = rendered
        .entries
        .iter()
        .find(|entry| entry.path.as_str().ends_with("pbr.wgsl"))
        .unwrap();
    let wgsl = std::str::from_utf8(&wgsl.bytes).unwrap();
    assert!(wgsl.contains("zr_material_surface"));
    for retired in ["vs_main", "fs_main", "lib://"] {
        assert!(!shader.contains(retired));
        assert!(!wgsl.contains(retired));
    }
}

#[test]
fn render_reuses_portable_project_name_admission_without_trimming() {
    for (name, expected) in [
        (
            "CON",
            ProjectNameError::WindowsReserved {
                value: "CON".to_string(),
            },
        ),
        (
            "Game.",
            ProjectNameError::WindowsTrailingAlias {
                value: "Game.".to_string(),
            },
        ),
        (
            " Game",
            ProjectNameError::SurroundingWhitespace {
                value: " Game".to_string(),
            },
        ),
        (
            "folder/Game",
            ProjectNameError::NotSingleComponent {
                value: "folder/Game".to_string(),
            },
        ),
    ] {
        let error = render_project_template(ProjectTemplateId::RenderableEmpty, name).unwrap_err();
        assert!(matches!(
            error,
            ProjectTemplatePackError::InvalidProjectName { source } if source == expected
        ));
    }
}

#[test]
fn renderable_empty_scene_declares_a_static_cube_with_persisted_project_references() {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "Scene Contract")
        .expect("render project template");
    let scene = rendered
        .entries
        .iter()
        .find(|entry| entry.path.as_str() == "assets/scenes/main.scene.toml")
        .expect("default scene entry");
    let scene = toml::from_str::<toml::Value>(
        std::str::from_utf8(&scene.bytes).expect("default scene must be UTF-8"),
    )
    .expect("default scene must be valid TOML");
    let entities = scene
        .get("entities")
        .and_then(toml::Value::as_array)
        .expect("default scene must contain entities");
    assert_eq!(entities.len(), 3);

    let camera = entity_named(entities, "Camera");
    assert!(camera.contains_key("camera"));
    assert_eq!(
        camera.get("active").and_then(toml::Value::as_bool),
        Some(true)
    );
    let camera_transform = camera
        .get("transform")
        .and_then(toml::Value::as_table)
        .expect("camera transform");
    assert_eq!(
        camera_transform
            .get("translation")
            .and_then(toml::Value::as_array),
        Some(&vec![
            toml::Value::Float(21.0),
            toml::Value::Float(2.0),
            toml::Value::Float(14.5),
        ]),
        "the default camera composition must preserve the runtime framing contract"
    );
    assert_eq!(
        camera_transform
            .get("rotation")
            .and_then(toml::Value::as_array),
        Some(&vec![
            toml::Value::Float(0.0),
            toml::Value::Float(0.0),
            toml::Value::Float(0.0),
            toml::Value::Float(1.0),
        ]),
        "the runtime framing contract uses an unrotated camera and a wide field of view"
    );
    let camera_settings = camera
        .get("camera")
        .and_then(toml::Value::as_table)
        .expect("camera settings");
    assert_eq!(
        camera_settings
            .get("fov_y_radians")
            .and_then(toml::Value::as_float),
        Some(1.7453293)
    );
    let sun = entity_named(entities, "Sun");
    assert!(sun.contains_key("directional_light"));
    assert_eq!(sun.get("active").and_then(toml::Value::as_bool), Some(true));
    assert_eq!(
        sun.get("mobility").and_then(toml::Value::as_str),
        Some("Static")
    );
    let cube = entity_named(entities, "Cube");
    assert_eq!(
        cube.get("active").and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        cube.get("mobility").and_then(toml::Value::as_str),
        Some("Static")
    );
    let transform = cube
        .get("transform")
        .and_then(toml::Value::as_table)
        .expect("cube transform");
    assert_eq!(
        transform.get("translation").and_then(toml::Value::as_array),
        Some(&vec![
            toml::Value::Float(0.0),
            toml::Value::Float(0.0),
            toml::Value::Float(0.0),
        ]),
        "the Cube must remain at the baseline position used by the runtime framing contract"
    );
    assert_eq!(
        transform.get("scale").and_then(toml::Value::as_array),
        Some(&vec![
            toml::Value::Float(1.0),
            toml::Value::Float(1.0),
            toml::Value::Float(1.0),
        ])
    );
    let mesh = cube
        .get("mesh")
        .and_then(toml::Value::as_table)
        .expect("cube mesh");
    assert_project_reference(
        mesh.get("model").and_then(toml::Value::as_table),
        "00000000-0000-0000-0000-000000000002",
        "assets/models/cube.obj",
    );
    assert_project_reference(
        mesh.get("material").and_then(toml::Value::as_table),
        "00000000-0000-0000-0000-000000000003",
        "assets/materials/default.zmaterial",
    );
}

#[test]
fn renderable_empty_asset_metadata_matches_its_persisted_references() {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "Asset Contract")
        .expect("render project template");
    let cube = template_toml(&rendered, "assets/models/cube.obj.zmeta");
    assert_eq!(
        cube.get("uuid").and_then(toml::Value::as_str),
        Some("00000000-0000-0000-0000-000000000002")
    );
    assert_eq!(
        cube.get("url").and_then(toml::Value::as_str),
        Some("res://models/cube.obj")
    );

    let material = template_toml(&rendered, "assets/materials/default.zmaterial.zmeta");
    assert_eq!(
        material.get("uuid").and_then(toml::Value::as_str),
        Some("00000000-0000-0000-0000-000000000003")
    );
    assert_eq!(
        material.get("url").and_then(toml::Value::as_str),
        Some("res://materials/default.zmaterial")
    );

    let shader = template_toml(&rendered, "assets/shaders/pbr_shader.zmeta");
    assert_eq!(
        shader.get("uuid").and_then(toml::Value::as_str),
        Some("00000000-0000-0000-0000-000000000001")
    );
    assert_eq!(
        shader.get("url").and_then(toml::Value::as_str),
        Some("res://shaders/pbr_shader")
    );
    let default_material = template_toml(&rendered, "assets/materials/default.zmaterial");
    let default_shader = default_material
        .get("shader")
        .and_then(toml::Value::as_table)
        .expect("default material shader reference");
    assert_eq!(
        default_shader.get("kind").and_then(toml::Value::as_str),
        Some("builtin")
    );
    assert_eq!(
        default_shader.get("locator").and_then(toml::Value::as_str),
        Some("builtin://shader/pbr.wgsl")
    );
    assert_eq!(default_shader.len(), 2);
}

fn template_toml(rendered: &super::super::RenderedProjectTemplate, path: &str) -> toml::Value {
    let entry = rendered
        .entries
        .iter()
        .find(|entry| entry.path.as_str() == path)
        .unwrap_or_else(|| panic!("template is missing {path}"));
    toml::from_str(std::str::from_utf8(&entry.bytes).expect("template entry must be UTF-8"))
        .unwrap_or_else(|error| panic!("template entry {path} must be valid TOML: {error}"))
}

fn entity_named<'a>(entities: &'a [toml::Value], name: &str) -> &'a toml::Table {
    entities
        .iter()
        .filter_map(toml::Value::as_table)
        .find(|entity| entity.get("name").and_then(toml::Value::as_str) == Some(name))
        .unwrap_or_else(|| panic!("default scene is missing {name}"))
}

fn assert_project_reference(reference: Option<&toml::Table>, guid: &str, path_hint: &str) {
    let reference = reference.expect("cube asset reference");
    assert_eq!(
        reference.get("kind").and_then(toml::Value::as_str),
        Some("project")
    );
    assert_eq!(
        reference.get("guid").and_then(toml::Value::as_str),
        Some(guid)
    );
    assert_eq!(
        reference.get("path_hint").and_then(toml::Value::as_str),
        Some(path_hint)
    );
}

#[test]
fn template_source_tree_contains_no_links_or_reparse_points() {
    let root = template_source_root();
    let mut pending = vec![root.clone()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            assert!(
                !metadata.file_type().is_symlink(),
                "link in template: {}",
                path.display()
            );
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
                assert_eq!(metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT, 0);
            }
            if metadata.is_dir() {
                pending.push(path);
            }
        }
    }
}

fn template_source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("templates")
        .join("projects")
        .join("renderable-empty")
}

fn collect_relative_files(root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    files
}
