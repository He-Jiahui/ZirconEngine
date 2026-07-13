---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/plan_status/support/runtime_plan_archives.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/recent_static_guards/parent_routing.rs
  - tools/tests/test_runtime_plan_status_archive_ownership.py
  - tests/acceptance/runtime-plan-status-archive-ownership-sync.md
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/ecs/archetype/index.rs
  - zircon_runtime/src/scene/ecs/events/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_asset_reload.rs
  - zircon_runtime/src/scene/tests/ecs_archetype_index_structure.rs
  - zircon_runtime/src/scene/tests/ecs_dynamic_components_structure.rs
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_observers_messages.rs
  - zircon_runtime/src/scene/tests/ecs_query_combinations.rs
  - zircon_runtime/src/scene/tests/ecs_query_state_structure.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/conflict_graph.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/mod.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/capture.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/load.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/merge.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/mutation.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session/selection.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/architecture_review.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/closeout.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/index_tables.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/recent_static_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/subplan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_01.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_02.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_03.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_04.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_06.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_07.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_08.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_10.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_11.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_12.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_13.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_14.rs
  - zircon_runtime/src/scene/dynamic_scene/session/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/archive.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/construction/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/construction/archive.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/construction/capture/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/construction/capture/world.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/construction/capture/level.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/construction/serialization.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/ordinary/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/ordinary/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/ordinary/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/global/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/global/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/global/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/tag/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/tag/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/world/retention/tag/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/ordinary/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/ordinary/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/ordinary/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/global/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/global/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/global/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/tag/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/tag/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/capture/level/retention/tag/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/store/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/store/push.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/store/upsert.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/store/remove.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/named/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/copy/selected/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/named/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/named/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/selected/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/export/selected/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/named/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/transfer/import/selected/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/rename/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/rename/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/rename/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/touch/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/touch/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/touch/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/mutation/remove.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/merge/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/merge/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/merge/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/global/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/global/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/global/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/tag/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/tag/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/retention/tag/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/basic.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/selection/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/selection/global.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/selection/tag.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/selection/typed.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/query/statistics.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/apply/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/apply/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/apply/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/restore/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/restore/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/diff/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/diff/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/restore/diff/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/validation/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/validation/support.rs
  - zircon_runtime/src/scene/dynamic_scene/session/facade/validation/invariants.rs
  - zircon_runtime/src/scene/dynamic_scene/session/construction/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/construction/archive.rs
  - zircon_runtime/src/scene/dynamic_scene/session/construction/capture.rs
  - zircon_runtime/src/scene/dynamic_scene/session/construction/serialization.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/loaded/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/loaded/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/loaded/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/source_path/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/source_path/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_merge/source_path/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/world/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/world/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/world/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/level/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/level/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/level/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/capture_retention/apply.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge/policy.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge/report.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge/algorithm/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge/algorithm/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/merge/algorithm/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/manifest/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/manifest/archive.rs
  - zircon_runtime/src/scene/dynamic_scene/session/manifest/order.rs
  - zircon_runtime/src/scene/dynamic_scene/session/manifest/summary.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot/capture.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot/restore.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot/diff.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot/summary.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/world/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/world/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/world/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/level/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/level/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/level/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_capture/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/named/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_copy/selected/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_export/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_export/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_export/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_selector/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_selector/selector.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_selector/report.rs
  - zircon_runtime/src/scene/dynamic_scene/session/slot_selector/resolve.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/apply/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/apply/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/apply/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/restore/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/restore/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/diff/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/diff/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/restore/diff/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/query/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/query/manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/query/selection.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/query/statistics.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/query/status.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/named/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/named/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/selected/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/loaded/selected/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/named/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/named/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/selected/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/export/source_path/selected/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/apply/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/apply/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/apply/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/restore/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/restore/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/diff/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/diff/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/restore/diff/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/named/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_api/transfer/copy/selected/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/named/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/basic/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/basic/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/basic/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_transfer/copy/selected/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/named/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/named/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/selected/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/loaded/selected/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/named/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/named/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/named/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/selected/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/selected/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_export/source_path/selected/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_query/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_query/manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_query/selection.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_query/statistics.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_query/status.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/global/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/global/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/global/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/tag/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/tag/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_retention/tag/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/rename/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/rename/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/rename/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/metadata/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/metadata/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/metadata/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/touch/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/touch/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/touch/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/remove/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/remove/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_mutation/remove/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/global/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/global/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/global/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/tag/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/tag/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/tag/preview.rs
  - zircon_runtime/src/scene/dynamic_scene/session/selected_retention/policy.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/apply/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/apply/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/apply/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/restore/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/restore/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/restore/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/diff/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/diff/named.rs
  - zircon_runtime/src/scene/dynamic_scene/session/path_restore/diff/selected.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/archive.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/capture.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/diff.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/mutation.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/restore.rs
  - zircon_runtime/src/scene/dynamic_scene/session/reports/transfer.rs
  - zircon_runtime/src/scene/dynamic_scene/session/target_path/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/session/target_path/source.rs
  - zircon_runtime/src/scene/dynamic_scene/session/target_path/target.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/plugin/core_profiles.rs
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - zircon_editor/src/scene
  - zircon_hub/src/projects/metadata.rs
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_sources.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_support_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
status: completed
last_refined: 2026-07-13
---

