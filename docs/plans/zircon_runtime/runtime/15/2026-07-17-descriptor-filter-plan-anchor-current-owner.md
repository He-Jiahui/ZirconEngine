---
record_kind: current_evidence_owner
status: current
created_at: 2026-07-17
plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
plan_sources:
  - docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
implementation_files:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/script_host.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
tests:
  - runtime_15_texture_descriptor_settings_parser_is_child_owner
  - runtime_15_script_host_value_descriptors_do_not_suppress_dead_code
---

# Runtime15 descriptor-filter current anchor owner

本 child record 是两组历史验收锚的唯一 current owner。父计划、runtime index 与两个 priority plans 不再复制这些完整 tuple；历史长正文仅保留在 archive evidence owner。

- `Runtime 15 M4 texture descriptor settings parser owner split` | `runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred` | `2026-06-24` | `asset/assets/texture/descriptor.rs` | `asset/assets/texture/descriptor/settings.rs` | `runtime_15_texture_descriptor_settings_parser_is_child_owner`
- `Runtime 15 F12 script host value descriptor dead-code cleanup` | `runtime_15_script_host_value_descriptors_coremin_check_passed` | `2026-06-22` | `script/vm/host/builtin_host_modules.rs` | `docs/zircon_runtime/script/vm/host/function_ledger.md` | `runtime_15_script_host_value_descriptors_do_not_suppress_dead_code`

Archive evidence：`docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md` 的 texture row（line 371）与 script-host row（line 431）。
