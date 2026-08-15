---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-08-01
summary_slug: wgpu-render-framework-state-accessor-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset/set_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile/set_quality_profile.rs
tests:
  - cargo test -p zircon_runtime --lib runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector --locked
---

# WgpuRenderFramework state accessor drift

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：current `zircon_runtime` lib-test compile blocker review
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：Performance01 的受管 Picking/Runtime15 验收编译到了 Render17 所属 render-framework 边界，并发现硬切后遗留的私有状态字段访问。

## 失败现象与复现证据

受管 job `684a8cf18953475fb12d40c4a824108a` / run `f5c8077826054fc590d004e20c66e507` 在 `zircon_runtime` lib-test 编译中报告六个 `E0609`：一处生产查询和五处同模块测试仍访问已删除的 `WgpuRenderFramework.state`。当前类型已经通过 `lock_state()` 统一封装 poison recovery；编译器建议的 `framework.core.state` 会绕过该边界，不能采用。

## 最低共享层根因

`WgpuRenderFramework` 状态存储迁入 `core` 并增加 `lock_state()` 后，四个相邻模块没有作为同一迁移批次更新；生产与测试因此同时保留了旧字段形状。

## 架构修复验收

- 四个相关文件不再直接访问 `framework.state`。
- 六处调用统一通过 `framework.lock_state()`，不暴露或绕过 `core.state`。
- 固定 Rust 1.94.1 rustfmt 与 scoped diff check 通过。
- 新的受管 `zircon_runtime` lib-test 编译不再在这四个文件报告 `E0609`；若仍被其他 owner 的错误阻塞，保留准确的 0-test/foreign-blocker 证据。

## 禁止临时方案

不得把 `core.state` 改为更宽可见性，不得在调用点重复 `.lock().unwrap_or_else(...)`，也不得删除覆盖 invalid executor 与 debug snapshot 生命周期的测试来规避编译。

## 修复结果与回传

已把一处生产查询和五处测试访问切到 canonical `lock_state()`。固定 Rust 1.94.1 rustfmt、scoped `git diff --check` 均通过；四个文件的 `framework.state` 直接访问计数为 0，`framework.lock_state()` 当前 occurrence 计数为 10。此次修复后的 current-source Cargo 验证尚未执行；在有真实 Cargo 结果前保持 `open`。
