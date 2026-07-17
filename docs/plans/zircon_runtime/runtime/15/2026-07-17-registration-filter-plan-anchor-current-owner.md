---
record_kind: current_evidence_owner
status: current
created_at: 2026-07-17
plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
plan_sources:
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate/registration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_registration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
tests:
  - runtime_15_provider_registration_uses_shared_owner
  - runtime_15_core_runtime_registration_structure_tests_are_folder_backed
---

# Runtime15 registration-filter current anchor owner

本 child record 是 `registration` 过滤器带入的两组历史验收锚的唯一 current owner。父计划、runtime index 与两个 priority plans 不再复制完整 tuple；历史长正文只保留在 archive evidence owner。

- `Runtime 15 F13 provider registration shared owner` | `runtime_15_provider_registration_shared_owner_coremin_check_passed` | `2026-06-22` | `graphics/runtime_provider/registration.rs` | `docs/zircon_runtime/graphics/runtime_provider/registration.md` | `runtime_15_provider_registration_uses_shared_owner`
- `Runtime 15 M3 core runtime registration structure owner split` | `runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred` | `2026-06-24` | `core/runtime/tests/registration/structure/mod.rs` | `core/runtime/tests/registration/structure/service_count_paths.rs` | `core/runtime/tests/registration/structure/service_list_caches.rs` | `runtime_15_core_runtime_registration_structure_tests_are_folder_backed`

Archive evidence：`docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md` 的 provider-registration row（line 503）与 core-runtime-registration row（line 681）。
