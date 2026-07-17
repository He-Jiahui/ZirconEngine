---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: project-open-repeated-manager-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_load_from_path.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_save_to_path.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document_welcome_pane_snapshot.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_assets.rs
  - zircon_editor/src/core/project
  - zircon_runtime/src/asset/project
tests:
  - project open manager/manifest/scan build-count regression
  - 1/100/1000 locator resolution project-manager open-count regression
  - open/save/import/workspace/watcher parity matrix
---

# Editor10：project open 与 locator resolution 重复 manager/scan

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：Editor core project + `ui/host/startup` 7/7 + `ui/host/project_access.rs` 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：project generation、manifest、asset inventory、workspace 与 locator mapping 必须由 Editor10/Runtime asset project 的共同稳定 owner 提供。

## 失败现象与复现证据

`EditorUiHost::open_project` 先由 `ProjectAuthority::open_project` canonicalize/parse manifest，再调用 runtime `AssetManager::open_project` 与 editor asset refresh，最后 `EditorProjectDocument::load_from_path` 又新建 `ProjectManager`、`scan_and_import`、clone manifest 并加载 scene。一次 MVP open 因此存在多个 project owner/manifest parse/inventory scan 候选。`resolve_ui_asset_path` 与 `resolve_asset_locator_path` 还会为每次 locator 新建 `ProjectManager::open`，批量 UI asset/session restore 会放大为 N 次 project open。

Workbench 逐文件审查又确认：welcome snapshot 每次调用 `ProjectAuthority::probe_draft`，把 canonicalize/stat/manifest 验证放入普通 UI projection；project save 可能在序列化前重新 `scan_and_import`；layout preset load/save/list 反复打开 project manager，list 逐文件读取解析全部 preset。共享 project generation 还必须覆盖这些 consumer，而不只优化首次 open。

## 最低共享层根因

open flow 只在层间传 path/String，不传递一个 generation-bound `OpenedProject`/`ProjectManager` snapshot；authority、runtime asset manager、editor document 与 path helper 各自重新建立 project truth。

## 架构修复验收

- 一次 open 构造一个 canonical project generation，manifest parse 与 source inventory/scan 各至多一次；runtime asset manager、editor document、workspace/watcher 共用结果。
- locator batch resolution 复用当前 project generation 的 path index，不为每个 locator reopen project。
- save/reload/import/hot-change 精确生成新 generation；旧 snapshot lifetime 安全且不会读到半提交状态。
- welcome/preset 的稳定 snapshot 不执行文件 I/O；draft/path/preset-dir generation 变化才触发 worker probe/list，退出与显式 save 有 generation-safe flush。
- 1/100/1000 assets 记录 project-manager open/manifest parse/tree enumeration 次数、bytes-read 与 wall；scene/workspace/import diagnostics 和错误顺序等价。

## 禁止临时方案

- 不得只缓存 root `PathBuf` 而继续重建 manifest/inventory。
- 不得绕过 `ProjectAuthority` 的 canonical/symlink/reparse validation。
- 不得让 runtime/editor 各持一份可独立变更的 manifest truth。

## 修复结果与回传

Open state: `待 Editor10/Runtime asset project 建立 generation-bound opened-project snapshot，并回传 open/scan/locator build-count 与 I/O trace`。
