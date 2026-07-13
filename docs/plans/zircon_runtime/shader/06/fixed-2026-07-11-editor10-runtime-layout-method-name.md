---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: editor10-runtime-layout-method-name
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_editor/editor/10
related_code:
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/paths.rs
tests:
  - cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked
resolved_at: 2026-07-11
---


# Editor 10: runtime layout method name drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：EC-M3 current interactive viewer rebuild
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：Shader06 viewer 的完整构建被 ProjectPaths 硬切后的调用名漂移阻断，最低共享原因属于 Editor10 工程路径 owner。

## 失败现象与复现证据

`ProjectManager::open` 调用不存在的 `ProjectPaths::ensure_runtime_layout`，当前 viewer production build 在链接 Shader06 前以 `E0599` 失败；实际收敛实现名为 `ensure_derived_layout`。

## 最低共享层根因

ProjectPaths 已把可再生目录收敛到 `.zircon` derived layout，但调用方在并行硬切中使用了计划外名称。问题是单一调用名漂移，不是缺少第二套 layout。

## 架构修复验收

- 调用方使用既有 `ensure_derived_layout`。
- 资产根仍单独通过 `ensure_asset_roots` 创建。
- 当前 viewer production build 通过。

## 禁止临时方案

- 不新增 `ensure_runtime_layout` 兼容别名、shim 或第二套目录实现。
- 不回退 `.zircon` 派生根硬切。

## 修复结果与回传

- 根因：ProjectManager call site used a non-existent intermediate runtime-layout method name after the derived-root hard cutover
- 架构修复：Use the single existing ensure_derived_layout contract and retain separate asset-root creation without compatibility aliases
- 验证：Current zircon_shader_pbr_viewer production build passed; the executable reached Ready with Lakes 2K and produced a nonblank mirror-reflection window capture
- 回传：Shader06 interactive viewer build and startup verification resumed
