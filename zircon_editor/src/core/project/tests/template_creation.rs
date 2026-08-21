use std::fs;

use crate::core::settings::{
    settings_registry_with_defaults, SettingsLoad, SettingsScope, SettingsStore,
};
use zircon_runtime::asset::{
    project::{ProjectManager, ProjectPaths},
    AssetReference, AssetRegistryDiagnostic, AssetUri, ReferenceResolutionError, SceneAsset,
    SceneCameraAsset, SceneEntityAsset, SceneMobilityAsset,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportBuildMode, ExportPackagingStrategy, ExportTargetPlatform,
};
use zircon_runtime::core::resource::ResourceState;
use zircon_runtime_interface::math::{view_matrix, Quat, Transform, Vec3};
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
    assert_eq!(opened.manifest().summary(), created.summary);
    assert_eq!(
        opened.manifest().summary().format_version,
        PROJECT_MANIFEST_FORMAT_VERSION
    );
    let export_profile = opened
        .manifest()
        .export_profiles
        .iter()
        .find(|profile| profile.name == "desktop_windows")
        .expect("RenderableEmpty manifest must retain the Windows client export profile");
    assert_eq!(export_profile.target_mode, RuntimeTargetMode::ClientRuntime);
    assert_eq!(
        export_profile.target_platform,
        ExportTargetPlatform::Windows
    );
    assert_eq!(export_profile.build_mode, ExportBuildMode::Release);
    assert_eq!(
        export_profile.strategies,
        vec![
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ]
    );
    assert_eq!(opened.paths().root(), created.root.as_path());
    for relative in [
        "assets/scenes/main.scene.toml",
        "assets/materials/default.zmaterial",
        "assets/models/cube.obj",
        "assets/shaders/pbr_shader.zmeta",
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
    let sun_direction_length = sun_light
        .direction
        .iter()
        .map(|component| *component * *component)
        .sum::<f32>()
        .sqrt();
    assert!(
        (sun_direction_length - 1.0).abs() <= 0.0001,
        "RenderableEmpty Sun direction must already be normalized before runtime extraction, got length {sun_direction_length}"
    );
    let default_sun_direction = [-0.4_f32, -1.0, -0.25];
    let default_sun_direction_length = default_sun_direction
        .iter()
        .map(|component| *component * *component)
        .sum::<f32>()
        .sqrt();
    for (actual_component, default_component) in
        sun_light.direction.iter().zip(default_sun_direction)
    {
        let expected_component = default_component / default_sun_direction_length;
        assert!(
            (*actual_component - expected_component).abs() <= 0.000001,
            "RenderableEmpty Sun direction must match the runtime default orientation; expected {expected_component}, got {actual_component}"
        );
    }
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
    assert_template_point_is_visible_at_mvp_runtime_aspect_ratio(
        camera,
        projection,
        cube.transform.translation,
        "initial Cube",
    );
    let mut f4_authored_cube_translation = cube.transform.translation;
    f4_authored_cube_translation[0] = 42.0;
    assert_template_point_is_visible_at_mvp_runtime_aspect_ratio(
        camera,
        projection,
        f4_authored_cube_translation,
        "F4-authored Cube",
    );

    drop(created);
    fs::remove_dir_all(location).unwrap();
}

