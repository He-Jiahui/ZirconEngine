---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/handle.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/handle.rs
  - zircon_runtime/src/core/manager/mod.rs
implementation_files:
  - docs/assets-and-rendering/directory-project-asset-rendering.md
  - docs/editor-and-tooling/crate-boundary-audit-round-2.md
  - docs/zircon_runtime/asset/facade.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
---

# Frameworks06 G7 Asset Manager 文档 Owner 硬切

Status: completed
Date: 2026-07-17
Session: `frameworks06-g7-asset-manager-doc-hardcut-20260717`

## 完成项目

- 将 directory-project asset 文档从退役 editor-specific resolver owner 硬切到当前 `editor_asset_manager/handle.rs`；另外两份文档收口 runtime asset-manager owner，不把 editor 改动错误泛化到全部三份文档。
- 将退役 concrete-handle 与 asset-specific resolver 两套 owner 合并为唯一 `asset_manager/handle.rs`；没有保留旧路径、alias、shim 或兼容说明。
- 文档架构语义同步为 `ManagerServiceHandle<dyn AssetManager>`：跨域长期保存 versioned handle，在 use point 通过 `core::manager::resolve_manager_service` 解析，不再描述 concrete `AssetManagerHandle` 或 asset-specific resolver。

## 验证

- 修改前 focused docs violations：12（3 documents）。
- 修改后 `python tools/check_conventions.py --only docs --json`：所选三份文档 0 violations；全库 current-source 快照从 281/30 documents 收敛为 269/27 documents。全库仍为 RED，剩余主要属于活动中的 Sound、Editor preferences、Runtime plugin/ZrVM owner 迁移，不能由本批次冒充完成。
- 两个 canonical owner 文件均存在；三种退役机器路径在本批次文档中为 0；`git diff --check` 通过。
- 独立只读复审经一轮 Important/Minor 修复后为 **Critical 0 / Important 0 / Minor 0**；确认 current `core::manager` resolver 不被误删、旧 asset-specific resolver 标识归零、完成记录与里程碑边界准确。

## 里程碑判定

本 G7 批次完成；Frameworks06 M1 与计划 06 仍为 `in_progress`，直到剩余文档路径与真实 CI 证据全部关闭。
