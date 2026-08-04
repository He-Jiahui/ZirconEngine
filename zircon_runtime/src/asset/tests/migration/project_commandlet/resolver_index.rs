use super::*;
use std::path::PathBuf;
use zircon_runtime_interface::project::RelPath;

use crate::asset::migration::{
    MigrationCompoundBinding, MigrationResolverIndex, MigrationSourceProjection,
};

#[test]
fn migration_resolver_is_filesystem_free_after_generation_build() {
    const RESOLVER_SOURCE: &str = include_str!("../../../migration/resolver.rs");
    const INDEX_SOURCE: &str = include_str!("../../../migration/resolver_index.rs");

    assert!(!RESOLVER_SOURCE.contains("std::path::PathBuf"));
    assert!(!RESOLVER_SOURCE.contains("roots:"));
    assert!(RESOLVER_SOURCE.contains("resolve_project_reference_from_lookup"));
    for source in [RESOLVER_SOURCE, INDEX_SOURCE] {
        for forbidden in [
            "std::fs",
            "fs::",
            "File::open(",
            "read_to_string(",
            "read_dir(",
            "symlink_metadata(",
            "canonicalize(",
            "metadata(",
            "AssetMetaDocument",
            "persisted_source_path_for_locator",
            "logical_locator_for_persisted_source",
            "FilesystemProjectSourceLookup",
        ] {
            assert!(
                !source.contains(forbidden),
                "generation-owned resolver source must not reintroduce {forbidden}"
            );
        }
    }
}

#[test]
fn resolver_index_keeps_one_generation_and_order_for_reference_and_root_scale_matrix() {
    const MAX_REFERENCES: usize = 100_000;

    for root_count in [1, 4] {
        let expected = (0..MAX_REFERENCES)
            .map(|reference_index| {
                let root_index = reference_index % root_count;
                let logical_root = format!("root-{root_index}");
                let relative = format!("scale/reference-{reference_index:06}.asset");
                let locator = AssetUri::parse(&format!("res://{relative}")).unwrap();
                let hint = RelPath::parse(format!("{logical_root}/{relative}")).unwrap();
                let projection = MigrationSourceProjection::new(
                    RelPath::parse(&logical_root).unwrap(),
                    PathBuf::from(format!("C:/project/{logical_root}")),
                    RelPath::parse(&relative).unwrap(),
                    PathBuf::from(format!("C:/project/{logical_root}/{relative}")),
                );
                (locator, hint, projection)
            })
            .collect::<Vec<_>>();
        let index = MigrationResolverIndex::build(
            expected.iter().map(|(_, _, projection)| projection.clone()),
            [],
        )
        .unwrap();

        for reference_count in [1, 1_000, MAX_REFERENCES] {
            let forward = expected
                .iter()
                .take(reference_count)
                .map(|(locator, _, _)| index.project_hint_for_locator(locator).unwrap())
                .collect::<Vec<_>>();
            let reverse = expected
                .iter()
                .take(reference_count)
                .map(|(_, hint, _)| index.locator_for_project_hint(hint).unwrap().unwrap())
                .collect::<Vec<_>>();

            assert_eq!(
                forward,
                expected
                    .iter()
                    .take(reference_count)
                    .map(|(_, hint, _)| hint.clone())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                reverse,
                expected
                    .iter()
                    .take(reference_count)
                    .map(|(locator, _, _)| locator.clone())
                    .collect::<Vec<_>>()
            );
            assert_eq!(forward.len() + reverse.len(), reference_count * 2);
        }
    }
}

#[test]
fn resolver_index_merges_physical_aliases_but_keeps_distinct_logical_roots_ambiguous() {
    let locator = AssetUri::parse("res://textures/shared.png").unwrap();
    let assets_projection = MigrationSourceProjection::new(
        RelPath::parse("assets").unwrap(),
        PathBuf::from("C:/project/assets"),
        RelPath::parse("textures/shared.png").unwrap(),
        PathBuf::from("C:/project/assets/textures/shared.png"),
    );
    let aliases =
        MigrationResolverIndex::build([assets_projection.clone(), assets_projection.clone()], [])
            .unwrap();
    assert_eq!(
        aliases.project_hint_for_locator(&locator).unwrap(),
        RelPath::parse("assets/textures/shared.png").unwrap()
    );

    let ambiguous = MigrationResolverIndex::build(
        [
            assets_projection,
            MigrationSourceProjection::new(
                RelPath::parse("content").unwrap(),
                PathBuf::from("C:/project/assets"),
                RelPath::parse("textures/shared.png").unwrap(),
                PathBuf::from("C:/project/assets/textures/shared.png"),
            ),
        ],
        [],
    )
    .unwrap();
    assert!(matches!(
        ambiguous.project_hint_for_locator(&locator),
        Err(ReferenceResolutionError::AmbiguousPath { .. })
    ));

    let four_roots = MigrationResolverIndex::build(
        ["assets", "content", "plugins", "shared"]
            .into_iter()
            .map(|logical_root| {
                MigrationSourceProjection::new(
                    RelPath::parse(logical_root).unwrap(),
                    PathBuf::from(format!("C:/project/{logical_root}")),
                    RelPath::parse("textures/shared.png").unwrap(),
                    PathBuf::from(format!("C:/project/{logical_root}/textures/shared.png")),
                )
            }),
        [],
    )
    .unwrap();
    assert!(matches!(
        four_roots.project_hint_for_locator(&locator),
        Err(ReferenceResolutionError::AmbiguousPath { .. })
    ));
}

