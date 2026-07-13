---
related_code:
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input/input_script.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries
  - current structure-convention standalone harness
doc_type: milestone-detail
---

# Runtime 15 Script HostRegistry 结构守卫同步

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | HostRegistry lock-poison guard follows generational-slot hard cutover | `completed-current-structure-guard-aligned` | 2026-07-14 | 当前 ZrVM owner 已将 `HostRegistry` 从 `HashMap<HostHandle, HostCapabilityRecord>` 硬切为 `HostRegistryState { slots, free_slots }`，但 Runtime15 守卫仍要求旧 `lock_handles`/`HashMap` 字面，fresh structure harness 因此为 1303/1304。守卫只同步到真实 `lock_state() -> MutexGuard<'_, HostRegistryState>` 及读写调用锚点，继续正向要求 poison recovery 与 `host_registry_accessors_recover_poisoned_handle_lock`；未修改活跃 ZrVM 生产代码、未恢复旧容器或兼容路径。 |

## 验收

- 精确守卫须由 fresh current-source harness 通过 1/1。
- 完整 structure-convention 须由同一 fresh harness 通过 1304/1304；代码审查与计划状态分别维持 80/80、48/48。
