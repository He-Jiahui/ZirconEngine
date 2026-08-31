---
record_kind: implementation_status
status: source_complete_validation_pending
recorded_at: 2026-08-29
summary_slug: ffi-panic-boundary-current-source-inventory
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/ffi_panic_boundary.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
---

# Runtime10 FFI panic boundary current-source inventory

状态：`runtime_10_15_ffi_panic_boundary_current_source_inventory_static_passed_cargo_deferred`。

## 实现结果

- 新增全 Runtime production Rust 递归 inventory，复用 `rust_source_view::production_section`
  排除注释与 `cfg(test)` item，不按易漂移文件清单假定入口范围。
- Runtime 实际 C ABI 共 `42/42`：Dynamic API `27/27`，native loader host callbacks
  `15/15`。新增或迁移入口必须落入明确 owner 分类，并在函数边界调用该 owner 的 panic
  guard；未分类 owner 会直接失败。
- export product host 是 Rust raw-string 生成模板，不与 Runtime 实际 ABI 重复计数；其 C/JNI
  exports 单独审计 `13/13`，全部调用 `zircon_export_ffi_guard`。
- Dynamic API getter 的 pointer/null panic projection、status-return wrappers、public/private native
  host callbacks 和 `NativePluginOutputSinkV4.write` 保持各自 typed ABI 结果；没有新增统一返回
  类型、compatibility facade、alias 或跨 ABI 状态转换。

## 当前证据

- Rust 1.94.1 直接编译 current-source test harness 并执行 `2/2` GREEN：Runtime production
  inventory `1/1`，generated export/JNI inventory `1/1`。
- `rustfmt +1.94.1 --edition 2021 --check` 通过；新 leaf 为 154 行，尾随空白为 0。
- 本记录只证明 current-source 分类和 guard 结构。受管 `structure_convention` / Dynamic API /
  native loader / export product Cargo gates 仍 pending，未声明 DLL、JNI 或产品动态验收完成。

## 后续约束

任何新增 `extern "C"` / `extern "system"` 入口必须同步更新 inventory count 和 owner 分类；禁止
通过 wildcard、跳过目录、字符串白名单或 `cfg(test)` bypass 隐藏 production ABI。受管验证通过前
Runtime10/15 milestone 继续保持 `in_progress`，不触发 commit 或企微里程碑通知。