fn assert_template_point_is_visible_at_mvp_runtime_aspect_ratio(
    camera: &SceneEntityAsset,
    projection: &SceneCameraAsset,
    point: [f32; 3],
    label: &str,
) {
    const MVP_RUNTIME_ASPECT_RATIO: f32 = 16.0 / 9.0;

    assert!(
        projection.fov_y_radians > 0.0 && projection.fov_y_radians < std::f32::consts::PI,
        "RenderableEmpty Camera must have a finite perspective field of view"
    );
    let camera_transform = Transform {
        translation: Vec3::from_array(camera.transform.translation),
        rotation: Quat::from_array(camera.transform.rotation),
        scale: Vec3::from_array(camera.transform.scale),
    };
    let cube_in_view = view_matrix(camera_transform).transform_point3(Vec3::from_array(point));
    let depth = -cube_in_view.z;
    assert!(
        depth > projection.z_near && depth < projection.z_far,
        "{label} must remain between Camera near/far planes, view_space={cube_in_view:?}"
    );

    let vertical_half_extent = depth * (projection.fov_y_radians * 0.5).tan();
    let horizontal_half_extent = vertical_half_extent * MVP_RUNTIME_ASPECT_RATIO;
    assert!(
        cube_in_view.x.abs() < horizontal_half_extent
            && cube_in_view.y.abs() < vertical_half_extent,
        "{label} center must remain inside the MVP runtime view frustum, view_space={cube_in_view:?}"
    );
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
    let root = created.root.clone();
    let mut manager = created.into_project();
    let imported = manager.scan_and_import().unwrap();
    assert!(!imported.is_empty());

    for uri in [
        "res://scenes/main.scene.toml",
        "res://models/cube.obj",
        "res://materials/default.zmaterial",
        "res://shaders/pbr_shader",
    ] {
        let uri = AssetUri::parse(uri).expect("F2 template asset URI");
        let record = manager
            .registry()
            .get_by_locator(&uri)
            .unwrap_or_else(|| panic!("template scan must register asset {uri}"));
        assert_eq!(
            record.state,
            ResourceState::Ready,
            "template asset {uri} must import successfully: {}",
            record.failure_reason().unwrap_or("no import diagnostic")
        );
        assert!(
            record.artifact_locator().is_some(),
            "template asset {uri} must retain an artifact locator"
        );
    }

    let document = fs::read_to_string(root.join("assets/scenes/main.scene.toml")).unwrap();
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
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn template_creation_rebuilds_regenerable_asset_state_from_source_after_deletion() {
    let location = temp_root("template-derived-state-rebuild");
    let draft = NewProjectDraft {
        project_name: "Derived State Rebuild".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let project_root = created.root.clone();
    let cache_root = created.project().paths().cache_root().to_path_buf();
    let registry_root = created.project().paths().registry_root().to_path_buf();
    let expected_registry_entries = [
        AssetUri::parse("res://models/cube.obj").unwrap(),
        AssetUri::parse("res://materials/default.zmaterial").unwrap(),
        AssetUri::parse("res://shaders/pbr_shader").unwrap(),
    ]
    .into_iter()
    .map(|locator| {
        let uuid = created
            .project()
            .asset_registry()
            .entry_by_path(&locator)
            .expect("the created template registry must contain every required source asset")
            .uuid();
        (locator, uuid)
    })
    .collect::<Vec<_>>();
    drop(created);

    fs::remove_dir_all(&cache_root).unwrap();
    fs::remove_dir_all(&registry_root).unwrap();
    assert!(!cache_root.exists());
    assert!(!registry_root.exists());

    let mut reopened = ProjectManager::open(&project_root).unwrap();
    assert!(cache_root.is_dir(), "opening must restore the cache layout");
    assert!(
        registry_root.join("asset-registry.json").is_file(),
        "opening must rebuild and persist the asset registry from source metadata"
    );

    for (locator, expected_uuid) in expected_registry_entries {
        let rebuilt = reopened
            .asset_registry()
            .entry_by_path(&locator)
            .expect("source asset must return to the rebuilt registry");
        assert_eq!(
            rebuilt.uuid(),
            expected_uuid,
            "source asset {locator} changed logical identity after regenerable state deletion"
        );
    }

    let imported = reopened.scan_and_import().unwrap();
    assert!(
        !imported.is_empty(),
        "a project with deleted regenerable state must import its source assets"
    );

    drop(reopened);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn template_creation_recovers_a_corrupt_persisted_registry_from_source_metadata() {
    let location = temp_root("template-corrupt-registry-recovery");
    let draft = NewProjectDraft {
        project_name: "Corrupt Registry Recovery".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let project_root = created.root.clone();
    let registry_path = created
        .project()
        .paths()
        .registry_root()
        .join("asset-registry.json");
    let expected_cube_uuid = created
        .project()
        .asset_registry()
        .entry_by_path(&AssetUri::parse("res://models/cube.obj").unwrap())
        .expect("the created template registry must contain the Cube model")
        .uuid();
    drop(created);

    fs::write(&registry_path, b"not-json").unwrap();
    let reopened = ProjectManager::open(&project_root).unwrap();

    let rebuilt_registry_bytes = fs::read(&registry_path).unwrap();
    assert_ne!(rebuilt_registry_bytes.as_slice(), b"not-json");
    assert_eq!(
        reopened
            .asset_registry()
            .entry_by_path(&AssetUri::parse("res://models/cube.obj").unwrap())
            .expect("the rebuilt registry must contain the Cube model")
            .uuid(),
        expected_cube_uuid
    );
    assert!(reopened
        .asset_registry()
        .diagnostics()
        .iter()
        .any(|diagnostic| {
            matches!(
                diagnostic,
                AssetRegistryDiagnostic::CorruptPersistenceRebuilt { path, .. }
                    if path == &registry_path
            )
        }));

    drop(reopened);
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
    let expected_root = ProjectPaths::resolve_existing_path(location.join(project_name)).unwrap();

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
        ProjectPaths::resolve_existing_path(location.join("Path Safe Project")).unwrap()
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
