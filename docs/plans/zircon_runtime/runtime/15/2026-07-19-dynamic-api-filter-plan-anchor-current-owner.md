---
record_kind: current_evidence_owner
status: current
created_at: 2026-07-19
plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
plan_sources:
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_profile.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_shader_prewarm_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic_dynamic_api_vampire.rs
tests:
  - runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry
  - runtime_15_dynamic_api_session_profile_is_child_owner
  - runtime_15_dynamic_api_session_registry_is_child_owner
  - runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner
  - runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner
---

# Runtime15 dynamic-API filter current anchor owner

本 child record 是 Runtime10 `dynamic_api` 上行过滤器带入的五组 Runtime15 历史验收锚的唯一 current owner。Runtime15 父计划、runtime index 与两个 priority plans 保持概览/路由职责，不再复制完整 tuple；历史长正文只保留在 archive evidence owner。

- `Runtime 15 M3 dynamic API session lock poison recovery` | `runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred` | `2026-06-24` | `dynamic_api/session.rs` | `dynamic_api/session/tests/lock_poison.rs` | `runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry`
- `Runtime 15 M4 dynamic API session profile owner split` | `runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred` | `2026-06-24` | `dynamic_api/session.rs` | `dynamic_api/session/profile.rs` | `runtime_15_dynamic_api_session_profile_is_child_owner`
- `Runtime 15 M4 dynamic API session registry owner split` | `runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred` | `2026-06-24` | `dynamic_api/session.rs` | `dynamic_api/session/registry/session_store.rs` | `runtime_15_dynamic_api_session_registry_is_child_owner`
- `Runtime 15 M4 dynamic API shader prewarm tests owner split` | `runtime_15_dynamic_api_shader_prewarm_tests_owner_split_static_passed_cargo_deferred` | `2026-07-01` | `dynamic_api/shader_prewarm.rs` | `dynamic_api/shader_prewarm/tests.rs` | `runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner`
- `Runtime 15 M3 asset-dynamic dynamic-API vampire guard child-owner split` | `runtime_15_asset_dynamic_dynamic_api_vampire_guard_child_owner_split_static_passed_cargo_deferred` | `2026-06-30` | `tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs` | `tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs` | `runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner`

Archive evidence：`docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md`。生产/module/status owners 继续由各自源码、模块文档和 status row/map 校验；本记录只收敛 current plan evidence 路由，不复制生产实现。
