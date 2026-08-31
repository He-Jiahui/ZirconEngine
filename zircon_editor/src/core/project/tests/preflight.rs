use std::fs;

use super::super::preflight_manifest_reader::MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES;
use super::super::{
    ProjectAuthority, ProjectAuthorityError, ProjectManifestMigrationAction,
    ProjectManifestMigrationDecision, ProjectPreflightCompositionProfile,
    ProjectPreflightRevalidation,
};
use super::temp_root;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::asset::project::ProjectScriptManifest;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime_interface::project::{
    ProjectEngineCompatibilityDisposition, ProjectEngineVersion, ProjectManifestDigest,
    ProjectManifestSummary, PROJECT_MANIFEST_FORMAT_VERSION,
};

#[test]
fn recent_project_validation_defaults_to_fail_closed() {
    assert_eq!(
        super::super::RecentProjectValidation::default(),
        super::super::RecentProjectValidation::InvalidProject
    );
}

#[test]
fn current_preflight_receipt_requires_a_persisted_project_guid() {
    let root = temp_root("preflight-current-guid-invariant");
    fs::write(
        root.join("zircon-project.toml"),
        current_manifest("Current Guid"),
    )
    .unwrap();
    let resolved = ProjectPaths::resolve_existing(&root).unwrap();
    let composition = super::super::ProjectPreflightCompositionPlan::compile(
        ProjectPreflightCompositionProfile::Normal,
        &ProjectPluginManifest::default(),
        &ProjectScriptManifest::default(),
    );

    let error = super::super::ProjectPreflightReceipt::new(
        resolved,
        ProjectManifestSummary {
            name: "Current Guid".to_string(),
            engine_version_req: None,
            default_scene: "res://scenes/main.scene.toml".to_string(),
            format_version: PROJECT_MANIFEST_FORMAT_VERSION,
            project_guid: None,
        },
        composition,
        ProjectManifestMigrationDecision::Current,
        ProjectManifestDigest::from_bytes(b"current manifest without a project GUID"),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectAuthorityError::CurrentManifestMissingProjectGuid
    ));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preflight_composition_is_not_a_public_pre_admission_capability() {
    let receipt_source = include_str!("../project_preflight/preflight_receipt.rs");
    let plan_source = include_str!("../preflight_composition/plan.rs");

    assert!(receipt_source.contains("pub(crate) fn composition"));
    assert!(!receipt_source.contains("pub fn composition"));
    assert!(plan_source.contains("pub(crate) struct ProjectPreflightCompositionPlan"));
    assert!(plan_source.contains("pub(crate) fn approved_project_plugins"));
    assert!(plan_source.contains("pub(crate) fn approved_project_scripts"));
}