# 05 scene/editor 边界收尾

## 现状与证据（2026-06-12 重核）

旧文两项核心"残留"已被解决，本计划工作面相应缩小并转向白名单机器化：

- **空目录已删（矫正）**：`zircon_runtime/src/scene/editor_projection/` 已不存在（`ls` 报 No such file or directory）。全仓 `editor_projection` 残留引用仅 1 处文本：`scene/tests/component_structure.rs`（执行时核验语义：是"断言不存在"的守卫还是过期注释）。原 M1 切片 1 的"删目录"工作已完成。
- **序列化纯净守卫已枚举完备（矫正）**：`scene/tests/authoring_boundary.rs` 已有双 token 禁入清单——`SERIALIZED_AUTHORING_TOKENS`（19 词：selection/selection_anchors/scene_gizmos/gizmo/overlay/camera_override/preview_lighting/viewport_camera/SceneViewportSettings/pane 等）与 `SOURCE_AUTHORING_TOKENS`（25 词：含 SelectionHighlightExtract/GridOverlayExtract/SceneGizmoOverlayExtract 等 extract 类型名）；helper `assert_text_excludes_authoring_tokens` + 负例自检 `authoring_boundary_guard_fails_on_representative_tokens`。消费方覆盖 world project、dynamic scene、asset scene、inspection JSON 与 serialization source 守卫。
- **inspection 守卫已存在（矫正）**：`scene/tests/inspection.rs` 已有 `world_inspection_serialization_excludes_editor_authoring_tokens` 与 `world_inspection_filters_missing_focus_without_storing_authoring_state`。原 M2 切片 2 的"给 inspection 加守卫"已完成。
- **inspection 公共面**：`scene/inspection/mod.rs` 仅导出 `WorldInspectionField` / `WorldInspectionHierarchyRow` / `WorldInspection` 三类型（field/hierarchy/snapshot 三文件），形状中性。
- **剩余真实工作 1——"editor" 命名白名单**：runtime 内 "editor" 命中约 181 文件（执行时重核：Grep `-l editor`，path `zircon_runtime/src`），混杂三类：合法 editor-host 目标语义（`dynamic_api/session.rs` 的 editor 模式、`plugin/core_profiles.rs:10` `pub struct EditorCoreProfile`、native loader 的 editor_host 校验）、测试夹具、待裁决 authoring 残留。无白名单则无法机器化判定新增违规。
- **剩余真实工作 2——"legacy" 命名裁决**："legacy" 403 处/84 文件。实仓抽样（`ui/surface/input/navigation.rs:22-54`）：`legacy` 是 `dispatch_navigation_event` 旧路由回复的本地变量名，承载真实运行语义（route/focus/diagnostics 全从它取值）——属"领域词化的迁移痕迹"，需逐类裁决是改名、文档化还是列债。
- 命名审计文档锚点（2026-06-12 实测存在）：`docs/engine-architecture/non-network-server-naming-m1.md`、`hard-cutover-migration-smells-m1.md`、`runtime-root-surface-m1.md`、`runtime-architecture-review-m0.md`——白名单产出并入这些既有口径，不另起新文件。

