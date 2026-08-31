#[test]
fn runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ecs_root = manifest_root.join("src/scene/ecs");

    for owner in [
        "archetype",
        "component",
        "entity",
        "events",
        "messages",
        "observer",
        "resource",
        "resource_store",
    ] {
        assert!(
            ecs_root.join(owner).join("mod.rs").exists(),
            "Runtime 08 ECS data owner `{owner}` should stay folder-backed"
        );
    }

    for retired in [
        "events.rs",
        "messages.rs",
        "component.rs",
        "component_id.rs",
        "component_registry.rs",
        "despawned_entity.rs",
        "entity_location.rs",
        "entity_registry.rs",
        "entity_registry_error.rs",
        "internal_entity.rs",
        "stable_entity_location.rs",
        "resource.rs",
        "resource_id.rs",
        "resource_registry.rs",
        "resource_store.rs",
        "observer.rs",
        "archetype_id.rs",
        "archetype_index.rs",
        "archetype_signature.rs",
    ] {
        assert!(
            !ecs_root.join(retired).exists(),
            "retired flat Runtime 08 ECS owner `{retired}` should not be restored"
        );
    }

    for (owner, children) in [
        (
            "archetype",
            &["id.rs", "index.rs", "record.rs", "signature.rs"][..],
        ),
        ("component", &["id.rs", "marker.rs", "registry.rs"][..]),
        (
            "entity",
            &[
                "despawned.rs",
                "error.rs",
                "internal.rs",
                "location.rs",
                "registry.rs",
                "slot.rs",
                "stable_location.rs",
            ][..],
        ),
        (
            "events",
            &[
                "cursor.rs",
                "id.rs",
                "metrics.rs",
                "queue.rs",
                "store.rs",
                "subscription.rs",
            ][..],
        ),
        (
            "messages",
            &["cursor.rs", "id.rs", "queue.rs", "store.rs"][..],
        ),
        (
            "observer",
            &[
                "callback_registry.rs",
                "callbacks.rs",
                "entry.rs",
                "id.rs",
                "store.rs",
            ][..],
        ),
        ("resource", &["id.rs", "marker.rs", "registry.rs"][..]),
        ("resource_store", &["store.rs", "stored_resource.rs"][..]),
    ] {
        for child in children {
            assert!(
                ecs_root.join(owner).join(child).exists(),
                "Runtime 08 ECS data owner `{owner}` should keep child `{child}`"
            );
        }

        let owner_mod_source =
            std::fs::read_to_string(ecs_root.join(owner).join("mod.rs")).unwrap();
        for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
            assert!(
                !owner_mod_source.contains(forbidden),
                "`{owner}/mod.rs` should remain a structural module/export owner, but found `{forbidden}`"
            );
        }
    }

    let commands_root = ecs_root.join("commands");
    assert!(
        commands_root.join("mod.rs").exists(),
        "Runtime 08 ECS commands owner should stay folder-backed"
    );
    assert!(
        commands_root.join("commands").join("mod.rs").exists(),
        "Runtime 08 ECS commands facade owner should stay folder-backed"
    );
    assert!(
        !commands_root.join("commands.rs").exists(),
        "retired flat Runtime 08 ECS commands facade owner `commands/commands.rs` should not be restored"
    );
    for child in [
        "command.rs",
        "command_queue.rs",
        "commands/entity_commands.rs",
        "commands/facade.rs",
        "commands/param.rs",
    ] {
        assert!(
            commands_root.join(child).exists(),
            "Runtime 08 ECS commands owner should keep child `{child}`"
        );
    }
    for relative in ["mod.rs", "commands/mod.rs"] {
        let mod_source = std::fs::read_to_string(commands_root.join(relative)).unwrap();
        for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
            assert!(
                !mod_source.contains(forbidden),
                "`commands/{relative}` should remain a structural module/export owner, but found `{forbidden}`"
            );
        }
    }

    let component_storage_root = ecs_root.join("storage").join("component_storage");
    let archetype_table_root = ecs_root.join("archetype").join("table");
    assert!(
        component_storage_root.join("mod.rs").exists(),
        "Runtime 08 ECS component storage owner should stay folder-backed"
    );
    assert!(
        !ecs_root
            .join("storage")
            .join("component_storage.rs")
            .exists(),
        "retired flat Runtime 08 ECS storage owner `storage/component_storage.rs` should not be restored"
    );
    for child in [
        "component_results.rs",
        "entry.rs",
        "location.rs",
        "sparse.rs",
        "store.rs",
    ] {
        assert!(
            component_storage_root.join(child).exists(),
            "Runtime 08 ECS component storage owner should keep child `{child}`"
        );
    }
    assert!(
        !component_storage_root.join("table.rs").exists(),
        "dense table values must not regain a second owner under ComponentStorage"
    );
    for child in [
        "mod.rs",
        "column.rs",
        "error.rs",
        "preflighted_row.rs",
        "table.rs",
        "taken_row.rs",
    ] {
        assert!(
            archetype_table_root.join(child).exists(),
            "Runtime 08 ArchetypeTable owner should keep child `{child}`"
        );
    }
    let component_storage_mod_source =
        std::fs::read_to_string(component_storage_root.join("mod.rs")).unwrap();
    for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
        assert!(
            !component_storage_mod_source.contains(forbidden),
            "`storage/component_storage/mod.rs` should remain a structural module/export owner, but found `{forbidden}`"
        );
    }
}