#[test]
fn resolver_index_adds_only_validated_compound_bindings() {
    let physical_root = PathBuf::from("C:/project/assets");
    let physical_meta = physical_root.join("shaders/redirect_surface.zmeta");
    let projection = MigrationSourceProjection::new(
        RelPath::parse("assets").unwrap(),
        physical_root,
        RelPath::parse("shaders/redirect_surface.zmeta").unwrap(),
        physical_meta.clone(),
    );
    let locator = AssetUri::parse("res://shaders/redirect_surface").unwrap();
    let index = MigrationResolverIndex::build(
        [projection],
        [MigrationCompoundBinding::new(
            locator.clone(),
            physical_meta,
        )],
    )
    .unwrap();

    assert_eq!(
        index.project_hint_for_locator(&locator).unwrap(),
        RelPath::parse("assets/shaders/redirect_surface.zmeta").unwrap()
    );
    assert_eq!(
        index
            .locator_for_project_hint(
                &RelPath::parse("assets/shaders/redirect_surface.zmeta").unwrap()
            )
            .unwrap(),
        Some(locator)
    );

    let mismatched = MigrationResolverIndex::build(
        [MigrationSourceProjection::new(
            RelPath::parse("assets").unwrap(),
            PathBuf::from("C:/project/assets"),
            RelPath::parse("shaders/not_redirect.zmeta").unwrap(),
            PathBuf::from("C:/project/assets/shaders/not_redirect.zmeta"),
        )],
        [MigrationCompoundBinding::new(
            AssetUri::parse("res://shaders/redirect_surface").unwrap(),
            PathBuf::from("C:/project/assets/shaders/not_redirect.zmeta"),
        )],
    )
    .unwrap();
    assert!(matches!(
        mismatched
            .project_hint_for_locator(&AssetUri::parse("res://shaders/redirect_surface").unwrap()),
        Err(ReferenceResolutionError::MissingPath { .. })
    ));
}

#[test]
fn direct_source_precedes_same_root_compound_candidate_but_zmeta_hint_stays_compound() {
    let physical_root = PathBuf::from("C:/project/assets");
    let direct_path = physical_root.join("shaders/redirect_surface");
    let meta_path = physical_root.join("shaders/redirect_surface.zmeta");
    let locator = AssetUri::parse("res://shaders/redirect_surface").unwrap();
    let index = MigrationResolverIndex::build(
        [
            MigrationSourceProjection::new(
                RelPath::parse("assets").unwrap(),
                physical_root.clone(),
                RelPath::parse("shaders/redirect_surface").unwrap(),
                direct_path,
            ),
            MigrationSourceProjection::new(
                RelPath::parse("assets").unwrap(),
                physical_root,
                RelPath::parse("shaders/redirect_surface.zmeta").unwrap(),
                meta_path.clone(),
            ),
        ],
        [MigrationCompoundBinding::new(locator.clone(), meta_path)],
    )
    .unwrap();

    assert_eq!(
        index.project_hint_for_locator(&locator).unwrap(),
        RelPath::parse("assets/shaders/redirect_surface").unwrap()
    );
    assert_eq!(
        index
            .locator_for_project_hint(
                &RelPath::parse("assets/shaders/redirect_surface.zmeta").unwrap()
            )
            .unwrap(),
        Some(locator)
    );
}

