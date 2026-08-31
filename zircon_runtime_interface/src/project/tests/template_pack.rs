use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::{
    render_project_template, ProjectNameError, ProjectTemplateId, ProjectTemplatePackError,
    PROJECT_MANIFEST_FORMAT_VERSION,
};

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
    assert_eq!(summary.format_version, PROJECT_MANIFEST_FORMAT_VERSION);
    assert_eq!(rendered.summary, summary);
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
    assert_project_reference(
        default_material
            .get("shader")
            .and_then(toml::Value::as_table),
        "00000000-0000-0000-0000-000000000001",
        "assets/shaders/pbr_shader.zmeta",
    );
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
