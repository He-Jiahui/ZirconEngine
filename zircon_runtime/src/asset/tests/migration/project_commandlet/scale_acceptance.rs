use super::*;

const FILE_CARDINALITIES: [usize; 3] = [1, 1_000, 100_000];
const DIRECTORY_CARDINALITIES: [usize; 3] = [1, 1_000, 100_000];
const REFERENCE_CARDINALITIES: [usize; 3] = [1, 1_000, 100_000];
const ROOT_CARDINALITIES: [usize; 2] = [1, 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MigrationScalePhase {
    DryRun,
    Apply,
    Unchanged,
    OnePercentChange,
}

const SCALE_PHASES: [MigrationScalePhase; 4] = [
    MigrationScalePhase::DryRun,
    MigrationScalePhase::Apply,
    MigrationScalePhase::Unchanged,
    MigrationScalePhase::OnePercentChange,
];

#[test]
fn scale_acceptance_matrix_declares_every_required_dimension() {
    assert_eq!(FILE_CARDINALITIES, [1, 1_000, 100_000]);
    assert_eq!(DIRECTORY_CARDINALITIES, [1, 1_000, 100_000]);
    assert_eq!(REFERENCE_CARDINALITIES, [1, 1_000, 100_000]);
    assert_eq!(ROOT_CARDINALITIES, [1, 4]);
    assert_eq!(
        SCALE_PHASES,
        [
            MigrationScalePhase::DryRun,
            MigrationScalePhase::Apply,
            MigrationScalePhase::Unchanged,
            MigrationScalePhase::OnePercentChange,
        ]
    );

    let case_count = FILE_CARDINALITIES.len()
        * DIRECTORY_CARDINALITIES.len()
        * REFERENCE_CARDINALITIES.len()
        * ROOT_CARDINALITIES.len()
        * SCALE_PHASES.len();
    assert_eq!(case_count, 216);
}

#[test]
fn focused_dry_run_reports_production_work_counters() {
    let root = fixture_root("scale-acceptance-counters");
    write_manifest(&root, &["assets"]);
    let shader_guid: AssetUuid = "8a111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
    let material = root.join("assets/materials/hero.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\nname = \"Hero\"\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        ),
    )
    .unwrap();

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert!(report.succeeded());
    assert_eq!(report.scanned_files(), 1);
    assert_eq!(
        report
            .changed_files()
            .iter()
            .filter(|change| change.path() == material)
            .count(),
        1
    );

    let metrics = *report.metrics();
    fs::remove_dir_all(root).unwrap();

    assert!(metrics.entry_visits() >= report.scanned_files());
    assert!(metrics.directory_reads() > 0);
    assert_eq!(metrics.directory_sorts(), metrics.directory_reads());
    assert_eq!(metrics.resolver_filesystem_probes(), 0);
    assert_eq!(metrics.document_reads(), report.scanned_files());
    assert_eq!(metrics.document_parses(), report.scanned_files());
    assert_eq!(metrics.reference_visits(), 3);
    assert_eq!(metrics.full_value_clones(), 0);
    assert!(metrics.output_bytes() > 0);
}

#[test]
fn unchanged_document_reports_real_lookup_work_without_output_bytes() {
    let root = fixture_root("scale-acceptance-unchanged-counters");
    write_manifest(&root, &["assets"]);
    let shader_guid: AssetUuid = "9a111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
    let material = root.join("assets/materials/stable.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\\n\\n[shader]\\nkind = \"project\"\\nguid = \"{shader_guid}\"\\npath_hint = \"assets/shaders/pbr.zshader\"\\n"
        ),
    )
    .unwrap();

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    let metrics = *report.metrics();
    fs::remove_dir_all(root).unwrap();

    assert!(report.succeeded());
    assert!(report.changed_files().is_empty());
    assert_eq!(metrics.document_reads(), 1);
    assert_eq!(metrics.document_parses(), 1);
    assert!(metrics.reference_visits() >= 2);
    assert_eq!(metrics.output_bytes(), 0);
}

