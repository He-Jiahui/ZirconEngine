---
related_code:
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/hooks.rs
  - zircon_runtime/src/dynamic_api/session/linked_session.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_runtime/src/dynamic_api/session/error.rs
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/mod.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/registry/mod.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/tests/mod.rs
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/tests/runtime_errors.rs
  - zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/docs.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/runtime_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ui_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/dynamic_api.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/support.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/handle.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/handle.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/asset/project/script_manifest.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml.zmeta
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/scripts/vampire_game/main.zr
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
implementation_files:
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/error.rs
  - zircon_runtime/src/dynamic_api/session/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/mod.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/registry/mod.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/tests/mod.rs
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/tests/runtime_errors.rs
  - zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/docs.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/runtime_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ui_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/dynamic_api.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/support.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/handle.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/handle.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/asset/project/script_manifest.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml.zmeta
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl
  - examples/vampire/scripts/vampire_game/main.zr
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Zircon Runtime 独立 3D 游戏能力与 Vampire 示例计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/minimum-viable-engine-foundation.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-10 vampire screen-space HUD, buff particles, shader lighting, and no model health bars
  - user: 2026-06-11 vampire readable ground, start menu, game-over menu, Start/Retry buttons, and screenshot validation
  - user: 2026-06-12 remove runtime Vampire fallback backend