## 目标

1. 物理与文本残留清零：`editor_projection` 的最后 1 处文本残留裁决处理。
2. "editor" 与 "legacy" 命名逐类裁决，产出白名单 + 机器化守卫，使违规新增可被测试拒绝。
3. 守卫从"已存在"升级为"覆盖矩阵可证"：三序列化出口 × 双 token 表 × inspection 的覆盖关系成表，token 清单有维护公约。

## 非目标

- 不迁移任何已在 `zircon_editor` 的投影/选中/gizmo 逻辑；不改 inspection 三类型的数据形状（除非裁决发现 authoring 泄漏）。
- 不在本计划处理 ui/surface/input 的输入路径重构（"legacy" 若裁决为迁移债，移交 UI 边界 owner 计划）。
- 渲染骨架内容归 render 计划 01-08。

### 全局硬约束（继承总计划 §4，违反即返工）

- 硬切换不留兼容层；不新增 crate；非网络语义 server 命名是 blocker（白名单审计顺带复核，口径并入 `non-network-server-naming-m1.md`）。

## 执行前检查清单

1. 活动会话对齐：serialization 守卫与 inspection 均在 `20260604-1232` 会话工作区延长线上——执行前确认该会话对应切片已完成或已交接，避免双写同一守卫。
2. worktree 脏文件检查：`git status --porcelain -- zircon_runtime/src/scene/ docs/engine-architecture/`。
3. 事实重核：
   - `ls zircon_runtime/src/scene/ | grep editor_projection`（应无输出）
   - `grep -rn "editor_projection" zircon_runtime/src --include=*.rs`（应仅 component_structure.rs 1 处）
   - `grep -rl "editor" zircon_runtime/src --include=*.rs | wc -l`（editor 命中文件数基线重核）
   - `grep -rn "legacy" zircon_runtime/src/ui/surface/input --include=*.rs | wc -l`
4. 基线记录：`cargo test -p zircon_runtime --lib scene:: --locked` 通过数记入状态节。

## 里程碑

### M1 残留收尾与命名白名单裁决

#### 切片 1.1 editor_projection 文本残留处理

- 目标文件：`zircon_runtime/src/scene/tests/component_structure.rs`（唯一残留处）。
- 改动形态：核验该处语义——若是"目录不得复活"的守卫断言则保留并加注释；若是过期文本则删除该词。无其他代码改动。
- 调用方迁移：无。
- 验收：Grep `editor_projection` 全仓结果与裁决一致（0 处或仅守卫 1 处）。
- DoD：重核命令输出与判词记入状态节。

#### 切片 1.2 "editor" 命中三分类白名单

- 目标文件：`docs/engine-architecture/runtime-root-surface-m1.md` 或 `runtime-architecture-review-m0.md`（并入既有审计口径，执行时与 `20260604-1232` 会话定稿落点，禁止另起新文件）；本计划状态节（清单副本）。
- 改动形态：纯文档 + 裁决。分类规则（已核实锚点）：
  - **白名单（合法 editor-host 目标语义）**：`dynamic_api/session.rs` editor 会话模式、`plugin/core_profiles.rs` `EditorCoreProfile`（:10）/`RuntimeCoreProfile`（:4）双 profile、native loader 的 editor_host 校验行、`builtin` 的 target mode 词汇。
  - **测试夹具**：测试文件内的 editor 字样按文件粒度白名单。
  - **违规清单（authoring 语义）**：生产代码中 selection/gizmo/inspector 词根（与 `SOURCE_AUTHORING_TOKENS` 25 词复用判据）命中者，逐项给迁移/删除条目。
  - 枚举命令：Grep `editor`，path `zircon_runtime/src`，glob `**/*.rs`（约 181 文件，按顶层目录分桶逐桶裁决）。