#[test]
fn production_counters_cover_dry_apply_unchanged_and_one_percent_change() {
    const DOCUMENT_COUNT: usize = 100;
    let root = fixture_root("scale-acceptance-phases");
    let shader_guid: AssetUuid = "aa111111-2222-4333-8444-555555555555".parse().unwrap();
    setup_project(&root, &["assets"], shader_guid);
    write_retired_materials(&root, "assets", DOCUMENT_COUNT, shader_guid);

    let dry_run = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert_production_counters(&dry_run, DOCUMENT_COUNT);
    assert!(!dry_run.applied());
    assert_eq!(dry_run.changed_files().len(), DOCUMENT_COUNT);
    assert!(dry_run.metrics().output_bytes() > 0);

    let apply =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert_production_counters(&apply, DOCUMENT_COUNT);
    assert!(apply.applied());
    assert_eq!(apply.changed_files().len(), DOCUMENT_COUNT);

    let unchanged = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert_production_counters(&unchanged, DOCUMENT_COUNT);
    assert!(unchanged.changed_files().is_empty());
    assert_eq!(unchanged.metrics().output_bytes(), 0);

    write_retired_material(&root, "assets", 0, shader_guid);
    let one_percent_change = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert_production_counters(&one_percent_change, DOCUMENT_COUNT);
    assert_eq!(
        one_percent_change.changed_files().len(),
        DOCUMENT_COUNT / 100
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn four_roots_share_one_inventory_generation() {
    let root = fixture_root("scale-acceptance-four-roots");
    let roots = ["assets-a", "assets-b", "assets-c", "assets-d"];
    let shader_guid: AssetUuid = "ab111111-2222-4333-8444-555555555555".parse().unwrap();
    setup_project(&root, &roots, shader_guid);
    for asset_root in roots {
        write_retired_materials(&root, asset_root, 1, shader_guid);
    }

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert_production_counters(&report, roots.len());
    assert_eq!(report.changed_files().len(), roots.len());

    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires the managed 1/1k/100k migration scale lane"]
fn managed_scale_sweep_executes_declared_cardinalities() {
    for document_count in FILE_CARDINALITIES {
        let root = fixture_root(&format!("scale-files-{document_count}"));
        let shader_guid: AssetUuid = "ac111111-2222-4333-8444-555555555555".parse().unwrap();
        setup_project(&root, &["assets"], shader_guid);
        write_retired_materials(&root, "assets", document_count, shader_guid);

        let report = migrate_project_assets(AssetMigrationOptions::new(
            &root,
            AssetMigrationMode::DryRun,
        ))
        .unwrap();
        assert_production_counters(&report, document_count);
        assert!(report.metrics().reference_visits() >= document_count * 3);

        fs::remove_dir_all(root).unwrap();
    }

    for reference_count in REFERENCE_CARDINALITIES {
        let root = fixture_root(&format!("scale-references-{reference_count}"));
        let shader_guid: AssetUuid = "ad111111-2222-4333-8444-555555555555".parse().unwrap();
        let texture_guid: AssetUuid = "ae111111-2222-4333-8444-555555555555".parse().unwrap();
        setup_project(&root, &["assets"], shader_guid);
        write_registered_source(
            &root,
            "assets",
            "textures/albedo.ztexture",
            texture_guid,
            AssetKind::Texture,
        );
        write_reference_dense_material(&root, reference_count, shader_guid, texture_guid);

        let report = migrate_project_assets(AssetMigrationOptions::new(
            &root,
            AssetMigrationMode::DryRun,
        ))
        .unwrap();
        assert_production_counters(&report, 1);
        assert!(report.metrics().reference_visits() >= (reference_count + 1) * 3);

        fs::remove_dir_all(root).unwrap();
    }

    for directory_count in DIRECTORY_CARDINALITIES {
        let root = fixture_root(&format!("scale-directories-{directory_count}"));
        write_manifest(&root, &["assets"]);
        write_empty_directories(&root, directory_count);

        let report = migrate_project_assets(AssetMigrationOptions::new(
            &root,
            AssetMigrationMode::DryRun,
        ))
        .unwrap();
        assert!(report.succeeded());
        assert_eq!(report.scanned_files(), 0);
        assert_eq!(report.metrics().document_reads(), 0);
        assert_eq!(report.metrics().document_parses(), 0);
        assert!(report.metrics().directory_reads() >= directory_count);
        assert_eq!(
            report.metrics().directory_sorts(),
            report.metrics().directory_reads()
        );

        fs::remove_dir_all(root).unwrap();
    }
}

fn setup_project(root: &std::path::Path, asset_roots: &[&str], shader_guid: AssetUuid) {
    write_manifest(root, asset_roots);
    write_registered_source(
        root,
        asset_roots[0],
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
}

fn write_retired_materials(
    root: &std::path::Path,
    asset_root: &str,
    document_count: usize,
    shader_guid: AssetUuid,
) {
    for index in 0..document_count {
        write_retired_material(root, asset_root, index, shader_guid);
    }
}

fn write_retired_material(
    root: &std::path::Path,
    asset_root: &str,
    index: usize,
    shader_guid: AssetUuid,
) {
    let material = root
        .join(asset_root)
        .join("materials")
        .join(format!("material-{index:06}.zmaterial"));
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        material,
        format!(
            "version = 2\nname = \"Material {index}\"\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        ),
    )
    .unwrap();
}

fn write_reference_dense_material(
    root: &std::path::Path,
    reference_count: usize,
    shader_guid: AssetUuid,
    texture_guid: AssetUuid,
) {
    let material = root.join("assets/materials/references.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    let mut source = format!(
        "version = 2\nname = \"Reference scale\"\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n\n[textures]\n"
    );
    for index in 0..reference_count {
        source.push_str(&format!(
            "slot_{index:06} = {{ uuid = \"{texture_guid}\", url = \"res://textures/albedo.ztexture\" }}\n"
        ));
    }
    fs::write(material, source).unwrap();
}

fn write_empty_directories(root: &std::path::Path, directory_count: usize) {
    for index in 0..directory_count {
        fs::create_dir_all(
            root.join("assets/directories")
                .join(format!("dir-{index:06}")),
        )
        .unwrap();
    }
}

fn assert_production_counters(
    report: &crate::asset::migration::AssetMigrationReport,
    document_count: usize,
) {
    assert!(report.succeeded());
    assert_eq!(report.scanned_files(), document_count);
    assert_eq!(report.metrics().document_reads(), document_count);
    assert_eq!(report.metrics().document_parses(), document_count);
    assert!(report.metrics().entry_visits() >= document_count);
    assert_eq!(
        report.metrics().directory_sorts(),
        report.metrics().directory_reads()
    );
    assert_eq!(report.metrics().resolver_filesystem_probes(), 0);
    assert_eq!(report.metrics().full_value_clones(), 0);
}
