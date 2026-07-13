use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::{render_project_template, ProjectTemplateId, PROJECT_MANIFEST_FORMAT_VERSION};

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
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "My \"Game\"")
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

    assert_eq!(summary.name, "My \"Game\"");
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