- 调用方迁移：无（裁决期；违规项的迁移在切片 1.4 或移交 owner）。
- 验收：白名单三分类表齐备，每桶有计数；违规清单每项有处置判词。
- DoD：清单落审计文档；违规清单为空或每条带 owner。

#### 切片 1.3 "legacy" 逐类裁决

- 目标文件：同 1.2 的审计文档落点；代表区域 `ui/surface/input/`（pointer_reply/pointer/navigation 集中区）。
- 改动形态：纯文档 + 裁决。三分类：
  - **领域词（有真实运行语义）**：如 `navigation.rs:22-54` 的 `legacy` 路由回复变量——route/focus/diagnostics 取值来源；裁决"保留 + 代码注释说明语义"或"改名为语义词（如 `routed_reply`）"，二选一判词。
  - **测试夹具**：asset 测试夹具中 legacy 样例按文件白名单。
  - **真实迁移债**：列入 UI 输入路径 owner 计划，本计划不展开。
  - 枚举命令：Grep `legacy`，path `zircon_runtime/src`，glob `**/*.rs`（403 处/84 文件基线，执行时重核）。
- 调用方迁移：无（裁决期）。
- 验收：84 文件分桶判词齐备；改名项（若有）列成独立切片清单。
- DoD：清单落审计文档，UI 债条目已移交（文档交叉引用）。

#### 切片 1.4 命名守卫机器化

- 目标文件：`zircon_runtime/src/scene/tests/authoring_boundary.rs`（扩展）或 `zircon_runtime/src/tests/`（新守卫文件，执行时按 1.2/1.3 白名单形态定稿落点）。
- 改动形态（已定稿）：

  ```rust
  #[test]
  fn runtime_editor_and_legacy_naming_is_classified_by_owner() {
      // 扫描 zircon_runtime/src，按文件 owner 分类 editor / legacy 命名；
      // 新增未分类命名会失败，已分类 debt 留给对应 owner 切片。
  }
  ```

- 调用方迁移：无。
- 验收：守卫对违规注入有负例（参照既有 `authoring_boundary_guard_fails_on_representative_tokens` :62 的自检模式）。
- DoD：`cargo test -p zircon_runtime --lib naming_boundary --locked` 全绿且白名单外新增命名会失败。

#### M1 测试阶段（milestone-first）

- 切片期：`cargo check -p zircon_runtime --lib --locked`
- 里程碑末：
  - `cargo test -p zircon_runtime --lib scene:: --locked`（无回归）
  - `cargo test -p zircon_runtime --lib authoring --locked -- --nocapture`（新守卫）
  - `cargo test -p zircon_runtime --lib naming_boundary --locked -- --nocapture`（命名白名单）
- 验收证据：白名单/违规清单写入既有命名审计文档（与 `20260604-1232` 口径合并）；守卫进常驻测试树。

### M2 守卫覆盖矩阵审计与维护公约

#### 切片 2.1 覆盖矩阵成表

- 目标文件：`docs/zircon_runtime/scene/inspection.md`（既有，刷新守卫说明；执行时核验：`ls docs/zircon_runtime/scene/`）。
- 改动形态：纯文档。成表：行 = 四个出口（world 序列化、dynamic scene、asset scene、inspection 快照），列 = 双 token 表（SERIALIZED 19 词 / SOURCE 25 词）与负例自检；逐格标注承载测试（已核实：`world_basics.rs` / `component_structure.rs` / `derived_state.rs` / `asset_scene.rs` / `inspection.rs:148,161`）。空格（某出口未覆盖某表）即补测试条目。
- 调用方迁移：无。
- 验收：矩阵无空格，或空格有补测试切片。
- DoD：矩阵落 `inspection.md` 且每格可点到测试名。

