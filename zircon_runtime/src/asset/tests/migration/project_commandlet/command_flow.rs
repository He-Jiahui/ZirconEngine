use super::*;

#[test]
fn dry_run_apply_and_second_apply_are_safe_and_byte_idempotent() {
    let root = fixture_root("idempotent");
    write_manifest(&root, &["assets", "content"]);
    let shader_guid: AssetUuid = "11111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "content",
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
    let legacy = format!(
        "version = 2\nname = \"Hero\"\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    let material = root.join("assets/materials/hero.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(&material, &legacy).unwrap();
    let cache = root.join("assets/.zircon/cache/do-not-touch.scene.toml");
    fs::create_dir_all(cache.parent().unwrap()).unwrap();
    fs::write(&cache, b"binary-ish cache bytes\0\xff").unwrap();
    let binary = root.join("assets/models/hero.glb");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, b"glTF\0binary").unwrap();

    let dry_run = migrate_project_assets(AssetMigrationOptions::new(
        root.clone(),
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert!(dry_run.succeeded());
    let dry_run_paths = dry_run
        .changed_files()
        .iter()
        .map(|change| change.path().to_path_buf())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(dry_run_paths.len(), 3);
    assert!(dry_run_paths.contains(&material));
    assert!(dry_run_paths.contains(&material.with_file_name("hero.zmaterial.zmeta")));
    assert!(dry_run_paths.contains(&binary.with_file_name("hero.glb.zmeta")));
    assert_eq!(fs::read_to_string(&material).unwrap(), legacy);

    let applied = migrate_project_assets(AssetMigrationOptions::new(
        root.clone(),
        AssetMigrationMode::Apply,
    ))
    .unwrap();
    assert!(applied.succeeded());
    let applied_paths = applied
        .changed_files()
        .iter()
        .map(|change| change.path().to_path_buf())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(applied_paths, dry_run_paths);
    let first_bytes = fs::read(&material).unwrap();
    let migrated: toml::Value = toml::from_str(std::str::from_utf8(&first_bytes).unwrap()).unwrap();
    assert_eq!(
        migrated["shader"]["guid"].as_str().map(str::to_owned),
        Some(shader_guid.to_string())
    );
    assert_eq!(
        migrated["shader"]["path_hint"].as_str(),
        Some("content/shaders/pbr.zshader")
    );
    assert!(migrated["shader"].get("uuid").is_none());
    assert!(migrated["shader"].get("url").is_none());
    let reloaded = ZMaterialDocument::from_project_toml_str(
        std::str::from_utf8(&first_bytes).unwrap(),
        |reference| {
            let reference = reference
                .project_ref()
                .expect("migrated material shader reference should stay project-local");
            Ok::<_, ReferenceResolutionError>(AssetReference::new(
                reference.guid(),
                AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
            ))
        },
    )
    .unwrap();
    assert_eq!(reloaded.shader.uuid, shader_guid);

    let second = migrate_project_assets(AssetMigrationOptions::new(
        root.clone(),
        AssetMigrationMode::Apply,
    ))
    .unwrap();
    assert!(second.succeeded());
    assert!(second.changed_files().is_empty());
    assert_eq!(fs::read(&material).unwrap(), first_bytes);
    assert_eq!(fs::read(&cache).unwrap(), b"binary-ish cache bytes\0\xff");
    assert_eq!(fs::read(&binary).unwrap(), b"glTF\0binary");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scene_material_and_model_are_the_only_first_wave_authoring_document_formats() {
    let root = fixture_root("formats");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "21111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "textures/hero.png",
        guid,
        AssetKind::Texture,
    );
    let legacy_ref = format!("uuid = \"{guid}\"\nurl = \"res://textures/hero.png\"\n");
    let documents = [
        (
            "scenes/main.scene.toml",
            format!(
                "[[entities]]\nentity = 1\nname = \"Hero\"\ntransform = {{ translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }}\n\n[entities.mesh.model]\n{legacy_ref}\n[entities.mesh.material]\n{legacy_ref}"
            ),
        ),
        (
            "materials/hero.zmaterial",
            format!("version = 2\n\n[shader]\n{legacy_ref}"),
        ),
        (
            "models/hero.model.toml",
            format!(
                "uri = \"res://models/hero.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\n{legacy_ref}"
            ),
        ),
    ];
    for (relative, document) in documents {
        let path = root.join("assets").join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, document).unwrap();
    }
    fs::write(
        root.join("assets/notes.toml"),
        format!("reference = {{ uuid = \"{guid}\", url = \"res://textures/hero.png\" }}\n"),
    )
    .unwrap();

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();

    let authoring_changes = report
        .changed_files()
        .iter()
        .map(|change| change.path().to_path_buf())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) != Some("zmeta"))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_authoring_changes = [
        root.join("assets/scenes/main.scene.toml"),
        root.join("assets/materials/hero.zmaterial"),
        root.join("assets/models/hero.model.toml"),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(authoring_changes, expected_authoring_changes);
    assert!(
        fs::read_to_string(root.join("assets/notes.toml"))
            .unwrap()
            .contains("uuid")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_manifest_is_a_typed_commandlet_error() {
    let root = fixture_root("manifest");
    let error = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap_err();

    assert!(error.to_string().contains("zircon-project.toml"));
    fs::remove_dir_all(root).unwrap();
}