#[test]
fn compound_zmeta_binding_produces_the_persisted_project_hint() {
    let root = fixture_root("indexed-resolver-compound");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "a4111111-2222-4333-8444-555555555555".parse().unwrap();
    let locator = AssetUri::parse("res://shaders/redirect_surface").unwrap();
    let compound_root = root.join("assets/shaders/redirect_surface");
    fs::create_dir_all(&compound_root).unwrap();
    let mut meta = crate::asset::project::AssetMetaDocument::new(guid, locator, AssetKind::Shader);
    meta.unit = crate::asset::project::AssetSourceUnit::Compound;
    meta.save(root.join("assets/shaders/redirect_surface.zmeta"))
        .unwrap();
    let material = root.join("assets/materials/redirect.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/redirect_surface\"\n"
        ),
    )
    .unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded(), "{}", report.format_text());
    let migrated: toml::Value = toml::from_str(&fs::read_to_string(&material).unwrap()).unwrap();
    assert_eq!(
        migrated["shader"]["guid"].as_str(),
        Some(guid.to_string().as_str())
    );
    assert_eq!(
        migrated["shader"]["path_hint"].as_str(),
        Some("assets/shaders/redirect_surface.zmeta")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retired_compound_meta_toml_uses_the_published_zmeta_hint_in_the_same_transaction() {
    let root = fixture_root("indexed-resolver-retired-compound");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "a5111111-2222-4333-8444-555555555555".parse().unwrap();
    let compound_root = root.join("assets/shaders/legacy_redirect_surface");
    fs::create_dir_all(&compound_root).unwrap();
    let retired_meta = root.join("assets/shaders/legacy_redirect_surface.meta.toml");
    fs::write(
        &retired_meta,
        format!(
            "format_version = 6\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy_redirect_surface\"\nasset_kind = \"Shader\"\nunit = \"compound\"\nsource_hash = \"legacy-redirect-digest\"\n"
        ),
    )
    .unwrap();
    let material = root.join("assets/materials/legacy_redirect.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy_redirect_surface\"\n"
        ),
    )
    .unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded(), "{}", report.format_text());
    let migrated: toml::Value = toml::from_str(&fs::read_to_string(&material).unwrap()).unwrap();
    assert_eq!(
        migrated["shader"]["path_hint"].as_str(),
        Some("assets/shaders/legacy_redirect_surface.zmeta")
    );
    assert!(!retired_meta.exists());
    assert!(root
        .join("assets/shaders/legacy_redirect_surface.zmeta")
        .is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retired_compound_meta_toml_defers_to_an_identical_current_zmeta() {
    let root = fixture_root("indexed-resolver-retired-compound-current");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "a6111111-2222-4333-8444-555555555555".parse().unwrap();
    let compound_root = root.join("assets/shaders/legacy_redirect_surface");
    fs::create_dir_all(&compound_root).unwrap();
    let retired_meta = root.join("assets/shaders/legacy_redirect_surface.meta.toml");
    let retired_source = format!(
        "format_version = 6\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy_redirect_surface\"\nasset_kind = \"Shader\"\nunit = \"compound\"\nsource_hash = \"legacy-redirect-digest\"\n"
    );
    fs::write(&retired_meta, &retired_source).unwrap();
    let mut current_value = toml::from_str::<toml::Value>(&retired_source).unwrap();
    let current_table = current_value.as_table_mut().unwrap();
    current_table.insert("format_version".to_string(), toml::Value::Integer(7));
    let source_hash = current_table.remove("source_hash").unwrap();
    current_table.insert("source_digest".to_string(), source_hash);
    let current_meta = root.join("assets/shaders/legacy_redirect_surface.zmeta");
    fs::write(
        &current_meta,
        toml::to_string_pretty(&current_value).unwrap(),
    )
    .unwrap();
    let material = root.join("assets/materials/legacy_redirect.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy_redirect_surface\"\n"
        ),
    )
    .unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded(), "{}", report.format_text());
    let migrated: toml::Value = toml::from_str(&fs::read_to_string(&material).unwrap()).unwrap();
    assert_eq!(
        migrated["shader"]["path_hint"].as_str(),
        Some("assets/shaders/legacy_redirect_surface.zmeta")
    );
    assert!(!retired_meta.exists());
    assert!(current_meta.is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn duplicate_registry_entry_precedes_resolver_ambiguity() {
    let root = fixture_root("indexed-resolver-registry-conflict");
    write_manifest(&root, &["assets", "content"]);
    let first: AssetUuid = "b4111111-2222-4333-8444-555555555555".parse().unwrap();
    let second: AssetUuid = "c4111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "textures/shared.png",
        first,
        AssetKind::Texture,
    );
    write_registered_source(
        &root,
        "content",
        "textures/shared.png",
        second,
        AssetKind::Texture,
    );

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();

    assert!(!report.succeeded());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::RegistryConflict
    );
    fs::remove_dir_all(root).unwrap();
}