#[test]
fn project_preflight_preserves_canonical_identity_without_opening_runtime_project_state() {
    let root = temp_root("preflight-data-only");
    fs::write(
        root.join("zircon-project.toml"),
        current_manifest("Data Only"),
    )
    .unwrap();

    let receipt = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap();

    assert_eq!(
        receipt.root(),
        ProjectPaths::resolve_existing(&root)
            .unwrap()
            .operation_path()
    );
    assert_eq!(receipt.canonical_descriptor().path(), receipt.root());
    assert_eq!(
        receipt
            .project_identity()
            .expect("current manifest must have a typed project identity")
            .canonical_descriptor(),
        receipt.canonical_descriptor()
    );
    assert_eq!(
        receipt
            .project_identity()
            .expect("current manifest must have a typed project identity")
            .project_guid(),
        receipt
            .summary()
            .project_guid
            .expect("current manifest GUID")
    );
    assert_eq!(
        receipt
            .project_identity()
            .expect("current manifest must have a typed project identity")
            .manifest_digest(),
        receipt.manifest_digest()
    );
    assert_eq!(receipt.summary().name, "Data Only");
    assert_eq!(
        receipt.manifest_migration(),
        ProjectManifestMigrationDecision::Current
    );
    assert!(!receipt.manifest_migration().blocks_activation());
    assert!(
        !root.join(".zircon").exists(),
        "preflight must not create runtime derived state"
    );
    assert!(
        !root.join("assets").exists(),
        "preflight must not materialize manifest asset roots"
    );

    fs::write(
        root.join("zircon-project.toml"),
        current_manifest("Changed"),
    )
    .unwrap();
    let changed = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap();
    assert_ne!(
        receipt.manifest_digest(),
        changed.manifest_digest(),
        "admission must be able to detect a manifest replacement after preflight"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_preflight_reports_migration_as_an_explicit_admission_decision() {
    let root = temp_root("preflight-migration-decision");
    fs::write(
        root.join("zircon-project.toml"),
        "name = \"Needs Migration\"\nformat_version = 1\ndefault_scene = \"res://scenes/main.scene.toml\"\nlibrary_version = 1\n",
    )
    .unwrap();

    let receipt = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap();

    assert_eq!(
        receipt.summary().format_version,
        PROJECT_MANIFEST_FORMAT_VERSION
    );
    let ProjectManifestMigrationDecision::RequiresExplicitDecision { plan } =
        receipt.manifest_migration()
    else {
        panic!("legacy manifest must require a migration decision");
    };
    assert_eq!(plan.source_format_version(), 1);
    assert_eq!(receipt.summary().project_guid, None);
    assert_eq!(receipt.project_identity(), None);
    assert!(!receipt.composition().allows_project_scripts());
    assert!(!receipt.composition().allows_native_extensions());
    assert!(!receipt.composition().allows_scene_restore());
    assert!(!root.join(".zircon").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_preflight_exposes_copy_convert_and_cancel_without_executing_any_action() {
    let root = temp_root("preflight-migration-actions");
    fs::write(
        root.join("zircon-project.toml"),
        "name = \"Needs Migration\"\nformat_version = 1\ndefault_scene = \"res://scenes/main.scene.toml\"\nlibrary_version = 1\n",
    )
    .unwrap();

    let receipt = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap();
    assert!(receipt.manifest_migration().blocks_activation());
    let ProjectManifestMigrationDecision::RequiresExplicitDecision { plan } =
        receipt.manifest_migration()
    else {
        panic!("legacy manifest must require an explicit migration decision");
    };

    assert_eq!(
        plan.available_actions(),
        [
            ProjectManifestMigrationAction::OpenCopy,
            ProjectManifestMigrationAction::ConvertInPlace,
            ProjectManifestMigrationAction::Cancel,
        ]
    );
    assert!(!ProjectManifestMigrationAction::OpenCopy.mutates_source());
    assert!(!ProjectManifestMigrationAction::OpenCopy.requires_source_backup());
    assert!(ProjectManifestMigrationAction::ConvertInPlace.mutates_source());
    assert!(ProjectManifestMigrationAction::ConvertInPlace.requires_source_backup());
    assert!(ProjectManifestMigrationAction::ConvertInPlace.requires_fresh_preflight());
    assert!(ProjectManifestMigrationAction::Cancel.cancels_launch());
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recent_project_validation_marks_legacy_manifests_for_explicit_migration() {
    let root = temp_root("recent-preflight-migration-required");
    fs::write(
        root.join("zircon-project.toml"),
        "name = \"Needs Migration\"\nformat_version = 1\ndefault_scene = \"res://scenes/main.scene.toml\"\nlibrary_version = 1\n",
    )
    .unwrap();

    assert_eq!(
        ProjectAuthority::default().validate_recent_project(root.to_str().unwrap()),
        super::super::RecentProjectValidation::RequiresMigration
    );
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_preflight_exposes_the_engine_compatibility_receipt_without_runtime_activation() {
    let root = temp_root("preflight-engine-compatibility");
    fs::write(
        root.join("zircon-project.toml"),
        format!(
            "{}\nengine_version_req = \">=0.1.0, <0.2.0\"\n",
            current_manifest("Engine Compatibility")
        ),
    )
    .unwrap();
    let receipt = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap();

    let compatible = receipt
        .evaluate_engine_compatibility(&ProjectEngineVersion::parse("0.1.7").unwrap())
        .unwrap();
    assert_eq!(
        compatible.disposition(),
        ProjectEngineCompatibilityDisposition::Compatible
    );

    let incompatible = receipt
        .evaluate_engine_compatibility(&ProjectEngineVersion::parse("0.2.0").unwrap())
        .unwrap();
    assert_eq!(
        incompatible.disposition(),
        ProjectEngineCompatibilityDisposition::Incompatible
    );
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_preflight_revalidation_reports_manifest_replacement_without_opening_project_state() {
    let root = temp_root("preflight-revalidation");
    fs::write(
        root.join("zircon-project.toml"),
        current_manifest("Original"),
    )
    .unwrap();
    let authority = ProjectAuthority::default();
    let approved = authority.preflight_project(&root).unwrap();

    let unchanged = authority.revalidate_preflight(&approved).unwrap();
    assert!(matches!(
        unchanged,
        ProjectPreflightRevalidation::Unchanged { .. }
    ));

    fs::write(
        root.join("zircon-project.toml"),
        current_manifest("Replacement"),
    )
    .unwrap();
    let changed = authority.revalidate_preflight(&approved).unwrap();
    match changed {
        ProjectPreflightRevalidation::Changed { expected, observed } => {
            assert_eq!(expected, approved.manifest_digest());
            assert_ne!(observed.manifest_digest(), expected);
            assert_eq!(observed.summary().name, "Replacement");
        }
        ProjectPreflightRevalidation::Unchanged { .. } => {
            panic!("revalidation must reject a manifest replacement")
        }
    }
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_preflight_rejects_an_oversized_manifest_before_runtime_project_open() {
    let root = temp_root("preflight-bounded-manifest");
    let manifest_path = root.join("zircon-project.toml");
    fs::write(
        &manifest_path,
        vec![b'x'; MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES + 1],
    )
    .unwrap();

    let error = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap_err();
    assert!(matches!(
        error,
        ProjectAuthorityError::ManifestPreflightTooLarge {
            path,
            max_bytes: MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES,
        } if path == manifest_path
    ));
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recent_project_validation_uses_the_bounded_data_only_manifest_reader() {
    let root = temp_root("recent-preflight-bounded-manifest");
    fs::write(
        root.join("zircon-project.toml"),
        vec![b'x'; MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES + 1],
    )
    .unwrap();

    assert_eq!(
        ProjectAuthority::default().validate_recent_project(root.to_str().unwrap()),
        super::super::RecentProjectValidation::InvalidManifest
    );
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn safe_preflight_compiles_a_non_executing_composition_plan_and_revalidation_keeps_it_safe() {
    let root = temp_root("preflight-safe-composition");
    fs::write(
        root.join("zircon-project.toml"),
        manifest_with_project_derived_code("project.native-editor"),
    )
    .unwrap();
    let authority = ProjectAuthority::default();

    let approved = authority
        .preflight_project_with_composition_profile(&root, ProjectPreflightCompositionProfile::Safe)
        .unwrap();
    let composition = approved.composition();
    assert_eq!(
        composition.profile(),
        ProjectPreflightCompositionProfile::Safe
    );
    assert!(composition.approved_project_plugins().selections.is_empty());
    assert!(composition.approved_project_scripts().is_empty());
    assert!(!composition.allows_project_scripts());
    assert!(!composition.allows_native_extensions());
    assert!(!composition.allows_scene_restore());

    let revalidated = authority.revalidate_preflight(&approved).unwrap();
    let ProjectPreflightRevalidation::Unchanged { current } = revalidated else {
        panic!("an unchanged safe preflight must remain unchanged")
    };
    assert_eq!(
        current.composition().profile(),
        ProjectPreflightCompositionProfile::Safe
    );
    assert!(current
        .composition()
        .approved_project_plugins()
        .selections
        .is_empty());
    assert!(current.composition().approved_project_scripts().is_empty());
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_preflight_also_excludes_all_project_derived_composition_inputs() {
    let root = temp_root("preflight-recovery-composition");
    fs::write(
        root.join("zircon-project.toml"),
        manifest_with_project_derived_code("project.recovery-native"),
    )
    .unwrap();

    let receipt = ProjectAuthority::default()
        .preflight_project_with_composition_profile(
            &root,
            ProjectPreflightCompositionProfile::Recovery,
        )
        .unwrap();
    let composition = receipt.composition();
    assert_eq!(
        composition.profile(),
        ProjectPreflightCompositionProfile::Recovery
    );
    assert!(composition.approved_project_plugins().selections.is_empty());
    assert!(composition.approved_project_scripts().is_empty());
    assert!(!composition.allows_project_scripts());
    assert!(!composition.allows_native_extensions());
    assert!(!composition.allows_scene_restore());
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn normal_preflight_keeps_static_project_manifest_inputs_for_later_approved_composition() {
    let root = temp_root("preflight-normal-composition");
    fs::write(
        root.join("zircon-project.toml"),
        manifest_with_project_derived_code("project.runtime"),
    )
    .unwrap();

    let receipt = ProjectAuthority::default()
        .preflight_project(&root)
        .unwrap();
    let composition = receipt.composition();
    assert_eq!(
        composition.profile(),
        ProjectPreflightCompositionProfile::Normal
    );
    assert_eq!(composition.approved_project_plugins().selections.len(), 1);
    assert_eq!(
        composition.approved_project_plugins().selections[0].id,
        "project.runtime"
    );
    assert_eq!(
        composition.approved_project_scripts().package_roots,
        vec!["scripts".to_string()]
    );
    assert_eq!(
        composition.approved_project_scripts().startup_packages,
        vec!["preflight_fixture".to_string()]
    );
    assert!(composition.allows_project_scripts());
    assert!(composition.allows_native_extensions());
    assert!(composition.allows_scene_restore());
    assert!(!root.join(".zircon").exists());
    assert!(!root.join("assets").exists());

    fs::remove_dir_all(root).unwrap();
}

fn current_manifest(name: &str) -> String {
    format!(
        "name = \"{name}\"\nformat_version = {PROJECT_MANIFEST_FORMAT_VERSION}\nproject_guid = \"9df4a497-e3b9-4871-8d4d-eefc15ab42ef\"\ndefault_scene = \"res://scenes/main.scene.toml\"\nlibrary_version = 1\n"
    )
}

fn manifest_with_project_derived_code(plugin_id: &str) -> String {
    format!(
        "{}\n[[plugins.selections]]\nid = \"{plugin_id}\"\nrequired = true\n\n[scripts]\npackage_roots = [\"scripts\"]\nstartup_packages = [\"preflight_fixture\"]\n",
        current_manifest("Composition Policy")
    )
}
