use super::*;

#[test]
fn missing_sidecars_are_minted_in_the_same_transaction_as_reference_rewrite() {
    let root = fixture_root("mint-missing-sidecars");
    write_manifest(&root, &["assets"]);
    let model_source = root.join("assets/models/hero.glb");
    let document = root.join("assets/models/hero.model.toml");
    fs::create_dir_all(model_source.parent().unwrap()).unwrap();
    fs::write(&model_source, b"recognizable model source").unwrap();
    fs::write(
        &document,
        "uri = \"res://models/hero.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nuuid = \"f1111111-2222-4333-8444-555555555555\"\nurl = \"res://models/hero.glb\"\n",
    )
    .unwrap();

    let dry_run = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert!(dry_run.succeeded());
    assert!(!model_source.with_file_name("hero.glb.zmeta").exists());
    assert!(!document.with_file_name("hero.model.toml.zmeta").exists());

    let applied =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(applied.succeeded());
    let sidecar = fs::read_to_string(model_source.with_file_name("hero.glb.zmeta")).unwrap();
    let meta = crate::asset::project::AssetMetaDocument::from_toml_str(&sidecar).unwrap();
    let migrated = fs::read_to_string(&document).unwrap();
    let value: toml::Value = toml::from_str(&migrated).unwrap();
    assert_eq!(
        value["primitives"][0]["mesh"]["guid"].as_str(),
        Some(meta.uuid.to_string().as_str())
    );
    let sidecar_bytes = fs::read(model_source.with_file_name("hero.glb.zmeta")).unwrap();
    let document_bytes = fs::read(&document).unwrap();

    let second =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(second.succeeded());
    assert!(second.changed_files().is_empty());
    assert_eq!(
        fs::read(model_source.with_file_name("hero.glb.zmeta")).unwrap(),
        sidecar_bytes
    );
    assert_eq!(fs::read(&document).unwrap(), document_bytes);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn labeled_legacy_locator_becomes_asset_ref_sub_and_formal_reader_reloads_it() {
    let root = fixture_root("sub-label");
    write_manifest(&root, &["assets"]);
    let root_guid: AssetUuid = "91111111-2222-4333-8444-555555555555".parse().unwrap();
    let sub_guid: AssetUuid = "a1111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_subasset(
        &root,
        "assets",
        "models/hero.glb",
        root_guid,
        sub_guid,
        "Mesh0",
    );
    let model = root.join("assets/models/hero.model.toml");
    let original = format!(
        "uri = \"res://models/hero.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nuuid = \"{sub_guid}\"\nurl = \"res://models/hero.glb#Mesh0\"\n"
    );
    fs::write(&model, original).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded());
    let migrated = fs::read_to_string(&model).unwrap();
    let parsed: toml::Value = toml::from_str(&migrated).unwrap();
    assert_eq!(
        parsed["primitives"][0]["mesh"]["sub"].as_str(),
        Some("Mesh0")
    );
    let reloaded = crate::asset::ModelAsset::from_project_toml_str(&migrated, |reference| {
        let reference = reference
            .project_ref()
            .expect("migrated model subasset reference should stay project-local");
        assert_eq!(reference.sub(), Some("Mesh0"));
        Ok::<_, ReferenceResolutionError>(AssetReference::new(
            reference.guid(),
            AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
        ))
    })
    .unwrap();
    assert_eq!(
        reloaded.primitives[0]
            .mesh
            .as_ref()
            .and_then(|reference| reference.locator.label()),
        Some("Mesh0")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn flattened_material_texture_slot_migrates_reference_and_preserves_siblings() {
    let root = fixture_root("material-flattened-slot");
    write_manifest(&root, &["assets"]);
    let shader_guid: AssetUuid = "92111111-2222-4333-8444-555555555555".parse().unwrap();
    let texture_guid: AssetUuid = "93111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
    write_registered_source(
        &root,
        "assets",
        "textures/albedo.png",
        texture_guid,
        AssetKind::Texture,
    );
    let material = root.join("assets/materials/slot.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n\n[textures.albedo]\nuuid = \"{texture_guid}\"\nurl = \"res://textures/albedo.png\"\nfallback = \"white\"\nuv_channel = 2\n\n[textures.albedo.transform]\nscale = [2.0, 3.0]\noffset = [0.25, 0.5]\n"
        ),
    )
    .unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    let migrated = fs::read_to_string(&material).unwrap();
    let reloaded = ZMaterialDocument::from_project_toml_str(&migrated, |reference| {
        let project = reference.project_ref().expect("slot stays project-local");
        let uri = if project.guid() == shader_guid {
            "res://shaders/pbr.zshader"
        } else {
            "res://textures/albedo.png"
        };
        Ok::<_, ReferenceResolutionError>(AssetReference::new(
            project.guid(),
            AssetUri::parse(uri).unwrap(),
        ))
    })
    .unwrap();
    let slot = &reloaded.textures["albedo"];
    assert_eq!(slot.fallback.as_deref(), Some("white"));
    assert_eq!(slot.uv_channel, 2);
    assert_eq!(slot.transform.unwrap().scale, [2.0, 3.0]);
    assert_eq!(slot.transform.unwrap().offset, [0.25, 0.5]);
    assert_eq!(slot.reference.as_ref().unwrap().uuid, texture_guid);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retired_meta_toml_and_source_hash_migrate_with_authoring_references_in_one_transaction() {
    let root = fixture_root("retired-sidecar");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "b1111111-2222-4333-8444-555555555555".parse().unwrap();
    let source = root.join("assets/shaders/legacy.zshader");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "shader source").unwrap();
    let retired = root.join("assets/shaders/legacy.zshader.meta.toml");
    fs::write(
        &retired,
        format!(
            "format_version = 6\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy.zshader\"\nasset_kind = \"Shader\"\nsource_hash = \"legacy-digest\"\n"
        ),
    )
    .unwrap();
    let material = root.join("assets/materials/legacy.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy.zshader\"\n"
        ),
    )
    .unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded());
    assert!(!retired.exists());
    let current = root.join("assets/shaders/legacy.zshader.zmeta");
    let current_text = fs::read_to_string(&current).unwrap();
    assert!(current_text.contains("format_version = 7"));
    assert!(current_text.contains("source_digest = \"legacy-digest\""));
    assert!(!current_text.contains("source_hash"));
    assert!(fs::read_to_string(&material)
        .unwrap()
        .contains("kind = \"project\""));

    let second =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(second.succeeded());
    assert!(second.changed_files().is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sidecar_migration_rejects_every_shape_outside_the_explicit_v6_whitelist() {
    let cases = [
        ("missing-version", "source_digest = \"digest\"\n"),
        (
            "non-integer-version",
            "format_version = \"6\"\nsource_hash = \"digest\"\n",
        ),
        (
            "future-version",
            "format_version = 8\nsource_digest = \"digest\"\n",
        ),
        (
            "other-old-version",
            "format_version = 5\nsource_hash = \"digest\"\n",
        ),
        ("v6-missing-hash", "format_version = 6\n"),
        ("v7-missing-digest", "format_version = 7\n"),
    ];
    for (label, suffix) in cases {
        let root = fixture_root(label);
        write_manifest(&root, &["assets"]);
        let source = root.join("assets/data/value.bin");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "source").unwrap();
        let sidecar = source.with_file_name("value.bin.zmeta");
        let document = format!(
            "uuid = \"e1111111-2222-4333-8444-555555555555\"\nurl = \"res://data/value.bin\"\nasset_kind = \"Data\"\n{suffix}"
        );
        fs::write(&sidecar, &document).unwrap();

        let report =
            migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
                .unwrap();
        assert!(!report.succeeded(), "case {label} must fail");
        assert_eq!(
            report.issues()[0].kind(),
            AssetMigrationIssueKind::InvalidDocument
        );
        assert_eq!(fs::read_to_string(sidecar).unwrap(), document);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn retired_meta_toml_rejects_non_v6_versions_without_renaming() {
    let root = fixture_root("retired-sidecar-non-v6");
    write_manifest(&root, &["assets"]);
    let source = root.join("assets/data/value.bin");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "source").unwrap();
    let retired = source.with_file_name("value.bin.meta.toml");
    let document = "format_version = 7\nuuid = \"f1111111-2222-4333-8444-555555555555\"\nurl = \"res://data/value.bin\"\nasset_kind = \"Data\"\nsource_digest = \"digest\"\n";
    fs::write(&retired, document).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(!report.succeeded());
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert_eq!(fs::read_to_string(&retired).unwrap(), document);
    assert!(!source.with_file_name("value.bin.zmeta").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_guid_falls_back_to_path_and_multi_root_conflict_remains_typed() {
    let root = fixture_root("failures");
    write_manifest(&root, &["assets", "content"]);
    let registered: AssetUuid = "31111111-2222-4333-8444-555555555555".parse().unwrap();
    let stale: AssetUuid = "41111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "textures/shared.png",
        registered,
        AssetKind::Texture,
    );
    let model = root.join("assets/models/main.model.toml");
    fs::create_dir_all(model.parent().unwrap()).unwrap();
    let original = format!(
        "uri = \"res://models/main.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nuuid = \"{stale}\"\nurl = \"res://textures/shared.png\"\n"
    );
    fs::write(&model, &original).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    let repaired = fs::read_to_string(&model).unwrap();
    assert!(repaired.contains(&registered.to_string()));
    assert!(!repaired.contains(&stale.to_string()));

    write_registered_source(
        &root,
        "content",
        "textures/shared.png",
        stale,
        AssetKind::Texture,
    );
    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert!(!report.succeeded());
    assert!(report.issues().iter().any(|issue| matches!(
        issue.kind(),
        AssetMigrationIssueKind::RegistryConflict | AssetMigrationIssueKind::AmbiguousPath
    )));
    assert_eq!(fs::read_to_string(&model).unwrap(), repaired);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn guid_path_repair_is_not_applied_when_another_document_is_fully_dangling() {
    let root = fixture_root("missing-path-and-dangling");
    write_manifest(&root, &["assets"]);
    let registered: AssetUuid = "51111111-2222-4333-8444-555555555555".parse().unwrap();
    let stale: AssetUuid = "61111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "textures/registered.png",
        registered,
        AssetKind::Texture,
    );
    let scenes = root.join("assets/models");
    fs::create_dir_all(&scenes).unwrap();
    let missing_path = format!(
        "uri = \"res://models/repair.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nuuid = \"{registered}\"\nurl = \"res://textures/missing.png\"\n"
    );
    let dangling = format!(
        "uri = \"res://models/dangling.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nuuid = \"{stale}\"\nurl = \"res://textures/dangling.png\"\n"
    );
    fs::write(scenes.join("repair.model.toml"), &missing_path).unwrap();
    fs::write(scenes.join("dangling.model.toml"), &dangling).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(!report.succeeded());
    assert!(report
        .issues()
        .iter()
        .any(|issue| issue.kind() == AssetMigrationIssueKind::DanglingReference));
    assert_eq!(
        fs::read_to_string(scenes.join("repair.model.toml")).unwrap(),
        missing_path
    );
    assert_eq!(
        fs::read_to_string(scenes.join("dangling.model.toml")).unwrap(),
        dangling
    );
    fs::remove_dir_all(root).unwrap();
}
