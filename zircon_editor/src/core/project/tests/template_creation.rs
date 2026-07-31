use std::fs;

use crate::core::settings::{
    settings_registry_with_defaults, SettingsLoad, SettingsScope, SettingsStore,
};
use zircon_runtime::asset::{
    project::ProjectManager, AssetReference, AssetUri, ReferenceResolutionError, SceneAsset,
    SceneMobilityAsset,
};
use zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION;
use zircon_runtime_interface::resource::ResourceScheme;

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
    let opened = created.project();

    assert_eq!(created.summary.name, "Authority Project");
    assert_eq!(opened.manifest().summary(), &created.summary);
    assert_eq!(
        opened.manifest().summary().format_version,
        PROJECT_MANIFEST_FORMAT_VERSION
    );
    assert_eq!(opened.paths().root(), created.root.as_path());
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
fn renderable_empty_template_has_the_f2_camera_cube_and_sun_contract() {
    let location = temp_root("f2-template-scene-contract");
    let draft = NewProjectDraft {
        project_name: "F2 Template Contract".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let document = fs::read_to_string(created.root.join("assets/scenes/main.scene.toml")).unwrap();
    let scene = SceneAsset::from_project_toml_str(&document, resolve_template_scene_reference)
        .expect("RenderableEmpty main scene must satisfy the persisted scene schema");
    assert_eq!(
        scene.entities.len(),
        3,
        "the F2 canonical scene must contain exactly Camera, Sun, and Cube"
    );

    let camera = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Camera")
        .expect("RenderableEmpty scene must contain Camera");
    assert_eq!(
        scene
            .entities
            .iter()
            .filter(|entity| entity.name == "Camera")
            .count(),
        1,
        "RenderableEmpty scene must contain exactly one Camera"
    );
    assert!(camera.active);
    let projection = camera
        .camera
        .as_ref()
        .expect("Camera must have a projection");
    assert!(projection.z_near > 0.0);
    assert!(projection.z_far > projection.z_near);
    assert!(camera.transform.scale.iter().all(|value| *value > 0.0));
    assert_eq!(camera.render_layer_mask, 0x0000_0001);
    assert!(camera.mesh.is_none());
    assert!(camera.directional_light.is_none());

    let sun = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Sun")
        .expect("RenderableEmpty scene must contain Sun");
    assert_eq!(
        scene
            .entities
            .iter()
            .filter(|entity| entity.name == "Sun")
            .count(),
        1,
        "RenderableEmpty scene must contain exactly one Sun"
    );
    assert!(sun.active);
    let sun_light = sun
        .directional_light
        .as_ref()
        .expect("Sun must have a directional light");
    assert!(sun_light.intensity > 0.0);
    assert!(sun_light
        .direction
        .iter()
        .any(|component| *component != 0.0));
    assert_eq!(sun.render_layer_mask, 0x0000_0001);
    assert!(sun.camera.is_none());
    assert!(sun.mesh.is_none());

    let cube = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Cube")
        .expect("RenderableEmpty scene must contain Cube");
    assert_eq!(
        scene
            .entities
            .iter()
            .filter(|entity| entity.name == "Cube")
            .count(),
        1,
        "RenderableEmpty scene must contain exactly one Cube"
    );
    assert!(cube.active);
    assert_eq!(cube.mobility, SceneMobilityAsset::Static);
    assert!(cube.transform.scale.iter().all(|value| *value > 0.0));
    assert_eq!(cube.render_layer_mask, 0x0000_0001);
    assert!(cube.camera.is_none());
    assert!(cube.directional_light.is_none());
    let mesh = cube.mesh.as_ref().expect("Cube must have a persisted mesh");
    assert_eq!(mesh.model.locator.to_string(), "res://models/cube.obj");
    assert_eq!(
        mesh.material.locator.to_string(),
        "res://materials/default.zmaterial"
    );

    drop(created);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn renderable_empty_template_scene_refs_match_the_project_registry_after_scan() {
    let location = temp_root("f2-template-registry-contract");
    let draft = NewProjectDraft {
        project_name: "F2 Template Registry Contract".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let mut manager = ProjectManager::open(&created.root).unwrap();
    let imported = manager.scan_and_import().unwrap();
    assert!(!imported.is_empty());

    let document = fs::read_to_string(created.root.join("assets/scenes/main.scene.toml")).unwrap();
    let scene = SceneAsset::from_project_toml_str(&document, |persisted_reference| {
        let runtime_reference = resolve_template_scene_reference(persisted_reference)?;
        let registry_entry = manager
            .asset_registry()
            .entry_by_path(&runtime_reference.locator)
            .ok_or_else(|| ReferenceResolutionError::MissingPath {
                path: runtime_reference.locator.to_string(),
            })?;
        if registry_entry.uuid() != runtime_reference.uuid {
            return Err(ReferenceResolutionError::Registry {
                message: format!(
                    "scene reference {} resolves to registry guid {} instead of {}",
                    runtime_reference.locator,
                    registry_entry.uuid(),
                    runtime_reference.uuid
                ),
            });
        }
        Ok(runtime_reference)
    })
    .expect("RenderableEmpty scene refs must resolve to the scanned project registry");

    let cube = scene
        .entities
        .iter()
        .find(|entity| entity.name == "Cube")
        .expect("RenderableEmpty scene must contain Cube");
    let mesh = cube.mesh.as_ref().expect("Cube must have a persisted mesh");
    for reference in [&mesh.model, &mesh.material] {
        let entry = manager
            .asset_registry()
            .entry_by_path(&reference.locator)
            .expect("parsed scene reference must have a registry entry");
        assert_eq!(entry.uuid(), reference.uuid);
    }

    drop(manager);
    drop(created);
    fs::remove_dir_all(location).unwrap();
}

fn resolve_template_scene_reference(
    reference: &zircon_runtime_interface::project::PersistedAssetReference,
) -> Result<AssetReference, ReferenceResolutionError> {
    let reference = reference
        .project_ref()
        .expect("RenderableEmpty scene references must be project assets");
    let relative = reference
        .path_hint()
        .as_str()
        .strip_prefix("assets/")
        .expect("RenderableEmpty project asset reference must use the assets root");
    let locator = AssetUri::new(
        ResourceScheme::Res,
        relative.to_owned(),
        reference.sub().map(str::to_owned),
    )
    .expect("RenderableEmpty project asset locator");
    Ok(AssetReference::new(reference.guid(), locator))
}

#[test]
fn template_creation_returns_the_canonical_published_root() {
    let location = temp_root("canonical-created-root");
    let project_name = "Canonical Root";
    let draft = NewProjectDraft {
        project_name: project_name.to_string(),
        location: location.join(".").to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let expected_root = fs::canonicalize(location.join(project_name)).unwrap();

    assert_eq!(created.root, expected_root);
    assert_eq!(created.project().paths().root(), expected_root.as_path());

    drop(created);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn template_creation_persists_current_project_settings_source() {
    let location = temp_root("template-project-settings");
    let draft = NewProjectDraft {
        project_name: "Project Settings".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let expected_path = created.root.join(".zircon").join("settings.toml");
    let store = SettingsStore::from_roots(location.join("user"), Some(&created.root));
    let mut registry = settings_registry_with_defaults();

    assert!(matches!(
        store.load_into(SettingsScope::Project, &mut registry).unwrap(),
        SettingsLoad::Loaded {
            path,
            schema_version: 1,
            ..
        } if path == expected_path
    ));

    fs::remove_dir_all(location).unwrap();
}

#[test]
fn template_creation_reopens_from_a_space_and_non_ascii_parent_path() {
    let parent = temp_root("unicode-parent");
    let location = parent.join("MVP 项目 根目录");
    fs::create_dir_all(&location).unwrap();
    let draft = NewProjectDraft {
        project_name: "Path Safe Project".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    assert_eq!(
        created.root,
        fs::canonicalize(location.join("Path Safe Project")).unwrap()
    );

    let mut reopened = ProjectManager::open(&created.root).unwrap();
    reopened.scan_and_import().unwrap();
    assert_eq!(reopened.paths().root(), created.root.as_path());
    assert_eq!(
        reopened.manifest().default_scene.to_string(),
        "res://scenes/main.scene.toml",
        "the filesystem parent must not alter the canonical default-scene URI"
    );

    drop(reopened);
    drop(created);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn created_project_owns_one_generation_after_the_disk_manifest_changes() {
    let location = temp_root("opened-generation");
    let draft = NewProjectDraft {
        project_name: "Generation One".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };
    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let opened = created.project().clone();

    let manifest_path = created.root.join("zircon-project.toml");
    let rewritten = fs::read_to_string(&manifest_path)
        .unwrap()
        .replace("name = \"Generation One\"", "name = \"Generation Two\"");
    fs::write(&manifest_path, rewritten).unwrap();

    assert_eq!(opened.manifest().summary().name, "Generation One");
    assert_eq!(opened.manifest().name, "Generation One");
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

#[test]
fn failed_open_after_commit_rolls_back_new_target_and_transaction_artifacts() {
    let location = temp_root("open-rollback");
    let target = location.join("Broken Open");
    let rendered = rendered_template_with_corrupt_asset_metadata("Broken Open");

    let error = ProjectAuthority::default()
        .create_rendered_project(&target, rendered)
        .unwrap_err();

    assert!(matches!(
        error,
        super::super::ProjectAuthorityError::ProjectGeneration { .. }
    ));
    assert!(
        !target.exists(),
        "a failed post-commit project open must not leave a partial project"
    );
    assert_eq!(staging_entries(&location), Vec::<String>::new());
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn failed_open_after_commit_restores_the_original_empty_target() {
    let location = temp_root("open-rollback-empty-target");
    let target = location.join("Broken Open");
    fs::create_dir(&target).unwrap();

    let error = ProjectAuthority::default()
        .create_rendered_project(
            &target,
            rendered_template_with_corrupt_asset_metadata("Broken Open"),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        super::super::ProjectAuthorityError::ProjectGeneration { .. }
    ));
    assert!(
        target.is_dir(),
        "the original empty target must be restored"
    );
    assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    assert_eq!(staging_entries(&location), Vec::<String>::new());
    fs::remove_dir_all(location).unwrap();
}

fn rendered_template_with_corrupt_asset_metadata(
    project_name: &str,
) -> zircon_runtime_interface::project::RenderedProjectTemplate {
    use zircon_runtime_interface::project::{
        render_project_template, RelPath, RenderedProjectTemplateEntry,
    };

    let mut rendered =
        render_project_template(NewProjectTemplate::RenderableEmpty.pack_id(), project_name)
            .unwrap();
    rendered.entries.extend([
        RenderedProjectTemplateEntry {
            path: RelPath::parse("assets/broken.asset").unwrap(),
            bytes: b"source that requires metadata".to_vec(),
        },
        RenderedProjectTemplateEntry {
            path: RelPath::parse("assets/broken.asset.zmeta").unwrap(),
            bytes: b"not valid metadata".to_vec(),
        },
    ]);
    rendered
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
