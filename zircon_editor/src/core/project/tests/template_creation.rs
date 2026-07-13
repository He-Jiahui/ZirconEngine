use std::fs;

use zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION;

use super::super::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use super::temp_root;

#[test]
fn template_creation_copies_pack_rewrites_manifest_and_opens() {
    let location = temp_root("template-copy");
    let draft = NewProjectDraft {
        project_name: "Authority Project".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let opened = ProjectAuthority::default()
        .open_project(&created.root)
        .unwrap();

    assert_eq!(created.summary.name, "Authority Project");
    assert_eq!(opened.summary, created.summary);
    assert_eq!(
        opened.summary.format_version,
        PROJECT_MANIFEST_FORMAT_VERSION
    );
    for relative in [
        "assets/scenes/main.scene.toml",
        "assets/materials/default.zmaterial",
        "assets/models/cube.obj",
        "assets/shaders/pbr_shader/pbr.zshader",
        "assets/shaders/pbr_shader/pbr.wgsl",
        ".gitignore",
    ] {
        assert!(created.root.join(relative).is_file(), "missing {relative}");
    }
    for relative in [
        ".zircon/cache",
        ".zircon/registry",
        ".zircon/autosave",
        ".zircon/play",
        ".zircon/thumbnails",
    ] {
        assert!(created.root.join(relative).is_dir(), "missing {relative}");
    }
    let retired_root = ["lib", "rary"].concat();
    assert!(!created.root.join(retired_root).exists());
    let manifest = fs::read_to_string(created.root.join("zircon-project.toml")).unwrap();
    assert!(manifest.contains("name = \"Authority Project\""));
    assert!(manifest.contains("asset_roots = [\"assets\"]"));

    fs::remove_dir_all(location).unwrap();
}

#[test]
fn non_empty_target_is_rejected_without_modifying_existing_content() {
    let location = temp_root("non-empty");
    let target = location.join("Existing");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("keep.txt"), "keep").unwrap();
    let draft = NewProjectDraft {
        project_name: "Existing".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let error = ProjectAuthority::default()
        .create_project(&draft)
        .unwrap_err();

    assert!(matches!(
        error,
        super::super::ProjectAuthorityError::TargetNotEmpty { .. }
    ));
    assert_eq!(fs::read_to_string(target.join("keep.txt")).unwrap(), "keep");
    assert_eq!(staging_entries(&location), Vec::<String>::new());
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn draft_rejects_unsafe_project_name_components_before_touching_disk() {
    let location = temp_root("unsafe-name");
    for project_name in ["..", "folder/Game", r"folder\Game", "CON", "Game.", "Game "] {
        let draft = NewProjectDraft {
            project_name: project_name.to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        };
        assert!(ProjectAuthority::default().create_project(&draft).is_err());
    }
    assert_eq!(fs::read_dir(&location).unwrap().count(), 0);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn read_only_probe_parses_manifest_and_does_not_create_derived_layout() {
    let root = temp_root("probe-invalid");
    fs::write(root.join("zircon-project.toml"), "name = [\"invalid\"]").unwrap();

    assert!(ProjectAuthority::default().probe_project(&root).is_err());
    assert!(!root.join(".zircon").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn conflicting_rendered_entry_rolls_back_staging_and_leaves_no_project() {
    use zircon_runtime_interface::project::{
        ProjectManifestSummary, ProjectTemplateId, RelPath, RenderedProjectTemplate,
        RenderedProjectTemplateEntry,
    };

    let location = temp_root("rollback");
    let target = location.join("Broken");
    let rendered = RenderedProjectTemplate {
        id: ProjectTemplateId::RenderableEmpty,
        summary: ProjectManifestSummary {
            name: "Broken".to_string(),
            engine_version_req: None,
            default_scene: "res://scenes/main.scene.toml".to_string(),
            format_version: PROJECT_MANIFEST_FORMAT_VERSION,
        },
        entries: vec![
            RenderedProjectTemplateEntry {
                path: RelPath::parse("assets").unwrap(),
                bytes: b"file blocks directory".to_vec(),
            },
            RenderedProjectTemplateEntry {
                path: RelPath::parse("assets/scenes/main.scene.toml").unwrap(),
                bytes: Vec::new(),
            },
        ],
    };

    assert!(ProjectAuthority::default()
        .create_rendered_project(&target, rendered)
        .is_err());
    assert!(!target.exists());
    assert_eq!(staging_entries(&location), Vec::<String>::new());
    fs::remove_dir_all(location).unwrap();
}

fn staging_entries(root: &std::path::Path) -> Vec<String> {
    let mut entries = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains("zircon-staging") || name.contains("zircon-backup"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}
