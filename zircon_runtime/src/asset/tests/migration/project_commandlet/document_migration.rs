use super::*;

#[test]
fn migration_document_mutates_one_typed_artifact_without_full_value_clones() {
    const MIGRATION_DOCUMENT_SOURCE: &str = include_str!("../../../migration/document.rs");
    const PROJECT_DOCUMENT_SOURCE: &str = include_str!("../../../assets/project_document.rs");
    const PROJECT_DOCUMENT_CODEC_SOURCE: &str =
        include_str!("../../../assets/project_document/codec.rs");
    const MATERIAL_READER_SOURCE: &str =
        include_str!("../../../assets/project_document/material.rs");
    const MODEL_READER_SOURCE: &str = include_str!("../../../assets/project_document/model.rs");
    const SCENE_READER_SOURCE: &str = include_str!("../../../assets/project_document/scene.rs");

    assert_eq!(
        PROJECT_DOCUMENT_CODEC_SOURCE.matches("from_str").count(),
        1,
        "project document codec must own the one mutable TOML parse per document"
    );
    assert!(PROJECT_DOCUMENT_CODEC_SOURCE.contains("toml::from_str::<toml::Value>"));
    assert!(!PROJECT_DOCUMENT_CODEC_SOURCE.contains("Clone"));
    assert!(!MIGRATION_DOCUMENT_SOURCE.contains("toml::from_str::<toml::Value>"));
    assert!(PROJECT_DOCUMENT_SOURCE.contains("ProjectDocumentArtifact"));
    assert!(MIGRATION_DOCUMENT_SOURCE.contains("artifact.into_project_document()"));
    assert!(!MIGRATION_DOCUMENT_SOURCE.contains("from_project_toml_str"));
    for formal_reader in [
        MATERIAL_READER_SOURCE,
        MODEL_READER_SOURCE,
        SCENE_READER_SOURCE,
    ] {
        assert!(!formal_reader.contains("from_str"));
        assert!(formal_reader.contains("ProjectDocumentArtifact"));
    }
    for retired_full_document_copy in [
        "original.clone()",
        "omit_toml_null_subfields",
        "artifact.clone()",
        "artifact.value().clone()",
        "self.value.clone()",
    ] {
        assert!(
            !MIGRATION_DOCUMENT_SOURCE.contains(retired_full_document_copy),
            "retired full-document copy owner remains: {retired_full_document_copy}"
        );
    }
    assert!(MIGRATION_DOCUMENT_SOURCE.contains("struct MigrationDocumentArtifact"));
    assert!(MIGRATION_DOCUMENT_SOURCE.contains("artifact.to_pretty_bytes(path)"));
    assert!(MIGRATION_DOCUMENT_SOURCE.contains("artifact.record_change"));
}