#### 切片 2.2 token 清单维护公约

- 目标文件：`scene/tests/authoring_boundary.rs`（注释公约）+ `docs/zircon_runtime/scene/inspection.md`（公约正文）。
- 改动形态：定稿公约——editor 侧新增 authoring 状态类型（如新 overlay/gizmo extract 类型）时，`SOURCE_AUTHORING_TOKENS` 必须同 PR 追加该类型名；公约写明判定规则（出现在 `zircon_editor/src/scene` 的 extract/投影类型名默认入表）。补一条结构测试（签名草案）：`authoring_token_tables_stay_sorted_and_deduplicated`（防清单腐化）。
- 调用方迁移：无。
- 验收：公约 + 结构测试。
- DoD：测试绿，公约可执行（有判定规则而非倡议）。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib authoring --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib inspection --locked -- --nocapture`
- 验收证据：覆盖矩阵 + 公约 + 守卫测试全绿；`docs/zircon_runtime/scene/inspection.md` 刷新。

### M3 收尾闭环（2026-06-12 二次细化新增；M1/M2 全切片已完成后的关账步骤）

#### 切片 3.1 legacy debt bucket 移交确认

- 目标文件：本计划状态节 + 各 owner 计划/审计文档交叉引用。
- 改动形态：核对 `runtime_naming_boundary` 审计输出中的 legacy naming debt bucket（2026-06-21 当前为 7 个：runtime graphics、DDS container policy、UI template schema、input event、asset schema、dynamic API migration、scene schema/render）——每个 bucket 必须可点到 owner 计划条目或审计文档判词；缺失者补交叉引用（input/UI template 债 → UI/Input 边界 owner 计划；DDS/graphics/scene render 债 → render/asset/scene 对应子计划）。
- 调用方迁移：无。
- 验收：10 bucket 全部有 owner 落点链接。
- DoD：移交表落状态节。

#### 切片 3.2 状态闭环

- 目标文件：本计划 frontmatter（status → completed）；`docs/plans/zircon_runtime/runtime/index.md` §3 状态行同步。
- 改动形态：收尾回归 `cargo test -p zircon_runtime --lib scene:: --locked`（全族无回归确认，此前验证用的是 naming_boundary/authoring/inspection 过滤词）；通过后翻转状态并在 §2.2 P6/P9 行补"已闭环"判词。
- 验收：scene:: 全族绿；index 与本计划状态一致。
- DoD：status: completed 落盘。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib scene:: --locked`
- 验收证据：命令输出摘要 + 状态翻转 + index 同步。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

2026-07-13 完成概述：`runtime_05_scene_1642_structure_1304_review_298_pmrem_parity_passed_closeout_acceptance_complete` 已在同一 fresh Windows lib-test 程序闭合 full `scene::`（1642 passed / 0 failed / 5 ignored）、`structure_convention`（1304/1304）、`code_review_findings`（298/298）以及 PMREM CPU/GPU parity（1/1，`PMREM_TEXEL_TOLERANCE=0.006` 未放宽）。机器守卫 `runtime_05_closeout_status_records_completed_scene_cargo_gate` 锁定该完成证据；父计划仅保留当前状态与路由，具体命令、时间和修复记录继续由编号归档拥有。

- 迁入记录：[`05/2026-07-09-scene-editor-boundary-closeout-output-records.md`](05/2026-07-09-scene-editor-boundary-closeout-output-records.md)
- fixed 已修复：[dynamic-scene-version-validation](02/fixed-2026-07-12-dynamic-scene-version-validation.md)
- fixed 已修复：[scene-test-support-file-budget](../frameworks/06/fixed-2026-07-13-scene-test-support-file-budget.md)
