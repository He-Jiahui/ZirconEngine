#[test]
fn dynamic_scene_root_owner_tree_stays_folder_backed_after_runtime_05_cutover() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dynamic_scene_root = manifest_root.join("src/scene/dynamic_scene");
    let root_owner_children: &[(&str, &[&str])] = &[
        (
            "document",
            &[
                "migration/mod.rs",
                "migration/project_world.rs",
                "read.rs",
                "schema.rs",
                "write.rs",
            ],
        ),
        (
            "entity",
            &[
                "dynamic_component.rs",
                "dynamic_entity.rs",
                "dynamic_resource.rs",
            ],
        ),
        ("scene", &["capture.rs", "spawn.rs", "validation.rs"]),
        (
            "scene_asset",
            &["dynamic_scene.rs", "error.rs", "prepared_spawn.rs"],
        ),
        ("spawn_task", &["loader.rs", "prepared.rs", "task.rs"]),
        ("value", &["json.rs", "remap.rs"]),
    ];

    let root_mod_source = std::fs::read_to_string(dynamic_scene_root.join("mod.rs")).unwrap();
    for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
        assert!(
            !root_mod_source.contains(forbidden),
            "dynamic_scene/mod.rs should remain a structural module/export owner, but found `{forbidden}`"
        );
    }

    for &(owner, children) in root_owner_children {
        let owner_root = dynamic_scene_root.join(owner);
        assert!(
            owner_root.join("mod.rs").exists(),
            "Runtime 05 dynamic_scene owner `{owner}` should stay folder-backed"
        );
        assert!(
            !dynamic_scene_root.join(format!("{owner}.rs")).exists(),
            "retired flat dynamic_scene/{owner}.rs should not be restored"
        );
        assert!(
            root_mod_source.contains(&format!("mod {owner};")),
            "dynamic_scene/mod.rs should continue declaring owner module `{owner}`"
        );

        for &child in children {
            assert!(
                owner_root.join(child).exists(),
                "Runtime 05 dynamic_scene owner `{owner}` should keep child `{child}`"
            );
        }
    }

    for owner in ["document", "entity", "scene_asset", "spawn_task", "value"] {
        let mod_source =
            std::fs::read_to_string(dynamic_scene_root.join(owner).join("mod.rs")).unwrap();
        for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
            assert!(
                !mod_source.contains(forbidden),
                "dynamic_scene/{owner}/mod.rs should remain a structural module/export owner, but found `{forbidden}`"
            );
        }
    }

    let scene_mod_source =
        std::fs::read_to_string(dynamic_scene_root.join("scene").join("mod.rs")).unwrap();
    for required in [
        "pub struct DynamicScene",
        "pub fn empty()",
        "pub fn from_world(",
        "pub fn spawn_into(",
        "pub fn ensure_supported(",
    ] {
        assert!(
            scene_mod_source.contains(required),
            "dynamic_scene/scene/mod.rs should keep DynamicScene facade anchor `{required}`"
        );
    }
    for retired in ["DYNAMIC_SCENE_FORMAT_VERSION", "pub format_version"] {
        assert!(
            !scene_mod_source.contains(retired) && !root_mod_source.contains(retired),
            "Plan 11 M2.2 retired DynamicScene version surface `{retired}` must stay deleted"
        );
    }
}

#[test]
fn dynamic_scene_session_owner_tree_stays_folder_backed_after_runtime_05_cutover() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dynamic_scene_root = manifest_root.join("src/scene/dynamic_scene");
    let session_root = dynamic_scene_root.join("session");

    assert!(
        session_root.join("mod.rs").exists(),
        "runtime session archive should stay folder-backed under src/scene/dynamic_scene/session"
    );
    assert!(
        !dynamic_scene_root.join("session.rs").exists(),
        "retired flat dynamic_scene/session.rs should not be restored"
    );

    let session_mod_source = std::fs::read_to_string(session_root.join("mod.rs")).unwrap();
    for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
        assert!(
            !session_mod_source.contains(forbidden),
            "session/mod.rs should remain a structural module/export owner, but found `{forbidden}`"
        );
    }

    for relative in [
        "capture_retention",
        "construction",
        "facade",
        "io",
        "manifest",
        "merge",
        "metadata",
        "path_api",
        "path_capture",
        "path_export",
        "path_merge",
        "path_mutation",
        "path_query",
        "path_restore",
        "path_retention",
        "path_transfer",
        "query",
        "reports",
        "restore",
        "retention",
        "selected_capture",
        "selected_mutation",
        "selected_retention",
        "slot",
        "slot_capture",
        "slot_copy",
        "slot_export",
        "slot_import",
        "slot_mutation",
        "slot_selector",
        "slot_store",
        "target_path",
        "validation",
    ] {
        assert!(
            session_root.join(relative).join("mod.rs").exists(),
            "expected Runtime 05 session owner subtree `{relative}` to stay folder-backed"
        );
    }

    for retired_flat_owner in [
        "capture_retention.rs",
        "construction.rs",
        "facade.rs",
        "io.rs",
        "manifest.rs",
        "merge.rs",
        "metadata.rs",
        "path_api.rs",
        "path_capture.rs",
        "path_export.rs",
        "path_merge.rs",
        "path_mutation.rs",
        "path_query.rs",
        "path_restore.rs",
        "path_retention.rs",
        "path_transfer.rs",
        "query.rs",
        "reports.rs",
        "restore.rs",
        "retention.rs",
        "selected_capture.rs",
        "selected_mutation.rs",
        "selected_retention.rs",
        "slot.rs",
        "slot_capture.rs",
        "slot_copy.rs",
        "slot_export.rs",
        "slot_import.rs",
        "slot_mutation.rs",
        "slot_selector.rs",
        "slot_store.rs",
        "target_path.rs",
        "validation.rs",
    ] {
        assert!(
            !session_root.join(retired_flat_owner).exists(),
            "retired flat Runtime 05 session owner `{retired_flat_owner}` should not be restored"
        );
    }
}
