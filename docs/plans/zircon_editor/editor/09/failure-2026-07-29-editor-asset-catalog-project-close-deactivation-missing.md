---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: editor-asset-catalog-project-close-deactivation-missing
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/api.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/editor_asset_state.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/host.rs
tests:
  - cargo test -p zircon_editor --lib --locked editor_asset_manager
  - cargo test -p zircon_editor --lib --locked document
---

# Editor09: Project close leaves the editor asset catalog active

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：document lifecycle typed-message producer
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：runtime `AssetManager::close_project` 可以退役 project generation，但 editor asset catalog、preview state 与 source-sync generation 属于 Editor09 的单一投影 owner；Document lifecycle 不得直接清空其内部状态。

## 失败现象与复现证据

`DefaultEditorAssetManager::refresh_from_runtime_project` 在 runtime `ProjectAssetManager` 没有活动 project 时直接返回 `Ok(())`，没有撤销 `EditorAssetState` 内的 project、catalog、locator/UUID 索引或 preview 状态。因此任何 Editor01 close producer 即使正确调用 runtime close，仍会让资产浏览器继续读取上一工程的 catalog。

`EditorUiHost::restart_ui_asset_workspace_watcher` 已能在无项目时释放 watcher 并转换 refresh pipeline，但目前没有 project-close 编排调用它；只关闭 UI tab、切换 welcome 页面或忽略 refresh 都不能构成关闭事实。

## 最低共享层根因

Editor09 缺少 generation-aware 的 editor asset projection deactivation：runtime project 从 `Some` 转为 `None` 时，catalog/preview/source-sync state 必须以新的空 generation 原子替换，并以 canonical editor asset change stream 通知消费者。

## 架构修复验收

- `EditorAssetManager` 提供唯一的 runtime-project deactivation/refresh 语义；无活动 project 不得保留旧 catalog、project root、locator/UUID 索引、preview cache 或在途 source-sync generation。
- 关闭必须发布一个新的空 catalog generation，旧 preview/sync completion 不能在切换后回写；不可用 `Ok(())` 保留旧投影，也不可提供旧 catalog fallback。
- Editor01 仅在 runtime `close_project` 返回 committed root 后编排 Editor09 deactivation、UI asset watcher transition，并发布一次 `DocumentMessage::Closed`。
- 回跑 Editor09 asset manager focused tests，再上行回跑 Editor01 document lifecycle 与 project-close producer tests。

## 禁止临时方案

- 不得由 DocumentLifecycleAuthority、asset browser view、welcome page 或 retained host 直接写 `EditorAssetState`。
- 不得把“没有 runtime project”解释为保留最近 catalog 的成功 refresh，或用空白 UI 遮挡旧数据。
- 不得保留旧 project catalog、preview 或 source-sync completion 作为 close 后 fallback。

## 产出记录与时间

| 日期 | 状态 | 产出与证据 |
| --- | --- | --- |
| 2026-07-29 | open | 当前源码检查确认 runtime project 为 `None` 时 `DefaultEditorAssetManager::refresh_from_runtime_project` 直接返回成功而不清空投影；该最低层缺口已从 Editor01 document producer 移交 Editor09。 |

## 修复结果与回传

Open state: `待修复`。Editor01 不会在该投影仍可暴露旧工程资产时发布 project close 的结构性 document 事件；Editor09 完成 source-bound 验证后，将 canonical artifact 回传至 Editor01 子计划并改名为 `fixed-*`。