#[test]
fn unsupported_material_version_does_not_mask_reference_resolution_failure() {
    let root = fixture_root("material-version-reference-error-order");
    write_manifest(&root, &["assets"]);
    let material = root.join("assets/materials/unsupported.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    let source = "version = 1\n\n[shader]\nuuid = \"d1111111-2222-4333-8444-555555555555\"\nurl = \"res://shaders/missing.zshader\"\n";
    fs::write(&material, source).unwrap();

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();

    assert!(!report.succeeded());
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::DanglingReference
    );
    assert_eq!(fs::read_to_string(&material).unwrap(), source);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolvable_unsupported_material_version_uses_canonical_validator_without_writing() {
    let root = fixture_root("material-unsupported-version-validator");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "d2111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let material = root.join("assets/materials/unsupported.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    let source = format!(
        "version = 1\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    fs::write(&material, &source).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(!report.succeeded());
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert!(report.issues()[0].message().contains(
        "zmaterial v2 document version `1` is unsupported; migrate material files to version = 2"
    ));
    assert_eq!(fs::read_to_string(&material).unwrap(), source);
    assert!(
        !material
            .with_file_name("unsupported.zmaterial.zmeta")
            .exists()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn formal_schema_failure_after_reference_rewrite_does_not_write_staged_bytes() {
    let root = fixture_root("formal-schema-failure-after-rewrite");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "e1111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "textures/registered.png",
        guid,
        AssetKind::Texture,
    );
    let model = root.join("assets/models/formal-invalid.model.toml");
    fs::create_dir_all(model.parent().unwrap()).unwrap();
    let source = format!(
        "uri = \"res://models/formal-invalid.model.toml\"\n\n[unexpected_reference]\nuuid = \"{guid}\"\nurl = \"res://textures/registered.png\"\n"
    );
    fs::write(&model, &source).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(!report.succeeded());
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert!(
        report.issues()[0]
            .message()
            .contains("formal authoring reader rejected document")
    );
    assert_eq!(fs::read_to_string(&model).unwrap(), source);
    assert!(
        !model
            .with_file_name("formal-invalid.model.toml.zmeta")
            .exists()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scene_reference_rewrite_is_formally_decoded_from_the_shared_artifact() {
    let root = fixture_root("scene-shared-document-artifact");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "e2111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(&root, "assets", "models/hero.glb", guid, AssetKind::Model);
    let scene = root.join("assets/scenes/main.scene.toml");
    fs::create_dir_all(scene.parent().unwrap()).unwrap();
    fs::write(
        &scene,
        format!(
            "[[entities]]\nentity = 1\nname = \"Hero\"\ntransform = {{ translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }}\n\n[entities.mesh.model]\nuuid = \"{guid}\"\nurl = \"res://models/hero.glb\"\n[entities.mesh.material]\nuuid = \"{guid}\"\nurl = \"res://models/hero.glb\"\n"
        ),
    )
    .unwrap();

    let first =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(first.succeeded());
    let migrated = fs::read_to_string(&scene).unwrap();
    assert!(migrated.contains("kind = \"project\""));
    let second =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(second.succeeded());
    assert!(second.changed_files().is_empty());
    assert_eq!(fs::read_to_string(&scene).unwrap(), migrated);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_material_version_fails_formal_decode_without_writing_staged_reference_bytes() {
    let root = fixture_root("material-missing-version-formal-failure");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "e3111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let material = root.join("assets/materials/missing-version.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    let source = format!("[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n");
    fs::write(&material, &source).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(!report.succeeded());
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert!(
        report.issues()[0]
            .message()
            .contains("formal authoring reader rejected document")
    );
    assert_eq!(fs::read_to_string(&material).unwrap(), source);
    assert!(
        !material
            .with_file_name("missing-version.zmaterial.zmeta")
            .exists()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_document_preserves_unrepaired_current_reference_fields_and_counts_only_repair() {
    let root = fixture_root("mixed-repaired-and-unchanged-current-references");
    write_manifest(&root, &["assets"]);
    let repaired_guid: AssetUuid = "71111111-2222-4333-8444-555555555555".parse().unwrap();
    let stale_guid: AssetUuid = "81111111-2222-4333-8444-555555555555".parse().unwrap();
    const UNTOUCHED_GUID_TEXT: &str = "AA111111-2222-4333-8444-555555555555";
    let untouched_guid: AssetUuid = UNTOUCHED_GUID_TEXT.parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "textures/repaired.png",
        repaired_guid,
        AssetKind::Texture,
    );
    write_registered_source(
        &root,
        "assets",
        "textures/untouched.png",
        untouched_guid,
        AssetKind::Texture,
    );
    let model = root.join("assets/models/mixed.model.toml");
    fs::create_dir_all(model.parent().unwrap()).unwrap();
    fs::write(
        &model,
        format!(
            "uri = \"res://models/mixed.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nkind = \"project\"\nguid = \"{stale_guid}\"\npath_hint = \"textures/repaired.png\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nkind = \"project\"\nguid = \"{UNTOUCHED_GUID_TEXT}\"\npath_hint = \"textures/untouched.png\"\n"
        ),
    )
    .unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded());
    let document_change = report
        .changed_files()
        .iter()
        .find(|change| change.path() == model)
        .expect("mixed document should be rewritten for the repaired reference");
    assert_eq!(document_change.reference_count(), 1);
    let migrated: toml::Value = toml::from_str(&fs::read_to_string(&model).unwrap()).unwrap();
    assert_eq!(
        migrated["primitives"][1]["mesh"]["guid"].as_str(),
        Some(UNTOUCHED_GUID_TEXT)
    );
    assert!(migrated["primitives"][1]["mesh"].get("sub").is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_current_reference_keeps_canonical_serde_issue_message() {
    let root = fixture_root("malformed-current-reference-message");
    write_manifest(&root, &["assets"]);
    let model = root.join("assets/models/malformed.model.toml");
    fs::create_dir_all(model.parent().unwrap()).unwrap();
    fs::write(
        &model,
        "uri = \"res://models/malformed.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nkind = \"project\"\nguid = \"a1111111-2222-4333-8444-555555555555\"\npath_hint = \"textures/malformed.png\"\nsub = 7\n",
    )
    .unwrap();
    let expected = serde_json::from_value::<
        zircon_runtime_interface::project::PersistedAssetReference,
    >(serde_json::json!({
        "kind": "project",
        "guid": "a1111111-2222-4333-8444-555555555555",
        "path_hint": "textures/malformed.png",
        "sub": 7,
    }))
    .unwrap_err()
    .to_string();

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();

    assert!(!report.succeeded());
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert_eq!(report.issues()[0].message(), expected);
    assert_eq!(
        fs::read_to_string(&model).unwrap(),
        "uri = \"res://models/malformed.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nkind = \"project\"\nguid = \"a1111111-2222-4333-8444-555555555555\"\npath_hint = \"textures/malformed.png\"\nsub = 7\n"
    );
    fs::remove_dir_all(root).unwrap();
}

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
fn retired_project_reference_without_subasset_omits_toml_null_and_is_idempotent() {
    let root = fixture_root("retired-project-reference-without-subasset");
    write_manifest(&root, &["assets"]);
    let shader_guid: AssetUuid = "90111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
    let material = root.join("assets/materials/no-subasset.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        ),
    )
    .unwrap();

    let first =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(first.succeeded(), "migration issues: {:?}", first.issues());
    let migrated_bytes = fs::read(&material).unwrap();
    let migrated_text = std::str::from_utf8(&migrated_bytes).unwrap();
    let migrated: toml::Value = toml::from_str(migrated_text).unwrap();
    assert_eq!(migrated["shader"]["kind"].as_str(), Some("project"));
    assert!(migrated["shader"].get("sub").is_none());
    assert!(!migrated_text.contains("sub ="));

    let second =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(second.succeeded());
    assert!(second.changed_files().is_empty());
    assert_eq!(fs::read(&material).unwrap(), migrated_bytes);
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
    assert!(
        fs::read_to_string(&material)
            .unwrap()
            .contains("kind = \"project\"")
    );

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
    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AssetMigrationIssueKind::DanglingReference)
    );
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
