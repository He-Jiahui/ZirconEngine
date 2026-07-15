---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation/lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation/lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/lock_poison_rows.rs
  - docs/zircon_runtime/script/vm/zr_vm_project_backend.md
  - docs/zircon_runtime/structure/module-convention.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - cargo test -p zircon_plugin_zr_vm_language_runtime --locked
  - runtime_15_asset_render_input_lock_poison_guard_child_owner_split
  - runtime_15_m3_child_group_moved_lock_poison_rows_are_child_owned
  - runtime_15_lock_poison_status_row_data_owner_is_child_backed
  - runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries
  - runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus
  - review_f2_scene_eventbus_locks_recover_after_poison
  - Runtime15 plan-status standalone harness
doc_type: milestone-detail
---

# Runtime 15 ZrVM 具体实现 Owner 硬切状态收敛

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | ZrVM concrete backend owner hard cut and Runtime15 status convergence | `runtime_15_zr_vm_concrete_backend_owner_hard_cut_plugin_tests_18_passed` | 2026-07-14 | 具体 ZrVM backend、native binding、runtime/session lock、package/instance/host-module 实现与测试已归属 `zircon_plugins/zr_vm_language/runtime`。Runtime 删除旧具体实现 owner 后，Runtime15 同步删除具体 backend lock-poison 当前行及其状态/日期映射；父状态模块删除约 27 KB 手写 raw-string mirror，lock-poison test support 改为从七个 child owner 组合聚合视图。没有兼容 re-export、feature 转发或旧 Runtime 实现路径。 |

## 验证

- 协调器管理的插件默认特性门禁：18/18 通过，doc-tests 通过。
- 独立 Runtime15 structure harness：asset/render/input child-owner 1/1、moved lock-poison rows 1/1、lock-poison child row-data 9/9、Runtime-neutral script registry 1/1 通过。
- F2 Scene lock guard：结构与审查精确测试各 1/1 通过。守卫改为要求真实的 `let mut levels = self.lock_levels();` + `levels.insert(...)` 局部 guard owner，继续正向锁定 poison-safe helper，不改生产语义。
- 完整 current-source structure harness：1297/1303。ZrVM 与 Scene F2 相关失败为 0；六个剩余失败为 Render GPU context 897 行（同时触发两项局部守卫和 production global budget）、UI text owner、mesh command-list tests 515 行、core runtime resolution behavior tests 821 行。
- 优先 code-review-findings harness：79/80。F18 AssetManager 解析、F2 Scene lock、插件/导入器 DX、typed-error 与本切片 ZrVM hard-cut 相关项通过；仅剩 Render F16 编排预算。
- Runtime plan-status harness：48/48。Runtime06、08、13 的 `last_refined` 已对齐各自主计划和编号归档中既有的 2026-07-14 最新记录；仅修元数据，不改变计划生命周期状态。
- Runtime core-min scene 构建先前已越过全部被删除的 ZrVM owner，仅剩无关 Scene reflection 数字类型失败。

## 未关闭范围

Runtime15 整体仍为 `in_progress`。本记录只关闭 ZrVM 具体实现 owner 与 Runtime15 当前状态镜像的硬切，不替代完整 structure-convention、review-findings、Cargo 或 Runtime04 广义 asset gate。