tests:
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
  - zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime/src/dynamic_api/session/tests/mod.rs
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/tests/runtime_errors.rs
  - zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/docs.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/runtime_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_profile.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/session/*.rs zircon_runtime/src/dynamic_api/session/tests/*.rs zircon_runtime/src/dynamic_api/runtime_loop.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/schedule_runner.rs zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/tests/*.rs
  - cargo test -p zircon_runtime --lib headless_session_capture_records_frame_extract_diagnostics --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1: pending after render-owned HZB compile blocker clears
  - cargo test -p zircon_runtime --lib frame_extract_rebuild_skips_unchanged_entities --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1: pending after render-owned HZB compile blocker clears; source/rustfmt static checks passed 2026-06-13
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never: blocked 2026-06-13 before this dynamic_api slice by render-owned HZB errors (`HzbOcclusionCullReport` missing; two `expected &ShadowMapRenderer, found &HzbOcclusionCuller`)
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_markdown.py
  - dynamic_api_test_boundary targeted audit after session test-owner split and 2026-06-21 Markdown renderer split: `expected_module_count = 12`, `legacy_tests_file_exists = false`, `oversized_modules = []`, `risks = []`, `dynamic_api_test_boundary.py = 89`, `dynamic_api_test_markdown.py = 35`
  - dynamic_runtime_api_boundary targeted audit and 2026-06-21 Markdown renderer split: `expected_source_file_count = 35`, `function_table_structs = 10/10`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `runtime_diagnostics_anchors = 15/15`, `missing_runtime_diagnostics_anchors = []`, `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_contract_duplicate_public_types = 0`, `ui_v2_contract_sync_anchors = 9/9`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `mirror_docs_guard_present = true`, `risks = []`, `dynamic_runtime_api_boundary.py = 330`, `dynamic_runtime_api_markdown.py = 65`; `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` passed static audit; Cargo gates pending
  - cargo test -p zircon_runtime --lib mouse_wheel_at_events_decode_delta_bits_for_dynamic_session --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture: passed 2026-06-21 with `1 passed; 0 failed; 4704 filtered out`; validates that the mouse-wheel-at-point source guard reads `session/events.rs`, the current owner of `handle_mouse_wheel`
  - cargo test -p zircon_runtime --lib vampire_project_session_w_key_moves_player_before_input_clear --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture: default-feature gate passed 2026-06-21 with `0 passed; 0 failed; 1 ignored`; the test now explicitly requires `backend-zr-vm` and `ZR_VM_RUST_BINDING_LIB_DIR`
  - cargo test -p zircon_runtime --lib destroy_session_removes_registry_entry_so_destroyed_handles_become_missing --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-03-gates-0704 --message-format short --color never -- --nocapture --test-threads=1: passed 2026-07-04 with 1/1 after splitting the session_lifecycle source guard across private Rust `session.rs` and FFI `exports.rs`
  - cargo test -p zircon_runtime --lib runtime_10_dynamic_session_test_owner_split_keeps_focused_modules --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-03-gates-0704 --message-format short --color never -- --nocapture --test-threads=1: passed 2026-07-04 with 1/1 after updating the test-owner guard to the current `runtime_session_error_preserves_step_and_typed_source` anchor
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-03-gates-0704 --message-format short --color never -- --test-threads=1: passed 2026-07-04 with 1 passed / 10 ignored after all real-backend Vampire project-session behavior tests were marked as requiring `backend-zr-vm`; the static WASD source check still runs by default
  - cargo test -p zircon_runtime --lib session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-03-gates-0704 --message-format short --color never -- --test-threads=1: passed 2026-07-04 with 161 passed / 0 failed / 10 ignored after default-feature real-backend gates were completed
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --message-format short
  - cargo check -p zircon_runtime --lib --message-format short --color never with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo check -p zircon_runtime --lib: passed 2026-06-09
  - cargo build -p zircon_app --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,backend-zr-vm" --bin zircon_runtime with ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug and PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09
  - target\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire with ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug and PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09; startup no longer reports vm backend not registered
  - cargo test -p zircon_runtime --lib dynamic_api::session::input_events::tests::keyboard_logical_key_maps_wasd_runtime_key_codes_for_gameplay_scripts -- --exact --nocapture: validates WASD runtime key-code mapping for project gameplay scripts
  - cargo test -p zircon_runtime --lib vampire_project_session_w_key_moves_player_before_input_clear --locked --jobs 1 --message-format short --color never -- --nocapture: passed 2026-06-09; validates project-session W input is consumed before per-frame input clearing and moves the vampire player
  - cargo test -p zircon_runtime --lib vampire_project_session -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates W movement, automatic Blood Bolt damage, player attack action-state writes, and enemy behavior-tree chase state in the standalone project session path
  - cargo test -p zircon_runtime --lib vampire_project_session_writes_hud_and_spawns_dynamic_enemy --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates runtime HUD component text and dynamic enemy spawn in the standalone project-session path
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 4 project-session tests passed, covering WASD, Blood Bolt damage, nav chase, HUD text, and dynamic enemy spawning
  - cargo test -p zircon_runtime --lib asset::tests::project::example_vampire::vampire_example_scene_extracts_playable_third_person_meshes -- --exact --nocapture: validates vampire scene import, mesh extraction, and third-person camera direction
  - target\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire captured examples\vampire\screenshots\vampire-runtime-playable.png on 2026-06-09; WASD movement is covered by the project-session input test because the follow camera can keep the player visually centered
  - cargo test -p zircon_runtime --lib runtime_session_error_preserves_step_and_typed_source --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo build -p zircon_app --bin zircon_runtime --features "target-client,first-party-runtime-plugins,first-party-navigation-runtime-plugin,first-party-zr-vm-language-runtime-plugin,backend-zr-vm" --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app, ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug, PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: passed 2026-06-09 after built-in GLB/image/text importer promotion
  - D:\cargo-targets\zircon-vampire-app\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire with ZR_VM_RUST_BINDING_LIB_DIR=E:\Git\zr_vm\build\codex-msvc-debug\lib\Debug and PATH including E:\Git\zr_vm\build\codex-msvc-debug\bin\Debug: remained running for 18 seconds on 2026-06-09; stdout/stderr empty, runtime log file empty, examples/vampire zmeta scan found no preview_state error or plugin_required GLB/image importers
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 6 project-session tests passed, covering WASD, Blood Bolt damage, nav chase, HUD text, dynamic enemy spawning, frame HUD capture, and extended-runtime HUD retention
  - ZR_VAMPIRE_CAPTURE_PNG=E:\Git\ZirconEngine\examples\vampire\screenshots\vampire-runtime-capture-current.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=140 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 and exported a 1280x720 late-frame PNG
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates generated jungle models, terrain albedo texture binding/import, multi-polygon height-varying jungle navmesh, scripts, shaders, and GLB records
  - cargo test -p zircon_runtime --lib vampire_example_scene_extracts_playable_third_person_meshes --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates third-person camera and jungle scene mesh extraction
  - cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 10 project-session tests passed, covering WASD, auto attacks, nav chase, HUD, dynamic spawning, XP/level choice, pickups/buffs, chest weapon choice, boss spawn, and screenshot HUD capture
  - cargo check -p zircon_runtime --lib --locked --message-format short with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after the graphical HUD, terrain-backed jungle, and screen-space health HUD update; existing zircon_runtime warnings only
  - cargo test -p zircon_runtime --lib runtime_session_hud_extract_builds_graphical_vampire_bars_from_text --locked --message-format short -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates graphical vampire HUD bars, icon slots, and text payloads
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after final terrain-backed jungle update; validates scene terrain component, TerrainAsset source, material layer, and navmesh height variation
  - cargo test -p zircon_runtime --lib vampire_project_session_ --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 10 project-session tests passed after graphical screen-space HUD update
  - ZR_VAMPIRE_CAPTURE_PNG=E:\Git\ZirconEngine\examples\vampire\screenshots\vampire-runtime-frame.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=60 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; exported final PNG with graphical HUD and terrain scene
  - cargo check -p zircon_runtime --lib --locked --message-format short with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after screen-space HUD, dynamic attack particles, and shader update; existing zircon_runtime warnings only
  - cargo test -p zircon_runtime --lib runtime_session_hud_extract_builds_graphical_vampire_bars_from_text --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; validates richer graphical HUD command and color payloads
  - cargo test -p zircon_runtime --lib vampire_project_session_ --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; 11 project-session tests passed, including no model health bars, base/buffed particles, HUD, choices, pickups, boss spawn, and frame capture
  - ZR_VAMPIRE_CAPTURE_PNG=E:\Git\ZirconEngine\examples\vampire\screenshots\vampire-runtime-hud-particles-shadow.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=90 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; exported runtime screenshot with screen-space HUD, shadow stats, and attack-particle data
  - cargo build -p zircon_app --bin zircon_runtime --locked --message-format short with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10; built standalone runtime executable with existing warnings only
  - D:\cargo-targets\zircon-vampire-app\debug\zircon_runtime.exe --project E:\Git\ZirconEngine\examples\vampire: remained running for 8 seconds on 2026-06-10 with empty stdout/stderr, then was stopped by the smoke test
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/session/menu.rs zircon_runtime/src/dynamic_api/session/tests/*.rs zircon_runtime/src/script/vm/gameplay_host.rs zircon_runtime/src/script/vm/gameplay_host/tests.rs zircon_runtime/src/asset/tests/project/example_vampire.rs: pending current validation stage for the 2026-06-11 start/game-over menu and readable-ground update
  - cargo test -p zircon_runtime --lib runtime_session_menu --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib gameplay_host_component_string_reads_string_dynamic_state --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --target-dir D:\cargo-targets\zircon-vampire-menu-0611 -- --nocapture --test-threads=1: pending current validation stage
  - cargo test -p zircon_runtime --lib vampire_project_session_starts_paused_until_start_button_click --features backend-zr-vm --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1 with Release ZR_VM_RUST_BINDING_LIB_DIR and PATH: passed 2026-06-16
  - cargo test -p zircon_runtime --lib vampire_project_session_game_over_menu_retries_to_playing --features backend-zr-vm --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1 with Release ZR_VM_RUST_BINDING_LIB_DIR and PATH: passed 2026-06-16
  - cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_world_hud_bars --features backend-zr-vm --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1 with Release ZR_VM_RUST_BINDING_LIB_DIR, PATH, ZR_VAMPIRE_CAPTURE_PNG, and 640x360 capture settings: passed 2026-06-16 after core `particle-render` insertion, world-HUD overlay depth routing, and EV100 scene exposure migration
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session.rs zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs zircon_runtime/src/dynamic_api/tests/support.rs: passed 2026-06-12 after optional render-bridge and headless/minimal lifecycle update
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs zircon_runtime/src/tests/runtime_absorption/mod.rs: passed 2026-06-13 for the Runtime 10 runtime-absorption guard
  - git diff --check -- zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs zircon_runtime/src/tests/runtime_absorption/mod.rs docs/zircon_runtime/dynamic_api/session.md docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md docs/plans/zircon_runtime/runtime/index.md .codex/sessions/20260612-0847-runtime-architecture-implementation.md: passed 2026-06-13 with LF-to-CRLF warnings only
  - conflict-marker/trailing-whitespace scan over the Runtime 10 runtime-absorption guard, runtime 10 plan docs, this module doc, and the active session note: passed 2026-06-13
  - source/doc anchor scan for optional RuntimeRenderBridge, uses_render_bridge(), empty-frame capture fallback, bind/unbind/present no-op, and runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces: passed 2026-06-13
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs zircon_runtime/src/tests/runtime_absorption/plan_status.rs: passed 2026-06-13 for the Runtime 10 M1.3 FFI panic-boundary absorption guard
  - source/doc anchor scan for runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary and runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge across exports/session/API-table tests/module docs/Runtime 10/index/M0 review: passed 2026-06-13
  - cargo test -p zircon_runtime --lib destroy_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-headless-0612 --message-format short --color never -- --nocapture: timed out after 904s on 2026-06-12 during Windows test-target compilation while another zircon_runtime cargo lane was active; the orphaned validation process was stopped and no Cargo pass is claimed
  - rustfmt --edition 2021 --check zircon_runtime/src/dynamic_api/session/project.rs (2026-07-14 hard-cut resolver caller return: passed)
  - source invariant scan for zero `crate::asset::pipeline::manager::resolve_asset_manager` calls and one `asset_manager_handle(core)` use-point resolution in `session/project.rs` (2026-07-14 hard-cut resolver caller return: passed; full Cargo rerun remains with the originating Shader04 validation)
  - rustfmt --edition 2021 --check zircon_runtime/src/script/vm/backend/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/tests.rs: passed 2026-06-12 after removing runtime project fallback backend
  - git diff --check -- zircon_runtime/src/script/vm/backend/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/tests.rs docs/zircon_runtime/script/vm/zr_vm_host_reflection.md docs/zircon_runtime/dynamic_api/session.md: passed 2026-06-12 with LF-to-CRLF warnings only
  - cargo test -p zircon_runtime --lib discovery_rejects_zr_vm_project_fallback_backend --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-no-fallback-0612 --message-format short -- --nocapture --test-threads=1: timed out after 604s on 2026-06-12 during Windows test-target compilation; orphaned cargo/rustc processes were stopped and no Cargo pass is claimed
doc_type: module-detail
---

# Dynamic Runtime Session

`zircon_runtime::dynamic_api::session` owns the runtime session state behind the exported `zircon_runtime_interface` C ABI. The public function table remains in `dynamic_api::exports`; session code owns handle validation, session registry access, event dispatch into runtime managers, frame capture/presentation, profile control, and host request draining.

The session module is intentionally private to `zircon_runtime`. Its job is to adapt versioned ABI payloads into existing runtime facilities without turning the dynamic library boundary into a second runtime architecture.

## 2026-07-10 executable and current-owner validation

Plan sources are the numbered Runtime09, Runtime10, Runtime15, and Frameworks02 output records. Guard implementation is owned by `tests/runtime_absorption/dynamic_api_session/**` and the Dynamic API Vampire naming guard; runtime production behavior was not changed in this reconciliation.

The available default-feature `dynamic_api` filter executed 85 behavior tests successfully, ignored 10 tests that require `backend-zr-vm` and its binding library, and reported eight stale archive/current-owner guard failures. Current-source standalone validation passes the full Runtime10 guard suite 11/11 and the exact naming guard 1/1. The aggregate structure audit reports no Runtime10 risk section. A fresh default-feature filter remains pending; acceptance evidence is recorded in `tests/acceptance/runtime-dynamic-api-current-result.md`.

## Owner Split

- `session.rs` keeps the Rust-ABI session owner functions and `RuntimeDynamicSession` lifecycle/orchestration. The folder-backed `session/registry/mod.rs` is a zero-behavior facade; `session/registry/session_store.rs` owns the session registry and registry lock, while `session_slot.rs` owns the per-session lock and close barrier. The exported C ABI entry points live in `exports.rs` wrappers.
- `session/events.rs` owns dynamic session event dispatch and input adaptation: pointer, mouse, touch, keyboard, IME, file-drag, window, gamepad, and accessibility events are translated into `core::framework::input` DTOs or camera/menu preview actions there. `session.rs` keeps only the private Rust-ABI `handle_event(...)` owner entry and delegates through `with_session(...)`.

Runtime 15 M2 input mouse-wheel line-delta naming hard cutover records `runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred`: dynamic preview scroll now uses `zircon_runtime/src/core/framework/input/mouse_wheel.rs` through `MouseWheelEvent::vertical_line_delta()` when it needs the scalar camera orbit delta. `naming_boundary/runtime_15_m2/input.rs::runtime_15_input_mouse_wheel_line_delta_uses_current_names` keeps `session/events.rs`, the input framework owner, and this document aligned without retaining old helper aliases.
- `session/diagnostics.rs` owns the `ProfileControlCommand::RuntimeDiagnosticsSnapshot` projection. It reuses the existing `profile_control` JSON ABI entry, collects `DiagnosticStore` series through `collect_runtime_diagnostics(...)`, and adds the last dynamic scene-asset reload frame report as neutral `RuntimeSceneAssetReloadDiagnostics` counts.
- `session/project.rs` adapts the optional ABI project-manifest byte slice into a runtime project root, loads `zircon-project.toml`, discovers startup ZrVM packages, and loads the manifest default scene through the existing scene/project asset path.
- `session/extract.rs` owns the dynamic-session frame extract facade: viewport resize, scene `RenderFrameExtract` cache lookup/construction, UI side-path extract selection, and the runtime-side extract diagnostic hook.
- `session/extract_cache.rs` owns `RuntimeFrameExtractCache`, the runtime-owned frame extract reuse key, and explicit cache invalidation for viewport changes. The cache key combines world `change_tick`, `query_cache_revision`, active camera, and viewport size so unchanged captures can reuse the previous `RenderFrameExtract` while scene edits rebuild.
- `session/extract_stats.rs` estimates the per-extract output collection footprint and records `extract.rebuild_clones`, `extract.output_bytes`, `extract.cache_hits`, and `extract.cache_misses` into the existing `DiagnosticStore`.
- `session/scene_asset_reload_diagnostics.rs` projects dynamic scene-asset reload frame reports into count/bool `DiagnosticStore` rows under `scene.asset_reload.*`.
- `session/status.rs` owns ABI `ZrStatus` construction for unsupported version, invalid argument, not found, and generic dynamic API errors.
- `session/host_requests.rs` converts neutral runtime input-manager host requests into ABI host request payloads.
- `session/input_events.rs` maps ABI numeric input/window/gamepad/IME constants into `core::framework::input` DTOs, including the ASCII-style WASD and digit-key codes emitted by the standalone app's physical-key converter for gameplay scripts and choice prompts.
- `session/preview.rs` owns fallback frame and accessibility preview payloads used when the dynamic preview cannot extract a full UI surface.
- `session/profile.rs` owns `RuntimeDynamicSessionProfile`, ABI profile byte parsing, fixed-step policy, diagnostic-log schedule selection, and render-bridge enablement policy. `session.rs` imports only the profile type and keeps session creation, ticking, rendering, and lifecycle orchestration.
- `session/registry/mod.rs` is a zero-behavior child mount and re-export facade. `session/registry/session_store.rs` owns the global registry, handle allocation, poison-safe lookup/destroy, and `with_session` dispatch; `session_slot.rs` owns action admission, the private session lock, and close/wake quiescence; `wake_registration.rs` owns callback admission/drain plus the current-thread callback marker. A synchronous same-session destroy issued from its active wake callback is rejected before close admission, while cross-thread destroy still disables and drains callbacks before unload. `session.rs` imports only the facade entry points it needs for ABI create/destroy/dispatch.
- `session/hud.rs` extracts runtime HUD UI from gameplay dynamic components such as `gameplay.hud_text` and passes it through the normal frame presentation path. Generic text still has a fallback path, while the vampire sample HUD parser converts structured HP/XP/time/weapon/buff text into a graphical screen-space panel with bars, icon slots, and prompt rows. Combat health bars for the sample must remain in this HUD layer instead of reappearing as scene mesh/cube entities.
- `session/menu.rs` extracts the runtime gameplay menu overlay from `gameplay.menu_state` and handles pointer hit-testing for its single command button. It writes the selected command back through `gameplay.control_state`, keeping Start/Retry interaction in the same dynamic-component channel the project script can consume on the next tick.
- `session/tests/` owns focused tests for the private session orchestrator. `vampire_runtime_support.rs` keeps shared project-session fixtures, `vampire_gameplay.rs` owns movement/combat/AI assertions, `vampire_menu.rs` owns Start/Game Over flows, `vampire_hud.rs` owns world-HUD capture checks, `frame_diagnostics.rs` owns extract/FPS diagnostics, and `runtime_errors.rs` owns narrow error-format coverage.
- `tests/` mirrors the same owner split for the exported API table, profile control, viewport/frame validation, session lifecycle, session entry-point handle validation, session profile/source-shape guards, host requests, accessibility, and input-event rejection paths.
- `tests/structure.rs` keeps that mirror executable by rejecting a recreated `tests.rs`, missing owner modules, non-navigational `mod.rs` content, and owner files that grow past the split threshold.

Runtime 15 M4 dynamic API session profile owner split is recorded as `runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred`. `dynamic_api/session.rs` remains the runtime ABI session lifecycle owner, while `dynamic_api/session/profile.rs` owns `RuntimeDynamicSessionProfile`, profile byte parsing, fixed-step policy, diagnostic-log schedule selection, and render-bridge enablement. The structure guard `runtime_15_dynamic_api_session_profile_is_child_owner` keeps the profile enum/constants/methods from returning to the parent and keeps both owners below the production-file budget.

Runtime 15 M4 dynamic API session registry owner split is recorded as `runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred`. `dynamic_api/session.rs` remains the runtime ABI session lifecycle owner, `dynamic_api/session/registry/mod.rs` remains a zero-behavior facade, `registry/session_store.rs` owns `SESSION_REGISTRY`, `SessionRegistry`, handle allocation and dispatch, and `registry/session_slot.rs` owns action/close admission plus the private session lock. The structure guard `runtime_15_dynamic_api_session_registry_is_child_owner` keeps behavior out of the facade and parent and keeps each production owner within budget.

This keeps the FFI boundary below the large-file warning line. The V6 ABI hard cut remains one atomic Runtime10/Editor02 candidate and is not accepted by a structure-only split.

## FFI Panic Boundary

`dynamic_api::exports` owns the final panic containment layer for the exported C ABI. `zircon_runtime_get_api_v6` validates `ZrHostApiV1` before returning the static `ZrRuntimeApiV6` table, and performs that lookup through an inner helper wrapped by `catch_unwind`; an unexpected unwind during table acquisition returns a null table pointer instead of crossing the `extern "C"` boundary.

Every advertised `ZrRuntimeApiV6` session function pointer points at an `_ffi` wrapper in `exports.rs`. The wrappers delegate normal validation and behavior to private Rust-ABI `dynamic_api::session` and `session::operation` owner functions, then translate any unexpected unwind into `ZrStatusCode::Panic` with the stable diagnostic `runtime dynamic API panic caught at FFI boundary`. This keeps version checks, handle checks, project loading, event routing, capture, surface, profile-control, tick, host-request, plugin-event, operation, and world-sync semantics in their session owners while keeping the ABI panic guard at the final dynamic-library edge.

`runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary` is the focused API-table source guard for this split: it rejects direct function-table entries that bypass the wrappers, requires the shared panic translator, locks the null-return path for panics during `zircon_runtime_get_api_v6`, and rejects `extern "C"` declarations from the private session and operation owner functions so unwind containment remains catchable.

`runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge` is the runtime-absorption guard for the same contract. It ties `exports.rs`, private session owners, the focused API-table test, this module document, Runtime 10, and the runtime index together so the exported C ABI edge cannot silently drift back into `session.rs`.

## Session Lifecycle Failure Contract

Session lifecycle failures are part of the exported ABI contract, not private implementation details. `destroy_session` accepts each live handle once, then reports `NotFound` with `runtime session not found` after registry removal. `ZrRuntimeSessionHandle::invalid()` is rejected before registry lookup with `InvalidArgument` and `invalid runtime session handle`.

`session_lifecycle.rs` owns the create/destroy/tick lifecycle cases: `destroy_session_reports_explicit_not_found_for_missing_nonzero_handle`, `destroy_session_removes_registry_entry_so_destroyed_handles_become_missing`, and `session_destroy_reports_explicit_not_found_after_headless_destroy`. `session_entry_points.rs` owns the cross-entry handle rejection cases: `all_session_entry_points_reject_invalid_handle`, `destroyed_headless_session_entry_points_reject_old_handle`, and `missing_session_entry_points_reject_nonzero_handle`. The dynamic entry-point coverage intentionally uses otherwise valid event, frame, viewport, profile, and host-request arguments so the tests reach session validation instead of version, viewport, or payload preflight branches.

On 2026-07-04, the lifecycle source guard was synchronized with the current ABI split: private Rust `destroy_session` remains in `session.rs`, while the C ABI `_ffi` wrapper remains in `exports.rs`. The focused destroy-session guard passed 1/1 with that split and did not move an exported C edge back into the session owner.

`minimal` and `headless` profiles now skip `RuntimeRenderBridge` creation. They still register and activate runtime modules, install scene hooks, create or load a level, dispatch input, tick time, and drain host requests, but frame capture returns an empty encoded frame and surface bind/unbind/present operations are no-ops. This keeps lifecycle and ABI validation independent from WGPU device limits while preserving the rendered `runtime`/`editor`/`dev` profiles.

`runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces` is the runtime-absorption source/doc guard for that contract. It keeps `RuntimeDynamicSession.render_bridge` optional, restricts `uses_render_bridge()` to rendered profiles, requires empty-frame capture fallback when no bridge is installed, and requires bind/unbind/present to return `Ok(())` rather than touching WGPU when `minimal` or `headless` skipped bridge creation.

## Extract Diagnostics

`RuntimeDynamicSession::current_extract()` records runtime-owned extract diagnostics after resolving the `RenderFrameExtract` and before handing it to the render bridge. Runtime 07 M2 now routes this through `RuntimeFrameExtractCache`: the first call for a `(change_tick, query_cache_revision, active_camera, viewport_size)` key calls `World::to_render_frame_extract()` and records `extract.rebuild_clones = 1`, `extract.cache_misses = 1`, and `extract.cache_hits = 0`; an unchanged follow-up capture clones the cached extract and records `extract.rebuild_clones = 0`, `extract.cache_hits = 1`, and `extract.cache_misses = 0`; a world mutation, structural query-cache revision change, active-camera change, or viewport resize builds a fresh extract again.

`extract.output_bytes` is a stable estimate of the main output collections carried by the extract: geometry vectors and phase queues, static-batch side vectors, virtual-geometry side vectors, animation poses, light lists, post-process stack/graph vectors and string payloads, overlay vectors, sprite/particle vectors, and visibility vectors. It is intentionally not a precise heap profiler and does not serialize the extract. Its purpose is before/after comparison for Runtime 07 M1/M2 hot-path work without adding frame-path allocation overhead.

`headless_session_capture_records_frame_extract_diagnostics` covers the diagnostic path without requiring WGPU. A first headless capture builds the session extract, records `extract.rebuild_clones = 1`, `extract.cache_hits = 0`, `extract.cache_misses = 1`, and writes a non-zero `extract.output_bytes` sample into the runtime diagnostic store once the current render-owned Cargo blocker is cleared and the focused test can run.

`frame_extract_rebuild_skips_unchanged_entities` is now the Runtime 07 M2 named assertion for the extract cache. Two unchanged headless captures must record `extract.rebuild_clones = [1, 0]`, `extract.cache_hits = [0, 1]`, and `extract.cache_misses = [1, 0]`, while `extract.output_bytes` remains non-zero and stable. `frame_extract_rebuilds_after_scene_change` mutates the active camera transform between captures and requires rebuilds `[1, 1]`, hits `[0, 0]`, and misses `[1, 1]`, proving scene mutations invalidate the dynamic-session extract cache instead of reusing stale frame data.

## Frame Profiling Spans

Runtime 07 M0.3 adds profiling spans for the outer frame breakdown without changing runtime behavior. `tick_frame()` records `runtime_frame_time_update` around clock advancement and `runtime_frame_update` around level tick execution. `session/extract.rs` records `runtime_frame_extract` around scene extract construction. `runtime_loop.rs` records `runtime_frame_submit` around both capture-submit and surface-present handoff to the render framework. `SceneScheduleRunner::run_stage(...)` records `runtime_frame_schedule_stage.<SystemStage>` as a dynamic stage-level span inside the update phase, so a trace can separate `First`/`PreUpdate`/fixed-loop/`Update`/`RenderExtract` work without changing scheduling or flush semantics.

The trace/profiling acceptance command is still pending until the current render-owned HZB Cargo blocker and active compile lanes clear. Static span and extract-cache coverage is locked by `runtime_07_hotspot_inventory_requires_counted_evidence_before_m2` and mirrored by `performance_hotpath_boundary`, which reports `expected_source_file_count = 46`, `expected_test_file_count = 14`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` keeps this dynamic-session mirror aligned with Runtime 07, the runtime index, the hotspot inventory, ECS docs, runtime-interface convergence, and the M0 review, including the QueryState cache owner split, animation scene hook count rows from `AnimationSceneFrameDiagnostics`, the profiling counter hotspot export `counter_hotspots.json`, the render product diagnostics owner split `render_product_diagnostics_owner_split_static_passed_cargo_deferred`, and the navigation runtime owner split. This mirror is static evidence only; the extract/ecs_query/profiling/FPS Cargo lane remains pending.

Current owner-budget mirror refresh 2026-07-01: `performance_hotpath_boundary` now reports `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, and `risks = []`. This only updates the large-file gate mirror; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-01: `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` moved to `performance_hotspots/owner_budget/mirror_docs.rs`, and Runtime 07 audit input now includes `performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs`. The current mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 14`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This is still static evidence only; the extract/ecs_query/profiling/FPS lane remains pending.

Current mirror-docs owner refresh 2026-07-05: `Runtime 15 M3 Runtime 07 submit-context guard child-owner split` / `runtime_15_runtime_07_submit_context_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/submit_context/{sources,source_extract_payloads,camera_loop_sharing,feedback_sidebands,status_docs,split_layout}.rs` to Runtime 07 audit input. The current mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 20`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This is still static evidence only; the extract/ecs_query/profiling/FPS lane remains pending.

Current mirror-docs owner refresh 2026-07-05: `Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split` / `runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/{sources,evidence_gate_docs,ecs_extract_counters,profiling_trace_render,split_layout}.rs` to Runtime 07 audit input. The current mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 25`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This is still static evidence only; the extract/ecs_query/profiling/FPS lane remains pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split` / `runtime_15_runtime_07_owner_budget_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/{sources,parent_routes,child_routes,source_inventory,line_budgets,status_docs,split_layout}.rs` to Runtime 07 audit input. The current mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 32`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This is still static evidence only; the extract/ecs_query/profiling/FPS lane remains pending.

## Runtime Diagnostics Profile-Control Snapshot

`profile_control` accepts `ProfileControlCommand::RuntimeDiagnosticsSnapshot` as a session-owned JSON command. The command returns `ProfileControlResponse.runtime_diagnostics` with `RuntimeDiagnosticsSnapshot`, `RuntimeDiagnosticSeriesSnapshot`, `RuntimeDiagnosticMeasurement`, and `RuntimeSceneAssetReloadDiagnostics` DTOs from `zircon_runtime_interface::profiling`.

This command intentionally does not add a new V6 runtime-table function pointer. Hosts still call the existing `profile_control` entry and free the returned buffer through the same runtime-owned profile buffer callback. Calling the lower-level profiling recorder with this command returns `runtime diagnostics snapshot requires dynamic session`, because only `RuntimeDynamicSession` owns the live `CoreRuntime`, project scene-asset reload queue, and last reload frame report.

When no project scene-asset reload queue exists, the response still includes `scene_asset_reload.enabled = false` with zero counts. When reload is enabled, the response mirrors the last frame's drained/scheduled/skipped/superseded/applied/failed/stale/pending counts and receiver-disconnected flag. Detailed asset events and runtime error types stay inside `zircon_runtime`; the dynamic ABI exposes only stable JSON counts.

The Runtime 10 scene-asset reload diagnostic path guard locks the frame report bridge and stable `DiagnosticStore` paths with status `runtime_10_scene_asset_reload_diagnostic_paths_static_guard_rustfmt_passed_cargo_deferred_tests_deferred`. The guard requires `tick_scene_asset_reload` to keep `tick_into_level`, `record_scene_asset_reload_frame_report`, and `last_scene_asset_reload_report`, and it pins the `scene.asset_reload.events_drained`, `scheduled`, `skipped`, `skipped_removed`, `skipped_reload_failed`, `skipped_missing_locator`, `skipped_stale_revision`, `superseded_pending`, `applied`, `failed`, `stale`, `pending`, and `receiver_disconnected` paths under the `["scene", "asset_reload"]` subsystem tags.

The 2026-06-20 Runtime 10 diagnostics inventory split records `runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_diagnostics_inventory.py` owns the runtime diagnostics and scene-asset reload diagnostic path inventories, and `dynamic_runtime_api_boundary` now reports `scene_asset_reload_diagnostic_path_anchors = 21/21` with `missing_scene_asset_reload_diagnostic_path_anchors = []` alongside `runtime_diagnostics_anchors = 15/15`.

The 2026-06-20 Runtime 10 host-request inventory split records `runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_host_request_inventory.py` owns the 38 host-request payload anchors for interface DTOs, runtime conversion, dynamic API tests, and app-side routing, and `dynamic_runtime_api_boundary` still reports `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, and `risks = []`.

The 2026-06-20 Runtime 10 UI contract inventory split records `runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_ui_contract_inventory.py` owns the UI pending-gate, single-source contract, and v2 sync anchors, and `dynamic_runtime_api_boundary` still reports `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_v2_contract_sync_anchors = 9/9`, and `risks = []`.

The 2026-06-20 Runtime 10 validation inventory split records `runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_validation_inventory.py` owns the behavior-test, pending Cargo gate, doc-anchor, and mirror-doc guard inventories, and `dynamic_runtime_api_boundary` still reports `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `missing_doc_anchors = []`, and `risks = []`.

The 2026-06-20 Runtime 10 session lifecycle inventory split records `runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_session_lifecycle_inventory.py` owns the headless/minimal lifecycle anchors, and `dynamic_runtime_api_boundary` still reports `headless_lifecycle_anchors = 12/12`, `missing_headless_lifecycle_anchors = []`, and `risks = []`.

The 2026-06-20 Runtime 10 failure boundary inventory split records `runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_failure_inventory.py` owns the FFI panic and loader failure anchors, and `dynamic_runtime_api_boundary` still reports `ffi_panic_anchors = 9/9`, `missing_ffi_panic_anchors = []`, `loader_failure_anchors = 10/10`, `missing_loader_failure_anchors = []`, and `risks = []`.

The 2026-06-21 Runtime 10 ABI source inventory split records `runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred`: `dynamic_runtime_api_abi_inventory.py` owns the source owner list, function table shape inventory, and session operation list, and `dynamic_runtime_api_boundary` still reports source files 35/35, function tables 10/10, runtime session wrappers 11/11, no direct table-entry bypass, `session_owner_extern_c_present = false`, and `risks = []`. The focused package check `cargo test -p zircon_runtime --lib dynamic_api_session --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-shared` timed out after 904s with no test result, so broader dynamic API/app/UI gates remain pending.

The 2026-06-21 Runtime 10 dynamic runtime API Markdown renderer split records `dynamic_runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_markdown.py` owns `render_dynamic_runtime_api_boundary_markdown(...)`, `dynamic_runtime_api_boundary.py` remains the 330-line audit/risk owner, and the Markdown owner is 65 lines. The split keeps the same 34/34 source, 10/10 function-table, 11/11 runtime-session wrapper, host-request 38/38, UI contract, doc-anchor, mirror-doc, and `risks = []` evidence; package-level dynamic API/app/UI gates remain pending.

## Dynamic Session Event Split

The 2026-06-14 event split moved the private event router and input handlers out of `session.rs` into `session/events.rs`. The split keeps `session.rs` below the large-file threshold while preserving the ABI rule that `exports.rs` wraps C entry points and `session.rs` owns private Rust-ABI session functions. The new guard `runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router` verifies `mod events;`, the private `handle_event(...)` owner entry, and the event helper ownership in `session/events.rs`. Runtime 07 also records the same split through `runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner`, tying this dynamic session event split to the current `large_file_hotspot_count = 0` / empty owner-class state.

## Dynamic Session Test Owner Split

The 2026-06-14 test split replaces the removed `session/tests.rs` monolith with `session/tests/{mod,vampire_runtime_support,vampire_gameplay,vampire_menu,vampire_hud,frame_diagnostics,runtime_errors,lock_poison}.rs`. `mod.rs` is only the navigation surface. `vampire_runtime_support.rs` owns shared vampire project-session setup, entity/component inspection, capture export, diagnostics lookup, and frame-request support. Gameplay, menu, HUD, frame-diagnostic, error-format, and lock-poison recovery assertions now live in separate owner modules so new dynamic-session coverage can grow by feature without reopening an unrelated thousand-line test file.

`frame_diagnostics.rs` is also the Runtime 07 evidence owner for `headless_session_capture_records_frame_extract_diagnostics`, `frame_extract_rebuild_skips_unchanged_entities`, `frame_extract_rebuilds_after_scene_change`, and `vampire_project_session_reports_runtime_fps_and_render_work`. The performance-hotpath audit therefore points at that file rather than the removed monolithic test module. `runtime_10_dynamic_session_test_owner_split_keeps_focused_modules` rejects recreating `session/tests.rs`, requires the folder-backed declarations, and keeps the module docs, Runtime 10 plan, and runtime index aligned with this owner split.

`foundation_render.rs` owns the F2 minimum-engine product contract. It builds a persisted temporary project with one camera, one static mesh primitive, and one static directional light, then enters through `RuntimeDynamicSession::new(Runtime, project)` rather than a renderer-only fixture. The test routes viewport, pointer, mouse-button, and keyboard ABI events into the real input manager before capturing a 640x360 frame through `RuntimeRenderBridge` and WGPU. Green primitive pixels plus RenderGraph, mesh-draw, light-extract, and material-validation counters prove that the visible result is not a static file or an empty/background-only capture.

The same product test captures an unchanged second frame and uses deterministic counters for performance acceptance: compiled-graph misses and cache entries stay fixed, one cache hit is added, the draw set stays fixed, and GPUScene reports zero dirty entries and zero uploaded bytes. It then drops the first session, creates a second session from the same persisted project, and requires matching mesh/light counts. `ZR_F2_BASIC_SCENE_CAPTURE_PNG` exports the accepted current-source frame to `docs/tests/runtime/render/f2_basic_scene_render_input_exit_wgpu_20260717.png`. The implementation is present; managed WGPU execution and PNG inspection remain pending until recorded in the Plan 01 child output.

On 2026-07-04, `runtime_10_dynamic_session_test_owner_split_keeps_focused_modules` was refreshed to pin the current `runtime_session_error_preserves_step_and_typed_source` behavior anchor and passed 1/1. A later broad `--lib session` recheck passed with 161 passed / 0 failed / 10 ignored after the remaining Vampire project-session real-backend behavior tests were gated under default features.

All Vampire project-session behavior tests that load and execute the authored ZrVM project now declare `#[cfg_attr(not(feature = "backend-zr-vm"), ignore = "...")]` with the same reason string. The standalone static source guard `vampire_project_session_wasd_axes_match_third_person_camera` stays default-runnable because it reads authored script text and does not initialize the VM backend. This keeps the no-fallback rule explicit: default builds skip unavailable real-backend execution instead of using a Rust Vampire compatibility backend.

Fresh 2026-06-25 Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover evidence (`Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover` / `runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred`): `dynamic_api/session/tests/helpers.rs` has been removed and the shared vampire project-session, HUD pixel, capture, diagnostics, and frame-request support owner now lives at `dynamic_api/session/tests/vampire_runtime_support.rs`. `dynamic_api/session/tests/mod.rs` mounts only `mod vampire_runtime_support;`, and `frame_diagnostics.rs`, `vampire_gameplay.rs`, `vampire_hud.rs`, and `vampire_menu.rs` import from that owner. `naming_boundary/runtime_15_m2/asset_dynamic.rs::runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name` pins the missing old file, the new owner/caller import shape, and the Runtime 15/status/docs anchors. Static validation covered rustfmt, old-path/source scans, docs/status anchor scan, whitespace scan, line-budget scan, and scoped diff hygiene. Cargo is deferred while active shared cargo/rustc lanes are running and is not claimed as passing.

## Runtime 15 M3 dynamic API session lock poison recovery

状态：`runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred`。

Runtime 15 E9/F2 now treats the dynamic session registry and per-session execution mutex as recoverable private runtime state. `session/registry/session_store.rs` privately owns `lock_registry()` for `SESSION_REGISTRY`, and `session_slot.rs` privately owns `lock_session()` for each stored `RuntimeDynamicSession`; the zero-behavior `registry/mod.rs` only projects the operations used by session callers.

`session/tests/lock_poison.rs::dynamic_api_session_registry_accessors_recover_poisoned_locks` poisons the registry through a `#[cfg(test)]` store-owner helper and poisons the session through the real `with_session` action-admission path. It then verifies that dispatch recovers and destroy removes the handle without exposing the registry map or session lock outside their owner modules. The Runtime 15 lock-poison guard mirrors the same contract into current status output and plan docs.

## Boundary Rules

The dynamic session may:

- validate ABI versions and handles before touching runtime state;
- adapt host ABI values into runtime framework DTOs;
- resolve runtime managers through the existing `CoreRuntime` handle path;
- run the runtime tick, optional render bridge, default level, project default-scene load, script scene hook installation, and camera-controller preview path.
- load startup script packages declared by `ProjectManifest.scripts` once the Script module and selected backend are active.

The dynamic session must not:

- duplicate module assembly rules owned by `zircon_runtime::builtin::runtime_modules`;
- retain module-load warning/error strings as runtime state; session creation consumes `RuntimeModuleLoadReport::fatal_messages()` only at the typed session-error boundary;
- expose new public Rust API from the dynamic ABI internals;
- encode editor authoring state as runtime session persistence;
- bypass `core::framework` DTOs with ad hoc dynamic-only event models;
- grow conversion, status, preview, host-request, or event helpers back into `session.rs`.
- add new dynamic API assertions back into a monolithic `tests.rs`; new coverage belongs in the matching `tests/<owner>.rs` module.

## Project Runtime Entry

`zircon_app` accepts `--project <path>` and `--project=<path>` for the `zircon_runtime` binary. The input may name either an existing project root or that root's `zircon-project.toml`. The app resolves aliases, junctions, SUBST drives, symbolic links, and Windows verbatim-path display form at its process boundary, normalizes a manifest input to its parent physical root, and encodes only that physical root anchor into the dynamic-session ABI `project_root` byte slice. Project-local scene, cache, and release paths remain normalized `RelPath` values and are joined to that anchor only at their owning filesystem boundary. The runtime ABI accepts the same root-or-manifest shape, resolves it into `ResolvedProjectPath` once, then opens it with `ProjectManager::open_resolved`; no downstream project service receives the caller's raw lexical path or reimplements Windows path compatibility.

Play-in-Editor supplies `--play-scene <relative-path>` and `--play-report-pipe <name>` through the
same `ZrRuntimeSessionConfigV3` startup carrier. The parser accepts equals and space forms, rejects
duplicate, blank, absolute, prefixed, or escaping scene values, and requires `--project`. The scene
is a current versioned `DynamicScene` snapshot under the project root; it is decoded before the
session's first level is selected. The report outlet emits ordered `starting`, `ready`,
`start-failed`, and `terminal` records through the editor-owned bounded child-output path, while
stdout and stderr diagnostics remain independently captured.

The `zircon_editor --project <path>` GUI entry follows the same rule. Its public `ProjectAuthority::open_project` boundary resolves the raw command-line input once, then transfers the resulting `ResolvedProjectPath` to `open_resolved_project`; its retained-host startup request uses the opened physical root while all visible text is projected through the resolver's display view. Project, asset, scene, and UI code must consume this resolved identity rather than add a Windows-only path branch.

When a project root is provided, `RuntimeProjectConfig` opens `zircon-project.toml`, reads `default_scene`, and calls `scene::load_level_asset(core, project_root, default_scene)`. That path scans/imports `project_root/assets`, loads the scene asset artifact, instantiates a `LevelSystem`, and then the session ticks that level before input `begin_frame`. The same config also reads `scripts.package_roots` and `scripts.startup_packages`, discovers `plugin.toml` packages under those roots, filters to the requested startup packages when listed, and loads them through `VmPluginManager`.

Runtime 10 now follows the Frameworks05 manager-service hard cut. `RuntimeProjectConfig::open_project_assets` obtains the versioned `asset_manager_handle(core)` and resolves it at the operation boundary with `resolve_manager_service(core, handle)` before calling `open_project(...)`. The deleted named `resolve_asset_manager` helper, Arc-holder wrapper, and `.shared()` conversion are not retained as aliases or compatibility paths.

Project-backed sessions create a `DynamicSceneAssetReloadQueue` from a `ProjectAssetManager` resolved through `project_asset_manager_handle(core)` at the same bounded use point. Optional navmesh loading follows the identical contract through `navigation_manager_handle(core)`. `RuntimeDynamicSession::tick_frame()` advances runtime time, drives the queue with `tick_into_level` against the current `LevelSystem`, records `scene.asset_reload.*` diagnostics for every reload frame report, and only then calls `LevelSystem::tick`; non-empty reload work is logged and cached as the last scene-asset reload frame report. Reload preparation or apply failures remain frame diagnostics instead of aborting the tick, and the queue never serializes or stores editor selection, viewport, gizmo, or inspector state.

Dynamic sessions install the script fixed-update and update scene hooks after module activation and before project scripts are loaded. The helper only registers missing hook ids, so first-party ZrVM language plugin hook registration can still own the same ids when that extension path is already present. This keeps project `script.bindings` active for the standalone dynamic runtime path without requiring the dynamic library to depend directly on external `zircon_plugins` crates.

Standalone keyboard events originate in `zircon_app::entry::runtime_entry_app::converters::keyboard`. Letter and digit keys are normalized to stable ASCII-style runtime codes before crossing the ABI; `session/input_events.rs` then preserves the logical key string so `gameplay.key_pressed("W")`, `A`, `S`, `D`, and `1`/`2`/`3` work in project scripts independent of platform scan-code hashing.

`vampire_project_session_w_key_moves_player_before_input_clear` locks the tick ordering for the standalone path: the dynamic session injects the W key, runs the project level tick while input is still visible to gameplay, and only then clears per-frame input state. This prevents the app ABI path from accepting keyboard events while starving project scripts of the same input during gameplay updates.

`vampire_project_session_auto_blood_bolt_damages_nearest_enemy` locks the combat path for the same standalone entry: one project-session tick must let the vampire auto-target the nearest enemy, reduce that enemy's scripted HP, write `vampire.action_state = 2`, and author `render.particle_sprites` with the base `blood_bolt` attack style. This path must execute through the authored ZR script and generic gameplay host APIs; runtime must not satisfy the test through a Rust Vampire fallback backend.

`vampire_project_session_enemy_behavior_tree_chases_player` locks the enemy AI path: one project-session tick must evaluate the authored enemy behavior tree, put enemy 20 into chase/run state with `vampire.action_state = 1` and `vampire.behavior_node = 31`, and move the enemy closer to the player through the navigation-preferred chase path. This keeps behavior-tree binding from becoming a manifest-only claim.

`vampire_project_session_starts_paused_until_start_button_click` locks the start-menu path: the first project tick writes `gameplay.menu_state.state = "start"` and `vampire.run_state = "start_menu"`, keeps the player and enemy transforms stable, accepts a left-click over the menu button, then advances to `vampire.run_state = "playing"` and clears the menu component. This prevents the standalone sample from beginning combat before the player presses Start Game.

`vampire_project_session_game_over_menu_retries_to_playing` locks the death/retry path: a lethal enemy contact writes `gameplay.menu_state.state = "game_over"` and `vampire.run_state = "game_over"`, then a Retry click resets player HP/position and re-enters gameplay. The session test can export `ZR_VAMPIRE_GAME_OVER_CAPTURE_PNG` from the same offscreen frame path used by runtime screenshots.

`vampire_project_session_writes_world_hud_for_scene_authored_enemies` locks the visible standalone gameplay path: project ticks must keep combat health in scene-following `render.world_hud_bars`, keep authored enemies stable, and avoid recreating `health_bar_fill` or `health_bar_back` scene roles. This prevents the dynamic executable from launching a static scene while tests only prove invisible script state.

`vampire_project_session_kill_xp_opens_level_up_and_choice_updates_hp` locks the kill reward and level-up path: a killed scripted enemy must award its XP before removal, open the level-up HUD prompt, accept digit-key choice input, and apply the max-HP evolution.

`vampire_project_session_pickups_apply_heal_shield_and_timed_buffs` locks the pickup path: nearby scripted pickup entities for heal, shield, attack, and haste must be consumed by the player update and reflected in `gameplay.hud_text`.

`vampire_project_session_buffed_attack_particles_use_buff_palette` locks the buff-driven attack VFX path: after attack, haste, and shield pickups are active, the next automatic attack must write a combined `blood_flame_haste_shield` particle payload, and the authored sprites must include red/orange blood flame, cyan/green haste, and blue shield accents for the renderer.

`vampire_project_session_chest_choice_grants_weapon_upgrade_from_digit_input` locks the rare chest path: collecting a chest opens the weapon-choice prompt, digit-key input chooses a weapon, and the HUD reports the upgraded inventory.

`vampire_project_session_boss_spawn_uses_five_minute_default_with_test_override` locks the boss spawn director without making tests wait five minutes. The runtime default remains `300.0` seconds; the test injects a player script property `boss_interval_seconds` to force the same timed path immediately.

`vampire_project_session_keeps_hud_after_extended_runtime` covers the same path after a longer simulation window. The current demo now transitions fatal contact into a menu-driven game-over state instead of deleting the player entity, which keeps the third-person camera target, world HUD data, and Retry flow stable for manual screenshots and playable smoke tests.

`vampire_project_session_capture_frame_draws_world_hud_bars` captures the runtime frame after project ticks and asserts that world-space health bars, shadow work, and particle data are present while screen-space combat HUD command count stays zero. It accepts `ZR_VAMPIRE_CAPTURE_WIDTH`, `ZR_VAMPIRE_CAPTURE_HEIGHT`, `ZR_VAMPIRE_CAPTURE_TICKS`, and `ZR_VAMPIRE_CAPTURE_PNG` so manual validation can export a deterministic gameplay PNG at the requested viewport and after a longer gameplay run without adding another test-only code path. The 2026-06-16 Release ZR VM validation passed after the runtime pipeline inserted the core `particle-render` scene pass before post-process terminal work and split world-HUD bars onto the no-depth-test particle overlay path.

When `current_ui_extract` sees a valid `gameplay.menu_state`, the menu overlay has priority over the gameplay HUD extract. This is intentional: Start and Game Over are modal runtime states, while the world HUD bars remain scene data. Left mouse release over the overlay button writes only the command string (`"start_game"` or `"retry_game"`) and leaves lifecycle ownership in the project script.

Dynamic API error statuses now carry the concrete session creation failure string across the ABI diagnostics slice instead of collapsing every startup error to `runtime dynamic API error`. That diagnostic payload is intentionally leaked for process lifetime because `ZrStatus` exposes only a borrowed `ZrByteSlice`; owned byte buffers remain reserved for APIs that include an explicit free callback. `RuntimeDynamicSession::new` also initializes the `runtime-dynamic` diagnostic log before project startup so frame/FPS diagnostics and startup failure context share the same runtime-owned diagnostic channel.

Runtime 15 F5 dynamic API session typed errors records `runtime_15_dynamic_api_session_typed_errors_static_passed_cargo_deferred`: `dynamic_api/session/error.rs` now owns `RuntimeDynamicSessionError` / `RuntimeDynamicSessionResult` and `RuntimeProjectError` / `RuntimeProjectResult`. Session construction, project root parsing, project asset/default scene/navmesh/script loading, scene hook registration, level ticking, render bridge submit/present/bind, and accessibility encode failures stay typed until `dynamic_api/session/status.rs::error_status(...)` converts them into the C ABI `ZrStatus` message. `review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary` locks `dynamic_api/session/error.rs`, `RuntimeDynamicSessionError::RenderBridgeStep`, `RuntimeProjectError::LoadDefaultScene`, the ABI status boundary, and the status-output anchors.

2026-06-27 editor layout validation exposed one remaining typed-error mismatch in the same boundary: `CoreRuntime::install_scene_runtime_hooks(...)` returns `RuntimeExtensionRegistryError`, so dynamic session startup now maps that step to `RuntimeDynamicSessionError::RuntimeExtensionRegistryStep` instead of incorrectly using `CoreStep`. Per-hook registration still uses the existing scene hook registration error path.

`examples/vampire` is the current acceptance fixture for this project-entry path. Its manifest declares `res://scenes/main.scene.toml`, a `scripts/vampire_game` ZrVM package, project-local model/material/shader/navmesh/terrain assets, and plugin selections for rendering, animation, navigation, glTF importing, texture importing, and ZrVM language runtime. The fixture now includes generated jungle terrain/foliage models, a real project texture at `res://textures/jungle_ground_albedo.png`, a real `TerrainAsset` at `res://terrain/jungle_clearing.terrain.toml` with a ready `.zmeta`, a multi-polygon baked jungle navmesh with authored height variation, a richer `default_pbr` shader that samples material maps and folds in shadow/reflection/detail-normal terms, a ground-light floor that prevents terrain from collapsing to black under fog, script-authored dynamic attack particles, and menu state components for Start/Game Over. The scene camera is authored in current EV100 exposure space (`exposure_ev100 = 9.2`); legacy near-zero multiplier-style values overexpose the PP-M3 resolve path and are not valid acceptance evidence for current captures. The scene's `Baked Jungle Terrain` entity intentionally carries both the visible mesh and the terrain component so the sample is terrain-backed instead of a decorative prop-only floor. Asset tests keep those pieces importable through the same project scan path used by the standalone runtime.

The dynamic executable links `zircon_app`, but the actual runtime session is created by `zircon_runtime.dll` through the ABI. App-side Rust plugin registration reports do not cross that dynamic boundary. The runtime therefore must keep enough default import capability for simple project startup. The default asset importer now includes built-in glTF/GLB, common image, and `.txt` text-data importers; linked plugin catalog importers can still override those matchers on static or embedded paths, but the dynamic runtime no longer depends on those app-linked registrations just to import the vampire GLB models, Kenney shared texture, or local license notes.

## Linked Plugin Session Composition

The editor product uses the process-linked runtime table and
`create_linked_runtime_session(...)` so Rust-owned `RuntimePluginRegistrationReport` values remain
in the same runtime image as the session registry. `session/linked_plugins.rs` is the composition
owner: it completes the project plugin manifest, delegates module ordering to
`runtime_modules_for_target_with_plugin_registration_reports`, merges selected extensions through
`RuntimePluginCatalog`, and rejects fatal diagnostics before `CoreRuntime` activation.

The session installs linked scene hooks after module activation and applies the merged extension
registry to the actual `LevelSystem` World before the first frame. Consequently plugin resources,
mirrored events, and runtime scene systems participate in the normal `LevelSystem::tick` path.
The dynamic-library ABI path still creates an empty linked-registration set. The application
loader requires the complete V6 table and rejects missing or incomplete runtime libraries.

The Rust-linked entry returns `RuntimeDynamicSessionError`, including a classified unknown-profile
variant and typed sources for core, project, render, extension, and operation failures. Public
linked-session callers do not receive a flattened `String` failure.

`linked_plugin_registration_ticks_and_drains_through_runtime_api` proves the generic path with a
report-owned mirrored event and runtime scene system: subscribe through the ABI, tick the real
session, then drain the emitted typed payload. The `zircon_app` product test adds the concrete
Navigation runtime/editor pair and verifies capability disable removes the consumer before the next
tick.

## Runtime API V6 Operation and World Sync Lifecycle

Current V6 owner inventory: `expected_source_file_count = 51`.

`submit_operation`, `poll_operation`, `harvest_operation`, `query_world`, `watch_world`,
`unwatch_world`, and `drain_world_invalidations` are required members of `ZrRuntimeApiV6`. The
app loader validates the complete V6 group and has no old-table fallback. `session/ffi.rs` owns
the private ABI entry functions, `session/operation.rs` owns operation ABI decode/encode, and
`session/world_sync.rs` owns runtime-world query/watch/drain routing. The parent `session.rs` is a
module/re-export façade; it does not expose `World` or editor view identities across the ABI.

`dynamic_runtime_api_boundary` tracks 51 source files, 10 function tables, the 24-field V6 table,
and `runtime_session_ffi_wrappers = 22/22`. The permanent guard remains
`runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts`.

## Validation

`zircon_runtime/src/dynamic_api/tests/` covers the exported function table, invalid ABI and handle paths, profile-control JSON validation, frame/accessibility request validation, session creation profile handling, host request encoding, accessibility fallback behavior, and input-event rejection paths.

The API table coverage includes `runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary`, which keeps all exported table entries behind `exports.rs` panic wrappers and requires `ZrStatusCode::Panic` rather than an unwind crossing the C ABI. The runtime-absorption coverage adds `runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge` as a cross-document architecture guard for the same boundary.

The 2026-06-21 `dynamic_runtime_api_boundary` structural mirror ties this module's Runtime 10 evidence to the ABI inventory, FFI wrapper table, headless/minimal lifecycle contract, loader failure-path guards, behavior-test anchors, runtime diagnostics profile-control chain, host-request payload ABI chain, UI pending gate, UI single-source contract guard, UI v2 contract synchronization, and pending Cargo gates. Markdown rendering is split into `dynamic_runtime_api_markdown.py`, while `dynamic_runtime_api_boundary.py` remains the 330-line audit/risk owner. Current static output reports `expected_source_file_count = 35`, `function_table_structs = 10/10`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `runtime_diagnostics_anchors = 15/15`, `missing_runtime_diagnostics_anchors = []`, `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_contract_duplicate_public_types = 0`, `ui_v2_contract_sync_anchors = 9/9`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `mirror_docs_guard_present = true`, and `risks = []`. The host-request chain now pins interface DTOs in `runtime_api/host_requests.rs`, runtime conversion in `dynamic_api/session/host_requests.rs`, dynamic API host-request tests, and app host routing through `runtime_entry_app/host_requests/{routing,cursor/request}.rs`. `runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending` records that Runtime 09's `v2-replacement-mainline` verdict is now mirrored into Runtime 10: interface `ui/v2` owns DTOs, runtime `ui/v2` consumes them, and `UiComponentApiVersion` remains the interface-owned version contract used by runtime validation. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps this module doc, Runtime 10, the runtime index, the M0 review, runtime-interface convergence, and the cdylib loader doc aligned with those counts. This remains static structure evidence; the `dynamic_api`, full app loader, and UI contract Cargo lanes remain pending.

The Runtime 10 absorption guard for this boundary is split under `runtime_absorption/dynamic_api_session/{shared,headless_profiles,event_split,test_owner_split,ffi_panic_boundary,runtime_diagnostics,ui_contract,v2_contract,mirror_docs}.rs`, with `dynamic_api_session.rs` kept as the mount file. Focused package validation passed on 2026-06-15 before the UI contract guard was added: `cargo test -p zircon_runtime --lib dynamic_api_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-dynamic-api-split-0615` ran 5 tests successfully with 4231 filtered out. Current standalone absorption validation passes 11/11 across the split guard modules. Concrete Runtime 09/10/15 and Frameworks 02 evidence now comes from numbered output archives, while status-table evidence comes from the current folder-backed child owners; route-only parent plans and retired aggregate test files are no longer treated as concrete evidence owners. A newly compiled full runtime test binary is still required before the broader `session`, `dynamic_api`, app-loader, or UI-contract gates can be promoted.

The 2026-06-19 dynamic session scene-asset reload integration and diagnostics projection were validated with rustfmt on `session.rs`, `session/project.rs`, `session/scene_asset_reload_diagnostics.rs`, and the dynamic-scene reload modules; conflict marker and trailing whitespace scans were clean; scoped `git diff --check` only reported existing LF/CRLF warnings; `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-dynamic-scene-asset-0619 --message-format short --color never` passed with existing warnings only. Focused behavior tests remain deferred under the implementation-first direction.

For architecture validation, the runtime structural audit should no longer list `zircon_runtime/src/dynamic_api/session.rs` under production large-file hotspots after this split. The audit also reports `dynamic_api_test_boundary`, which must keep the legacy `zircon_runtime/src/dynamic_api/tests.rs` absent, all 12 owner modules declared, and oversized test owner modules at zero. The current host-request test-owner split keeps drain/session-flow behavior in `host_requests.rs` at 98 lines and moves payload encode/free-token coverage into `host_request_payloads.rs` at 156 lines. The 2026-06-21 renderer split keeps the audit owner in `runtime_structure_audits/dynamic_api_test_boundary.py` at 89 lines and moves Markdown output into `runtime_structure_audits/dynamic_api_test_markdown.py` at 35 lines so the main architecture audit script remains an orchestration boundary instead of becoming another mixed large file.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split` / `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/artifact_render_diagnostics_splits/{artifact_cache_payload,render_product_diagnostics,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 35`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 scene/project guard child-owner split` / `runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/scene_project_splits/{scene_asset,project_io,dynamic_session_event,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 39`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters child-owner split` / `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/ecs_extract_counters/{query_change,extract_cache,asset_animation,frame_diagnostics,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 44`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split` / `runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/mirror_docs/{sources,performance_guard,source_inventory,audit_wiring,doc_mirrors,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 50`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split` / `runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/split_layout/{sources,route,source_inventory,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 54`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split` / `runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/split_layout/{route,source_inventory,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 57`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 submit-context split-layout guard folder-backed split` / `runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/submit_context/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The new guard is `runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_submit_context_guard_child_owner_split` remains as the historical wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split` / `runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/scene_project_splits/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 65`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The new guard is `runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_scene_project_guard_child_owner_split` remains as the historical wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split` / `runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/artifact_render_diagnostics_splits/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 69`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The new guard is `runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split` remains as the historical wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters split-layout guard folder-backed split` / `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 73`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The new guard is `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_split`, while `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split` remains as the historical wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split` / `runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/mirror_docs/sources/{assertions,load,views}.rs` to Runtime 07 audit input while keeping `performance_hotspots/owner_budget/mirror_docs/sources.rs` as the route/type owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 76`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The new guard is `runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget source refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split` / `runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/sources/load.rs` to Runtime 07 audit input while keeping `performance_hotspots/owner_budget/sources.rs` as the route/type owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 77`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The new guard is `runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget child-routes refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split` / `runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/child_routes/{submit_context,hotspot_inventory,scene_project,artifact_render_diagnostics,owner_budget}.rs` to Runtime 07 audit input while keeping `performance_hotspots/owner_budget/child_routes.rs` as the route owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 82`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The new guard is `runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget line-budgets refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split` / `runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/line_budgets/{root,artifact_render_diagnostics,hotspot_inventory,owner_budget,scene_project,submit_context}.rs` to Runtime 07 audit input while keeping `performance_hotspots/owner_budget/line_budgets.rs` as the route owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 88`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The new guard is `runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget split-layout route refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split` / `runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/split_layout/route/{parent_route,split_route,support_routes}.rs` to Runtime 07 audit input while keeping `performance_hotspots/owner_budget/split_layout/route.rs` as the route owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 91`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The new guard is `runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.
