use super::*;

#[test]
fn runtime_15_scene_dynamic_document_uses_value_migration_owner() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let document_dir = manifest_root.join("src/scene/dynamic_scene/document");
    let migration_dir = document_dir.join("migration");

    for retired in ["legacy.rs", "v1_project_document.rs"] {
        assert!(
            !document_dir.join(retired).exists(),
            "dynamic scene hard cut must not restore retired owner {retired}"
        );
    }
    for current in ["mod.rs", "project_world.rs"] {
        assert!(
            migration_dir.join(current).exists(),
            "dynamic scene value migration should keep {current}"
        );
    }

    let document_mod = read_text(
        &document_dir.join("mod.rs"),
        "dynamic scene document module entry should be readable",
    );
    assert_contains_all(
        "dynamic scene document module",
        &document_mod,
        &["mod migration;", "mod read;", "mod schema;", "mod write;"],
    );
    assert!(
        !document_mod.contains("v1_project_document") && !document_mod.contains("mod legacy;"),
        "dynamic scene document module must not mount retired DTO owners"
    );

    let migration = read_text(
        &migration_dir.join("project_world.rs"),
        "dynamic scene project-world value migration should be readable",
    );
    assert_contains_all(
        "dynamic scene project-world migration",
        &migration,
        &["serde_json::{json, Map, Value}", "migrate_project_world"],
    );
    for forbidden in [
        "V1ProjectDocument",
        "serde::Deserialize",
        "from_value::<World>",
    ] {
        assert!(
            !migration.contains(forbidden),
            "dynamic scene migration must not retain typed legacy decode `{forbidden}`"
        );
    }
}
