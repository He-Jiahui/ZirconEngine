#[test]
fn runtime_05_dynamic_scene_patch_preview_api_stays_read_only() {
    let patch_source = include_str!("../../scene/dynamic_scene/patch.rs");
    let dynamic_scene_mod_source = include_str!("../../scene/dynamic_scene/mod.rs");
    let scene_mod_source = include_str!("../../scene/dynamic_scene/scene/mod.rs");
    let spawn_source = include_str!("../../scene/dynamic_scene/scene/spawn.rs");
    let behavior_source = include_str!("../../scene/tests/dynamic_scene.rs");
    let capture_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/capture.rs");
    let persistence_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/persistence.rs");
    let retention_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/retention.rs");
    let mutation_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/mutation.rs");
    let selection_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/selection.rs");
    let merge_behavior_source = include_str!("../../scene/tests/dynamic_scene_session/merge.rs");
    let load_behavior_source = include_str!("../../scene/tests/dynamic_scene_session/load.rs");
    let queries_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/queries.rs");
    let path_management_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_session/path_management.rs");
    let asset_reload_behavior_source =
        include_str!("../../scene/tests/dynamic_scene_asset_reload.rs");
    let runtime_05_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let dynamic_scene_doc = include_str!("../../../../docs/zircon_runtime/scene/dynamic_scene.md");

    for public_export in [
        "ScenePatch",
        "ScenePatchPreviewComponentType",
        "ScenePatchPreviewEntityRemap",
        "ScenePatchPreviewReport",
        "ScenePatchPreviewResource",
    ] {
        assert!(
            dynamic_scene_mod_source.contains(public_export),
            "Runtime 05 patch preview export `{public_export}` must stay on the public dynamic-scene facade"
        );
    }
    assert!(
        patch_source.contains("pub struct ScenePatchPreviewReport")
            && patch_source.contains("pub struct ScenePatchPreviewEntityRemap")
            && patch_source.contains("pub source_entity: EntityId")
            && patch_source.contains("pub target_entity: EntityId")
            && patch_source.contains("pub entity_remaps: Vec<ScenePatchPreviewEntityRemap>")
            && patch_source.contains("pub fn has_entity_remaps(")
            && patch_source.contains("pub existing_component_type_count: usize")
            && patch_source.contains("pub new_component_type_count: usize")
            && patch_source.contains("pub struct ScenePatchPreviewComponentType")
            && patch_source.contains("pub component_types: Vec<ScenePatchPreviewComponentType>")
            && patch_source.contains("pub already_registered: bool")
            && patch_source.contains("pub struct ScenePatchPreviewResource")
            && patch_source.contains("pub resources: Vec<ScenePatchPreviewResource>")
            && patch_source.contains("pub already_present: bool")
            && patch_source.contains("pub can_create_on_apply: bool")
            && patch_source.contains("pub field_count: usize")
            && patch_source.contains("pub fn has_new_component_types(")
            && patch_source.contains("pub fn new_component_types(")
            && patch_source.contains("pub fn resources_requiring_creation(")
            && patch_source.contains("pub fn preview_apply(")
            && patch_source.contains("self.scene.preview_spawn_into(world)"),
        "ScenePatch must keep its preview API as a read-only DynamicScene facade call"
    );
    assert!(
        scene_mod_source.contains("pub fn preview_spawn_into(")
            && scene_mod_source.contains("spawn::preview_scene_spawn_into(self, world)"),
        "DynamicScene must keep preview_spawn_into routed to the read-only spawn preview helper"
    );

    let preview_body = spawn_source
        .split("pub(super) fn preview_scene_spawn_into")
        .nth(1)
        .expect("preview_scene_spawn_into should stay in scene/spawn.rs")
        .split("fn install_component_type_descriptors")
        .next()
        .expect("install_component_type_descriptors should stay after preview helper");
    for forbidden_call in [
        "install_component_type_descriptors(",
        "insert_entity_records(",
        "apply_components(",
        "apply_resources(",
    ] {
        assert!(
            !preview_body.contains(forbidden_call),
            "preview_scene_spawn_into must not call mutating apply helper `{forbidden_call}`"
        );
    }
    for required_anchor in [
        "scene.ensure_supported()?",
        "ensure_component_type_descriptors_are_compatible(scene, world)?",
        "build_entity_remap(scene, world)?",
        "validate_remapped_parents(scene, world, &remap)?",
        "validate_components_are_previewable(scene, world, &remap)?",
        "preview_resources(scene, world, &remap)?",
        "ScenePatchPreviewReport",
        "ScenePatchPreviewEntityRemap",
        "component_type_count",
        "existing_component_type_count",
        "new_component_type_count",
        "ScenePatchPreviewComponentType",
        "already_registered",
        "component_types",
        "component_instance_count",
        "resources",
        "preserved_entity_count",
        "remapped_entity_count",
        "entity_remaps",
        "source_entity",
        "target_entity",
    ] {
        assert!(
            preview_body.contains(required_anchor),
            "preview_scene_spawn_into should keep required read-only planning anchor `{required_anchor}`"
        );
    }
    for preflight_anchor in [
        "fn validate_components_are_previewable(",
        "fn validate_component_is_previewable(",
        "fn preview_resources(",
        "fn preview_resource(",
        "runtime_registration(&component.type_path)?",
        "runtime_registration(&resource.type_path)?",
        "ReflectError::NoComponentAdapter",
        "ReflectError::NoResourceAdapter",
        "ReflectError::MissingResource",
        "reflected_fields_to_json_object(&component.fields, remap)?",
        "remap_reflected_value(&field.value, remap)?",
        "component_type_count.saturating_sub(existing_component_type_count)",
        "component_type_descriptor(&descriptor.type_id)",
        "plugin_id: descriptor.plugin_id.clone()",
        "display_name: descriptor.display_name.clone()",
        "let already_present = adapter.contains(world)",
        "let can_create_on_apply = adapter.ensure.is_some()",
        "field_count: resource.fields.len()",
    ] {
        assert!(
            spawn_source.contains(preflight_anchor),
            "preview preflight should keep anchor `{preflight_anchor}`"
        );
    }

    for documented_source in [runtime_05_plan, runtime_index] {
        for anchor in [
            "Runtime 05 dynamic scene patch preview API",
            "dynamic_scene_patch_preview_api_static_passed_cargo_timeout_no_result_tests_deferred",
            "Runtime 05 dynamic scene patch preview status guard",
            "dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending",
            "ScenePatchPreviewReport",
            "ScenePatch::preview_apply",
            "DynamicScene::preview_spawn_into",
            "scene_patch_preview_reports_remaps_without_mutating_target_world",
            "Runtime 05 dynamic scene patch preview resource preflight details status guard",
            "dynamic_scene_patch_preview_resource_preflight_details_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_resource_preflight_details_static_passed_cargo_deferred_tests_deferred",
            "ScenePatchPreviewResource",
            "resources_requiring_creation()",
            "already_present",
            "can_create_on_apply",
            "Runtime 05 dynamic scene patch preview resource ensure creation status guard",
            "dynamic_scene_patch_preview_resource_ensure_creation_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_resource_ensure_creation_static_passed_cargo_deferred_tests_deferred",
            "register_frame_counter_resource_with_ensure",
            "frame_counter_adapter_with_ensure",
            "frame_counter_ensure",
            "preview_with_ensure.resources[0].can_create_on_apply",
            "Runtime 05 dynamic scene patch preview component type install details status guard",
            "dynamic_scene_patch_preview_component_type_install_details_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_component_type_install_details_static_passed_cargo_deferred_tests_deferred",
            "ScenePatchPreviewComponentType",
            "component_types",
            "already_registered",
            "Runtime 05 dynamic scene patch preview component type install counts status guard",
            "dynamic_scene_patch_preview_component_type_install_counts_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_component_type_install_counts_static_passed_cargo_deferred_tests_deferred",
            "existing_component_type_count",
            "new_component_type_count",
            "has_new_component_types()",
            "Runtime 05 dynamic scene patch preview reflection preflight status guard",
            "dynamic_scene_patch_preview_reflection_preflight_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_reflection_preflight_static_passed_cargo_deferred_tests_deferred",
            "validate_components_are_previewable",
            "ReflectError::MissingResource",
            "Runtime 05 dynamic scene patch preview component workload status guard",
            "dynamic_scene_patch_preview_component_workload_status_guard_static_passed_cargo_pending",
            "dynamic_scene_patch_preview_component_workload_static_passed_cargo_deferred_tests_deferred",
            "component_instance_count",
            "Runtime 05 dynamic scene patch preview remap status guard",
            "dynamic_scene_patch_preview_remap_status_guard_static_passed_cargo_pending",
            "ScenePatchPreviewEntityRemap",
            "entity_remaps",
            "has_entity_remaps()",
        ] {
            assert!(
                documented_source.contains(anchor),
                "Runtime 05 patch preview status/doc source should keep anchor `{anchor}`"
            );
        }
    }
    for anchor in [
        "dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending",
        "Runtime 05 dynamic scene patch preview status guard",
        "dynamic_scene_patch_preview_remap_details_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_component_workload_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_reflection_preflight_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_component_type_install_counts_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_component_type_install_details_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_resource_preflight_details_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_resource_preflight_details_status_guard_static_passed_cargo_pending",
        "Runtime 05 dynamic scene patch preview resource preflight details status guard",
        "dynamic_scene_patch_preview_resource_ensure_creation_static_passed_cargo_deferred_tests_deferred",
        "dynamic_scene_patch_preview_resource_ensure_creation_status_guard_static_passed_cargo_pending",
        "Runtime 05 dynamic scene patch preview resource ensure creation status guard",
        "ScenePatchPreviewComponentType",
        "ScenePatchPreviewResource",
        "ScenePatchPreviewEntityRemap",
        "component_types",
        "already_registered",
        "already_present",
        "can_create_on_apply",
        "new_component_type_count",
        "resources_requiring_creation()",
        "component_instance_count",
    ] {
        assert!(
            runtime_05_plan.contains(anchor),
            "Runtime 05 patch preview subplan should keep localized anchor `{anchor}`"
        );
    }
    for anchor in [
        "Dynamic scene patch preview API",
        "ScenePatchPreviewReport",
        "ScenePatchPreviewEntityRemap",
        "ScenePatchPreviewComponentType",
        "ScenePatchPreviewResource",
        "already_registered",
        "already_present",
        "can_create_on_apply",
        "new_component_type_count",
        "resource preflight details",
        "ensure-backed resource",
        "component type install preview",
        "component_instance_count",
        "reflection schema preflight",
        "ScenePatch::preview_apply",
        "DynamicScene::preview_spawn_into",
        "scene_patch_preview_reports_remaps_without_mutating_target_world",
    ] {
        assert!(
            dynamic_scene_doc.contains(anchor),
            "dynamic-scene module docs should keep behavior anchor `{anchor}`"
        );
    }

    assert!(
        behavior_source.contains("scene_patch_preview_reports_remaps_without_mutating_target_world")
            && behavior_source.contains("scene_patch_applies_reflected_resources")
            && behavior_source.contains(".preview_apply(&target)")
            && behavior_source.contains("preview.resources[0].type_path")
            && behavior_source.contains("preview.resources_requiring_creation()")
            && behavior_source.contains("register_frame_counter_resource_with_ensure")
            && behavior_source.contains("frame_counter_adapter_with_ensure")
            && behavior_source.contains("frame_counter_ensure")
            && behavior_source.contains("preview_with_ensure.resources[0].can_create_on_apply")
            && behavior_source.contains("target_with_ensure.get_resource::<FrameCounter>().is_none()")
            && behavior_source.contains("preview.entity_remaps[0].source_entity")
            && behavior_source.contains("preview.entity_remaps[0].target_entity")
            && behavior_source.contains("preview.entity_remaps[1].source_entity")
            && behavior_source.contains("preview.entity_remaps[1].target_entity")
            && behavior_source.contains("preview.component_instance_count")
            && behavior_source.contains("preview.has_entity_remaps()")
            && behavior_source.contains("preview.new_component_type_count")
            && behavior_source.contains("preview.has_new_component_types()")
            && behavior_source.contains("component_types[0].type_id")
            && behavior_source.contains("new_component_types()")
            && behavior_source.contains("target_before")
            && behavior_source.contains("assert!(!target.contains_entity(child));"),
        "focused behavior anchor should keep remap diagnostics and target-world immutability checks"
    );

    for anchor in [
        "runtime_session_archive_world_capture_commit_matches_preview_generated_slot",
        "preview_capture_world_slot(\" manual \", &source, metadata.clone())",
        "capture_world_slot(\" manual \", &source, metadata)",
        "committed_summary.metadata, preview.metadata",
        "committed_summary.entity_count, preview.entity_count",
        "committed_summary.resource_count, preview.resource_count",
    ] {
        assert!(
            capture_behavior_source.contains(anchor),
            "Runtime 05 capture preview/commit parity behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_level_capture_preview_preserves_from_level_semantics",
        "RuntimeSessionSlot::from_level(\" level-preview \", &level)",
        "preview_capture_level_slot(\" level-preview \", &level)",
        "preview.metadata, expected_slot.metadata",
        "preview.entity_count, expected_slot.scene.entities.len()",
        "preview.resource_count, expected_slot.scene.resources.len()",
    ] {
        assert!(
            capture_behavior_source.contains(anchor),
            "Runtime 05 level capture preview/from_level behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_capture_retention_reuses_shared_preview_report_projection",
        "preview_capture_world_slot(\" manual-mid \", &captured_world, metadata.clone())",
        "preview_capture_world_slot_with_tag_retention(",
        "retention_preview.capture, capture_preview",
        "retention_preview.prune.removed_slot_ids",
        "retention_preview.manifest.slot_ids().collect::<Vec<_>>()",
        "archive.slot_ids().collect::<Vec<_>>()",
    ] {
        assert!(
            capture_behavior_source.contains(anchor),
            "Runtime 05 capture-retention shared preview behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_selected_capture_targets_resolved_slot_and_preserves_metadata",
        "preview_capture_world_selected_slot(",
        "capture_world_selected_slot(",
        "capture_world_selected_slot_preserving_metadata(",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(\" manual \")",
        "RuntimeSessionSlotSelector::oldest_updated_with_tag(\"manual\")",
        "preserved.metadata.updated_at_unix_millis, Some(10)",
        "runtime_session_archive_selected_capture_to_path_previews_and_prunes_atomically",
        "preview_capture_world_selected_slot_with_tag_retention_to_path(",
        "capture_world_selected_slot_with_tag_retention_to_path_atomically(",
        "preview.prune.removed_slot_ids",
        "archive payload should remain after preview",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            capture_behavior_source.contains(anchor),
            "Runtime 05 selected capture behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_preview_save_to_path_reports_targets_without_writing_files",
        "preview_save_to_path(&missing_path)",
        "missing_target.will_replace_target",
        "missing_target.statistics.slot_count",
        "preview_save_to_path(&existing_path)",
        "existing_target.will_replace_target",
        "fs::read_to_string(&existing_path)",
        "preview_save_to_path(&directory_target)",
        "RuntimeSessionArchiveError::Io(error)",
        "temporary_archive_leftovers",
    ] {
        assert!(
            persistence_behavior_source.contains(anchor),
            "Runtime 05 full-save preview behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_preview_save_to_path_rejects_parent_file_without_writes",
        "preview_save_to_path(&target_path)",
        "parent_file_target",
        "fs::read_to_string(&parent_file)",
        "parent_file.is_file()",
        "!target_path.exists()",
    ] {
        assert!(
            persistence_behavior_source.contains(anchor),
            "Runtime 05 target parent-file preview behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_preview_capture_retention_prunes_clone_without_mutating_archive",
        "preview_capture_world_slot_with_tag_retention(",
        "preview.prune.removed_slot_ids",
        "preview.manifest.slot_ids().collect::<Vec<_>>()",
        "!archive.contains_slot(\"autosave-new\")",
        "runtime_session_archive_capture_retention_protects_captured_slot_before_pruning",
        "capture_world_slot_with_retention(",
        "RuntimeSessionArchiveRetentionPolicy::keep_latest(0)",
        "report.prune.removed_slot_ids",
        "archive.slot_ids().collect::<Vec<_>>()",
    ] {
        assert!(
            retention_behavior_source.contains(anchor),
            "Runtime 05 capture-retention transaction behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_selected_retention_protects_latest_tagged_slot",
        "preview_prune_slots_with_selected_protection(",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(\" manual \")",
        "prune_slots_with_selected_protection(",
        "runtime_session_archive_tag_selected_retention_ignores_protection_outside_bucket",
        "preview_prune_slots_with_tag_and_selected_protection(",
        "RuntimeSessionSlotSelector::slot_id(\" manual-protected \")",
        "prune_slots_with_tag_and_selected_protection(",
        "runtime_session_archive_path_selected_retention_preview_does_not_write_archive",
        "preview_prune_slots_with_selected_protection_from_path(",
        "RuntimeSessionSlotSelector::oldest_updated_with_tag(\" manual \")",
        "temporary_archive_leftovers",
    ] {
        assert!(
            retention_behavior_source.contains(anchor),
            "Runtime 05 selected retention behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_named_mutation_commits_preserve_preview_boundaries",
        "preview_rename_slot(\"manual\", \" manual-renamed \")",
        "rename_slot(\"manual\", \" manual-renamed \")",
        "RuntimeSessionArchiveError::DuplicateSlotId",
        "runtime_session_archive_selected_mutations_resolve_targets_before_committing",
        "preview_rename_selected_slot(",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(\" manual \")",
        "preview_remove_selected_slot(",
        "remove_selected_slot(",
        "runtime_session_archive_selected_path_mutations_preview_and_commit_atomically",
        "preview_rename_selected_slot_from_path(",
        "rename_selected_slot_at_path_atomically(",
        "preview_remove_selected_slot_from_path(",
        "remove_selected_slot_at_path_atomically(",
        "temporary_archive_leftovers",
    ] {
        assert!(
            mutation_behavior_source.contains(anchor),
            "Runtime 05 mutation behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_merge_preview_and_keep_existing_commit_are_side_effect_free",
        "preview_merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)",
        "merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)",
        "preview.inserted_slot_ids",
        "incoming.slot_ids().collect::<Vec<_>>()",
        "runtime_session_archive_path_merge_preview_commit_and_same_path_guard_are_atomic",
        "preview_merge_archive_at_path(",
        "preview_merge_archive_from_path_at_path(",
        "merge_archive_from_path_at_path_atomically(",
        "RuntimeSessionArchiveMergePolicy::ReplaceExisting",
        "assert_same_path_merge_rejected",
        "target archive path must differ from source archive path",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            merge_behavior_source.contains(anchor),
            "Runtime 05 merge behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_restores_slot_from_path_to_empty_world",
        "restore_slot_from_path_to_empty_world(&path, \"manual\")",
        "runtime_session_archive_restores_slot_from_path_into_level_and_applies_metadata",
        "restore_slot_from_path_into_level(&path, \"level\", &level)",
        "stale_entity",
        "runtime_session_archive_applies_slot_from_path_to_live_world_and_level",
        "apply_slot_from_path_to_world(&path, \"prefab\", &mut world)",
        "apply_slot_from_path_to_level(&path, \"prefab\", &level)",
        "mapped_world_entity",
        "mapped_level_entity",
        "runtime_session_archive_path_load_helpers_report_missing_slot",
        "RuntimeSessionArchiveError::MissingSlot",
    ] {
        assert!(
            load_behavior_source.contains(anchor),
            "Runtime 05 load behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_loads_statistics_from_path",
        "statistics_from_path(&path)",
        "runtime_session_archive_reads_slot_summaries_directly_from_path",
        "slot_ids_from_path(&path)",
        "contains_slot_from_path(&path, \"manual\")",
        "slot_summary_from_path(&path, \"manual\")",
        "runtime_session_archive_diffs_slot_from_path_without_mutating_target",
        "diff_slot_from_path_with_world(&path, \"manual\", &target)",
        "diff_slot_from_path_with_level(&path, \"manual\", &level)",
        "runtime_session_archive_previews_path_retention_without_saving",
        "preview_prune_slots_from_path(",
        "preview_prune_slots_with_tag_from_path(",
        "runtime_session_archive_selects_updated_slots_directly_from_path",
        "latest_updated_slot_id_from_path(&path)",
        "oldest_updated_slot_id_from_path(&path)",
        "latest_updated_slot_id_with_tag_from_path(&path, \" manual \")",
        "oldest_updated_slot_id_with_tag_from_path(&path, \"manual\")",
        "runtime_session_archive_filters_manifest_summaries_directly_from_path",
        "slots_with_tag_from_path(&path, \" manual \")",
        "slots_matching_display_name_from_path(&path, \"Two\")",
        "archive payload should remain readable after query",
        "archive payload should remain readable after preview",
        "archive payload should remain readable after selection",
        "archive payload should remain readable after filter",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            queries_behavior_source.contains(anchor),
            "Runtime 05 query behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_renames_slot_at_path_atomically",
        "rename_slot_at_path_atomically(&path, \"manual-old\", \" manual-new \")",
        "runtime_session_archive_updates_slot_metadata_at_path_atomically",
        "update_slot_metadata_at_path_atomically(",
        "runtime_session_archive_touches_slot_at_path_atomically",
        "touch_slot_at_path_atomically(&path, \"manual\", 90)",
        "runtime_session_archive_removes_slot_at_path_atomically",
        "remove_slot_at_path_atomically(&path, \"autosave\")",
        "runtime_session_archive_previews_slot_mutations_without_mutating_archive",
        "preview_rename_slot(\"manual\", \" manual-renamed \")",
        "RuntimeSessionArchiveError::DuplicateSlotId",
        "runtime_session_archive_previews_slot_mutations_from_path_without_mutating_archive",
        "preview_update_slot_metadata_from_path(",
        "preview_remove_slot_from_path(&path, \"autosave\")",
        "runtime_session_archive_copies_slot_at_path_atomically",
        "copy_slot_with_metadata_at_path_atomically(",
        "runtime_session_archive_previews_slot_copy_without_mutating_archive",
        "preview_copy_slot_with_metadata(",
        "runtime_session_archive_previews_slot_copy_from_path_without_mutating_archive",
        "preview_copy_slot_with_metadata_from_path(",
        "runtime_session_archive_imports_single_slot_at_path_atomically",
        "import_slot_from_archive_with_metadata_at_path_atomically(",
        "runtime_session_archive_imports_single_slot_from_path_at_path_atomically",
        "import_slot_from_archive_path_with_metadata_at_path_atomically(",
        "runtime_session_archive_previews_single_slot_import_without_mutating_archives",
        "preview_import_slot_from_archive_with_metadata(",
        "runtime_session_archive_previews_single_slot_import_from_path_without_mutating_archives",
        "preview_import_slot_from_archive_path_with_metadata_at_path(",
        "runtime_session_archive_saves_single_slot_archive_from_path_atomically",
        "save_single_slot_archive_from_path_atomically(",
        "runtime_session_archive_saves_single_slot_archive_from_memory_atomically",
        "save_single_slot_archive_to_path_atomically(",
        "runtime_session_archive_merges_archive_at_path_atomically",
        "merge_archive_at_path_atomically(",
        "runtime_session_archive_merges_archive_from_path_at_path_atomically",
        "merge_archive_from_path_at_path_atomically(",
        "runtime_session_archive_previews_merge_without_mutating_archives",
        "preview_merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)",
        "runtime_session_archive_previews_merge_from_path_without_mutating_archives",
        "preview_merge_archive_from_path_at_path(",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            path_management_behavior_source.contains(anchor),
            "Runtime 05 path-management behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "dynamic_scene_asset_reload_supersedes_older_pending_scene_revision",
        "DynamicSceneAssetReloadQueue::new(fixture.project.clone(), events)",
        "fixture.register_ready_revision(\"scene-v1\")",
        "fixture.register_ready_revision(\"scene-v2\")",
        "drain_until_events(&mut queue, &scheduler, 2)",
        "drain.superseded_pending[0].event().revision(), 1",
        "pending.pending[0].event().revision(), 2",
        "ready.ready[0].event().revision(), 2",
        "ready.spawn_ready_into(&mut world)",
        "world.node_records().len(), 2",
        "dynamic_scene_asset_reload_skips_removed_and_reload_failed_events",
        "DynamicSceneAssetReloadSkipReason::ReloadFailed",
        "DynamicSceneAssetReloadSkipReason::Removed",
        "dynamic_scene_asset_reload_tick_into_applies_ready_payload_to_world",
        "queue.tick_into(&scheduler, &mut world)",
        "frame.apply.applied[0].event().revision(), 1",
        "dynamic_scene_asset_reload_tick_into_level_applies_ready_payload_to_level_world",
        "queue.tick_into_level(&scheduler, &level)",
        "level.with_world(|world| world.node_records().len()), 2",
        "dynamic_scene_asset_reload_renamed_scene_event_schedules_new_project_uri",
        "AssetEventKind::Renamed",
        ".rename(&fixture.uri, renamed_uri.clone())",
        "pending.pending[0].event().previous_locator()",
        "frame.apply.applied[0].event().locator(), Some(&renamed_uri)",
        "fn drain_until_events(",
        "fn wait_for_pending(",
        "struct SceneReloadFixture",
        "unique_temp_project_root(label)",
        "create_test_project(&root)",
    ] {
        assert!(
            asset_reload_behavior_source.contains(anchor),
            "Runtime 05 asset reload behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_selector_resolves_in_memory_slots",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(",
        "latest_manual.selected_slot_id, \"manual-new\"",
        "RuntimeSessionArchiveError::MissingSlot",
        "runtime_session_archive_selected_retention_protects_resolved_slot",
        "preview_prune_slots_with_tag_and_selected_protection(",
        "prune_slots_with_tag_and_selected_protection(",
        "runtime_session_archive_selected_path_query_and_remove_are_atomic",
        "RuntimeSessionArchive::select_slot_from_path(",
        "RuntimeSessionArchive::remove_selected_slot_at_path_atomically(",
        "runtime_session_archive_selected_metadata_update_targets_resolved_slot",
        "update_selected_slot_metadata(",
        "runtime_session_archive_selected_transfer_helpers_use_resolved_slots",
        "copy_selected_slot(",
        "selected_single_slot_archive(",
        "import_selected_slot_from_archive_with_metadata(",
        "runtime_session_archive_selected_single_slot_export_to_path_is_atomic",
        "save_selected_single_slot_archive_to_path_atomically(",
        "runtime_session_archive_selected_restore_apply_and_diff_use_resolved_slots",
        "restore_selected_slot_to_empty_world(",
        "restore_selected_slot_into_level(",
        "apply_selected_slot_to_level(",
        "runtime_session_archive_selected_path_restore_apply_and_diff_use_resolved_slots",
        "restore_selected_slot_from_path_to_empty_world(",
        "diff_selected_slot_from_path_with_level(",
        "apply_selected_slot_from_path_to_level(",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            selection_behavior_source.contains(anchor),
            "Runtime 05 selection behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_world_capture_preview_commit_parity_rustfmt_passed_cargo_deferred_tests_deferred",
        "world capture commit delegates to preview-generated slot",
        "runtime_session_archive_world_capture_commit_matches_preview_generated_slot",
        "runtime_session_archive_level_capture_preview_from_level_semantics_rustfmt_passed_cargo_deferred_tests_deferred",
        "level capture preview preserves RuntimeSessionSlot::from_level semantics",
        "runtime_session_archive_level_capture_preview_preserves_from_level_semantics",
        "runtime_session_archive_capture_retention_shared_preview_projection_rustfmt_passed_cargo_deferred_tests_deferred",
        "capture-retention keeps shared preview report projection",
        "runtime_session_archive_capture_retention_reuses_shared_preview_report_projection",
        "runtime_session_archive_full_save_preview_no_write_targets_rustfmt_passed_cargo_deferred_tests_deferred",
        "missing-target、existing-file、non-file-target 与 no-write/no-temp 行为锚点",
        "runtime_session_archive_preview_save_to_path_reports_targets_without_writing_files",
        "runtime_session_archive_target_parent_file_preview_rustfmt_passed_cargo_deferred_tests_deferred",
        "parent-file target preview 行为锚点",
        "target parent-file preflight parity",
        "runtime_session_archive_preview_save_to_path_rejects_parent_file_without_writes",
        "runtime_session_archive_capture_retention_transaction_behavior_rustfmt_passed_cargo_deferred_tests_deferred",
        "capture-retention protects captured slot",
        "preview does not mutate archive/path",
        "tag retention prunes only tagged bucket",
        "runtime_session_archive_preview_capture_retention_prunes_clone_without_mutating_archive",
        "runtime_session_archive_capture_retention_protects_captured_slot_before_pruning",
        "runtime_session_archive_selected_retention_behavior_rustfmt_passed_cargo_deferred_tests_deferred",
        "selected retention protects latest/tagged slot",
        "tag selected protection outside tag bucket is harmless",
        "path selected retention preview does not write archive",
        "runtime_session_archive_selected_retention_protects_latest_tagged_slot",
        "runtime_session_archive_tag_selected_retention_ignores_protection_outside_bucket",
        "runtime_session_archive_path_selected_retention_preview_does_not_write_archive",
        "runtime_session_archive_mutation_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_05_dynamic_scene_mutation_behavior_anchors_rustfmt_passed_focused_cargo_timeout_no_result_scene_gate_pending",
        "runtime_session_archive_named_mutation_commits_preserve_preview_boundaries",
        "runtime_session_archive_selected_mutations_resolve_targets_before_committing",
        "runtime_session_archive_selected_path_mutations_preview_and_commit_atomically",
        "selected mutation selector resolution",
        "selected path-level preview no-write behavior",
        "runtime_session_archive_selection_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_05_dynamic_scene_slot_selector_behavior_anchors_rustfmt_passed_cargo_deferred_scene_gate_pending",
        "runtime_05_dynamic_scene_selected_transfer_behavior_anchors_rustfmt_passed_cargo_deferred_scene_gate_pending",
        "runtime_05_dynamic_scene_selected_restore_apply_diff_behavior_anchors_rustfmt_passed_cargo_deferred_scene_gate_pending",
        "runtime_session_archive_selector_resolves_in_memory_slots",
        "runtime_session_archive_selected_transfer_helpers_use_resolved_slots",
        "runtime_session_archive_selected_restore_apply_and_diff_use_resolved_slots",
        "runtime_session_archive_selected_path_restore_apply_and_diff_use_resolved_slots",
        "runtime_session_archive_selected_capture_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_05_dynamic_scene_selected_capture_behavior_anchors_rustfmt_passed_cargo_deferred_scene_gate_pending",
        "selected capture behavior anchors",
        "runtime_session_archive_selected_capture_targets_resolved_slot_and_preserves_metadata",
        "runtime_session_archive_selected_capture_to_path_previews_and_prunes_atomically",
        "runtime_session_archive_merge_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_05_dynamic_scene_merge_behavior_anchors_rustfmt_passed_cargo_deferred_scene_gate_pending",
        "dynamic scene merge behavior anchors",
        "runtime_session_archive_merge_preview_and_keep_existing_commit_are_side_effect_free",
        "runtime_session_archive_path_merge_preview_commit_and_same_path_guard_are_atomic",
        "runtime_session_archive_load_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_session_archive_load_from_path_code_check_passed_tests_deferred",
        "runtime_session_archive_restores_slot_from_path_to_empty_world",
        "runtime_session_archive_restores_slot_from_path_into_level_and_applies_metadata",
        "runtime_session_archive_applies_slot_from_path_to_live_world_and_level",
        "runtime_session_archive_path_load_helpers_report_missing_slot",
        "runtime_session_archive_query_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_session_archive_path_query_code_check_passed_tests_deferred",
        "runtime_session_archive_path_slot_summary_code_check_passed_tests_deferred",
        "runtime_session_archive_path_manifest_filter_code_static_passed_cargo_blocked_external_postprocess_drift_tests_deferred",
        "runtime_session_archive_path_slot_selection_code_static_passed_cargo_blocked_external_postprocess_drift_tests_deferred",
        "runtime_session_archive_path_retention_preview_code_static_passed_cargo_blocked_external_postprocess_drift_tests_deferred",
        "runtime_session_archive_loads_statistics_from_path",
        "runtime_session_archive_reads_slot_summaries_directly_from_path",
        "runtime_session_archive_diffs_slot_from_path_without_mutating_target",
        "runtime_session_archive_previews_path_retention_without_saving",
        "runtime_session_archive_selects_updated_slots_directly_from_path",
        "runtime_session_archive_filters_manifest_summaries_directly_from_path",
        "runtime_session_archive_path_management_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_session_archive_path_management_code_check_passed_tests_deferred",
        "runtime_session_archive_slot_mutation_preview_code_check_passed_tests_deferred",
        "runtime_session_archive_copy_preview_code_check_passed_tests_deferred",
        "runtime_session_archive_single_slot_import_preview_code_check_passed_tests_deferred",
        "runtime_session_archive_source_path_single_slot_import_code_check_passed_tests_deferred",
        "runtime_session_archive_loaded_single_slot_save_code_check_passed_tests_deferred",
        "runtime_session_archive_single_slot_export_code_check_passed_tests_deferred",
        "runtime_session_archive_merge_preview_code_check_passed_tests_deferred",
        "runtime_session_archive_source_path_merge_code_check_passed_tests_deferred",
        "runtime_session_archive_renames_slot_at_path_atomically",
        "runtime_session_archive_previews_slot_mutations_without_mutating_archive",
        "runtime_session_archive_copies_slot_at_path_atomically",
        "runtime_session_archive_previews_slot_copy_from_path_without_mutating_archive",
        "runtime_session_archive_imports_single_slot_from_path_at_path_atomically",
        "runtime_session_archive_previews_single_slot_import_from_path_without_mutating_archives",
        "runtime_session_archive_saves_single_slot_archive_from_path_atomically",
        "runtime_session_archive_saves_single_slot_archive_from_memory_atomically",
        "runtime_session_archive_merges_archive_from_path_at_path_atomically",
        "runtime_session_archive_previews_merge_from_path_without_mutating_archives",
        "runtime_05_dynamic_scene_asset_reload_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "runtime_05_dynamic_scene_asset_reload_behavior_anchors_rustfmt_passed_cargo_deferred_active_cargo_lane_scene_gate_pending",
        "runtime_05_dynamic_scene_asset_reload_frame_entrypoint_anchors_rustfmt_passed_cargo_deferred_active_cargo_lane_scene_gate_pending",
        "runtime_05_dynamic_scene_asset_reload_renamed_event_anchors_rustfmt_passed_cargo_deferred_active_cargo_lane_scene_gate_pending",
        "dynamic scene asset reload behavior anchors",
        "dynamic scene asset reload frame entrypoint anchors",
        "dynamic scene asset reload renamed event anchors",
        "dynamic_scene_asset_reload_supersedes_older_pending_scene_revision",
        "dynamic_scene_asset_reload_skips_removed_and_reload_failed_events",
        "dynamic_scene_asset_reload_tick_into_applies_ready_payload_to_world",
        "dynamic_scene_asset_reload_tick_into_level_applies_ready_payload_to_level_world",
        "dynamic_scene_asset_reload_renamed_scene_event_schedules_new_project_uri",
    ] {
        assert!(
            runtime_05_plan.contains(anchor),
            "Runtime 05 capture/save preview status should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "Runtime session archive world capture preview/commit parity",
        "runtime_session_archive_world_capture_commit_matches_preview_generated_slot",
        "preview-generated slot",
        "Runtime session archive level capture preview/from_level semantics",
        "runtime_session_archive_level_capture_preview_preserves_from_level_semantics",
        "RuntimeSessionSlot::from_level semantics",
        "Runtime session archive capture-retention shared preview projection",
        "runtime_session_archive_capture_retention_reuses_shared_preview_report_projection",
        "report.capture",
        "Runtime session archive full-save preview target/no-write behavior",
        "runtime_session_archive_preview_save_to_path_reports_targets_without_writing_files",
        "RuntimeSessionArchiveSavePreviewReport",
        "no-write/no-temp",
        "Runtime session archive target parent-file preview behavior",
        "runtime_session_archive_preview_save_to_path_rejects_parent_file_without_writes",
        "parent-file target preview",
        "Runtime session archive capture-retention transaction behavior",
        "runtime_session_archive_preview_capture_retention_prunes_clone_without_mutating_archive",
        "runtime_session_archive_capture_retention_protects_captured_slot_before_pruning",
        "captured slot protection",
        "Runtime session archive selected retention behavior anchors",
        "runtime_session_archive_selected_retention_protects_latest_tagged_slot",
        "selected latest tagged slot protection",
        "runtime_session_archive_tag_selected_retention_ignores_protection_outside_bucket",
        "tag selected protection outside tag bucket behavior",
        "runtime_session_archive_path_selected_retention_preview_does_not_write_archive",
        "path selected retention preview no-write/no-temp behavior",
        "Runtime session archive mutation behavior anchors",
        "runtime_session_archive_named_mutation_commits_preserve_preview_boundaries",
        "runtime_session_archive_selected_mutations_resolve_targets_before_committing",
        "runtime_session_archive_selected_path_mutations_preview_and_commit_atomically",
        "Runtime session archive selection behavior static guard",
        "runtime_session_archive_selector_resolves_in_memory_slots",
        "runtime_session_archive_selected_transfer_helpers_use_resolved_slots",
        "runtime_session_archive_selected_restore_apply_and_diff_use_resolved_slots",
        "runtime_session_archive_selected_path_restore_apply_and_diff_use_resolved_slots",
        "Runtime session archive selected capture behavior static guard",
        "Runtime session archive selected capture behavior anchors",
        "runtime_session_archive_selected_capture_targets_resolved_slot_and_preserves_metadata",
        "runtime_session_archive_selected_capture_to_path_previews_and_prunes_atomically",
        "Runtime session archive merge behavior static guard",
        "Runtime session archive merge behavior anchors",
        "runtime_session_archive_merge_preview_and_keep_existing_commit_are_side_effect_free",
        "runtime_session_archive_path_merge_preview_commit_and_same_path_guard_are_atomic",
        "Runtime session archive load behavior static guard",
        "runtime_session_archive_restores_slot_from_path_to_empty_world",
        "runtime_session_archive_restores_slot_from_path_into_level_and_applies_metadata",
        "runtime_session_archive_applies_slot_from_path_to_live_world_and_level",
        "runtime_session_archive_path_load_helpers_report_missing_slot",
        "Runtime session archive query behavior static guard",
        "runtime_session_archive_loads_statistics_from_path",
        "runtime_session_archive_reads_slot_summaries_directly_from_path",
        "runtime_session_archive_diffs_slot_from_path_without_mutating_target",
        "runtime_session_archive_previews_path_retention_without_saving",
        "runtime_session_archive_selects_updated_slots_directly_from_path",
        "runtime_session_archive_filters_manifest_summaries_directly_from_path",
        "Runtime session archive path-management behavior static guard",
        "runtime_session_archive_renames_slot_at_path_atomically",
        "runtime_session_archive_previews_slot_mutations_without_mutating_archive",
        "runtime_session_archive_copies_slot_at_path_atomically",
        "runtime_session_archive_previews_slot_copy_from_path_without_mutating_archive",
        "runtime_session_archive_imports_single_slot_from_path_at_path_atomically",
        "runtime_session_archive_previews_single_slot_import_from_path_without_mutating_archives",
        "runtime_session_archive_saves_single_slot_archive_from_path_atomically",
        "runtime_session_archive_saves_single_slot_archive_from_memory_atomically",
        "runtime_session_archive_merges_archive_from_path_at_path_atomically",
        "runtime_session_archive_previews_merge_from_path_without_mutating_archives",
        "Dynamic scene asset reload behavior static guard",
        "runtime_05_dynamic_scene_asset_reload_behavior_static_guard_rustfmt_passed_cargo_deferred_tests_deferred",
        "Dynamic scene asset reload behavior anchors",
        "dynamic_scene_asset_reload_supersedes_older_pending_scene_revision",
        "dynamic_scene_asset_reload_skips_removed_and_reload_failed_events",
        "dynamic_scene_asset_reload_tick_into_applies_ready_payload_to_world",
        "dynamic_scene_asset_reload_tick_into_level_applies_ready_payload_to_level_world",
        "dynamic_scene_asset_reload_renamed_scene_event_schedules_new_project_uri",
    ] {
        assert!(
            dynamic_scene_doc.contains(anchor),
            "dynamic-scene module docs should keep capture/save preview semantics anchor `{anchor}`"
        );
    }
}