#[test]
fn runtime_08_ecs_change_detection_owner_tree_stays_folder_backed_after_cutover() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ecs_root = manifest_root.join("src/scene/ecs");
    let change_detection_root = ecs_root.join("change_detection");

    assert!(
        change_detection_root.join("mod.rs").exists(),
        "Runtime 08 ECS change detection owner should stay folder-backed"
    );
    assert!(
        !ecs_root.join("change_detection.rs").exists(),
        "retired flat Runtime 08 ECS change detection owner `change_detection.rs` should not be restored"
    );

    for retired in [
        "change_tick.rs",
        "change_tick_window.rs",
        "component_ticks.rs",
        "change_detection_stats.rs",
        "change_detection_wrappers.rs",
    ] {
        assert!(
            !ecs_root.join(retired).exists(),
            "retired flat Runtime 08 ECS change detection child `{retired}` should not be restored"
        );
    }

    for child in [
        "change_tick.rs",
        "change_tick_window.rs",
        "component_ticks.rs",
        "stats.rs",
        "wrappers.rs",
    ] {
        assert!(
            change_detection_root.join(child).exists(),
            "Runtime 08 ECS change detection owner should keep child `{child}`"
        );
    }

    assert_structural_module_owner(
        &change_detection_root.join("mod.rs"),
        "change_detection/mod.rs",
    );
}

#[test]
fn runtime_08_ecs_root_leaf_owners_stay_explicit_after_data_cutover() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ecs_root = manifest_root.join("src/scene/ecs");
    let root_mod_source = std::fs::read_to_string(ecs_root.join("mod.rs")).unwrap();
    let root_leaf_owners: &[(&str, &str, &[&str])] = &[
        (
            "bundle",
            "pub use bundle::Bundle;",
            &[
                "pub trait Bundle: 'static + Send + Sync",
                "fn stage_into<S>(self, staging: &mut S) -> SceneResult<()>",
                "fn stage<T>(&mut self, component: T) -> SceneResult<()>",
                "tuple_bundle!(A, B, C, D, E, F, G, H);",
            ],
        ),
        (
            "bundle_transaction_diagnostics",
            "pub use bundle_transaction_diagnostics::{",
            &[
                "pub struct BundleTransactionDiagnostics",
                "pub staged_value_allocations: u64",
                "pub(crate) fn record_commit",
            ],
        ),
        (
            "removal",
            "pub use removal::{",
            &[
                "pub struct RemovedComponentEvent",
                "pub struct RemovedComponentEvents",
                "pub struct RemovedComponentReader<T>",
                "pub struct RemovedComponentRetention",
                "pub struct RemovedComponentRetentionMetrics",
                "pub fn read<'reader, 'events>(",
            ],
        ),
        (
            "storage_type",
            "pub use storage_type::StorageType;",
            &["pub enum StorageType", "Table,", "SparseSet,"],
        ),
    ];

    for &(owner, public_anchor, source_anchors) in root_leaf_owners {
        let owner_file = format!("{owner}.rs");
        let owner_path = ecs_root.join(&owner_file);

        assert!(
            owner_path.exists(),
            "Runtime 08 ECS root leaf owner `{owner_file}` should stay explicit"
        );
        assert!(
            !ecs_root.join(owner).is_dir(),
            "Runtime 08 ECS root leaf owner `{owner_file}` should not drift into an unplanned subtree"
        );
        assert!(
            root_mod_source.contains(&format!("mod {owner};")),
            "scene/ecs/mod.rs should continue declaring root leaf owner `{owner}`"
        );
        assert!(
            root_mod_source.contains(public_anchor),
            "scene/ecs/mod.rs should continue exporting root leaf owner `{owner}` through `{public_anchor}`"
        );

        let source = std::fs::read_to_string(owner_path).unwrap();
        assert_source_contains(&source, &owner_file, source_anchors);
    }
}

#[test]
fn runtime_08_archetype_table_owner_stays_folder_backed() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let table_root = manifest_root.join("src/scene/ecs/archetype/table");

    assert!(
        table_root.join("mod.rs").exists(),
        "Runtime 08 archetype table owner should stay folder-backed"
    );
    assert!(
        !manifest_root
            .join("src/scene/ecs/archetype/table.rs")
            .exists(),
        "retired flat Runtime 08 archetype table owner should not be restored"
    );
    for child in [
        "column.rs",
        "error.rs",
        "preflighted_row.rs",
        "table.rs",
        "taken_row.rs",
    ] {
        assert!(
            table_root.join(child).exists(),
            "Runtime 08 archetype table owner should keep child `{child}`"
        );
    }
    assert_structural_module_owner(&table_root.join("mod.rs"), "archetype/table/mod.rs");
}

fn assert_structural_module_owner(path: &std::path::Path, label: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    for forbidden in ["fn ", "impl ", "struct ", "enum ", "trait ", "macro_rules!"] {
        assert!(
            !source.contains(forbidden),
            "`{label}` should remain a structural module/export owner, but found `{forbidden}`"
        );
    }
}

fn assert_source_contains(source: &str, label: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "`{label}` should retain Runtime 08 root leaf anchor `{anchor}`"
        );
    }
}
