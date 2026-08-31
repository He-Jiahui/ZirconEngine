---
status: source-complete-managed-validation-pending
created_at: 2026-08-29
implementation_status: duplicate-document-authority-removed
managed_validation_status: pending
related_code:
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/layout/manager
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/host/layout_hosts
  - zircon_editor/fixtures/workbench/default-layout.json
---

# Layout07 content workspace single-authority hard cut

## 架构裁决

current source 曾让 `MainHostPageLayout::WorkbenchPage.document_workspace` 与 `ActivityWindowLayout.content_workspace` 同时持有中心文档树。默认预设分别构造两份树，布局命令、host placement、snapshot 与 preset 又主要读写 main-page 副本，违反“window 是承载唯一事实源”的 Layout07 合同，也使后续 placement index 与原子布局事务无法定义唯一 generation owner。

本切片采用 activity-window authority：

- `MainHostPageLayout::WorkbenchPage` 只保留 `id/title/activity_window`，page 是导航身份与 window 外键，不再拥有内容树。
- `ActivityWindowLayout.content_workspace` 是页签、split、active tab 与文档 placement 的唯一可变事实源。
- `WorkbenchLayout::{content_workspace_for_page, content_workspace_for_page_mut}` 统一解析 page -> activity window -> content workspace；manager、preset、host、snapshot 与 retained tab drag 不再直接解构 page 内容。
- `ensure_workbench_content_workspace` 同时确保 page 与其 activity window 存在，修复路径不会生成孤立 page 或第二份默认树。
- 旧 JSON `document_workspace` 字段由 `deny_unknown_fields` 直接拒绝；没有 alias、双写、fallback 或兼容迁移。

该方向与 UE docking 的 persistent tab stack owner 一致：page/tab-manager identity 引用 dock/window 承载，持久布局树不在导航 page 与 window 中复制。完整布局 transaction、stable stack/node id、placement index 与 generation 仍按 `docs/plans/performance/01/2026-08-19-editor-ui-workbench-layout-atomic-transaction-location-index-architecture-review.md` 后续实施，本切片不以局部 rollback 代替该结构。

## 完成范围

- 删除 main-page `DocumentNode` 字段和 `document_workspace_mut` 旧 accessor。
- 迁移 attach/detach/focus/split/restore、preset capture/apply、host active/placement/repair、snapshot build 与 tab-drag host resolution。
- 默认 Material/Fyrox/JetBrains/Unreal preset 只构造 activity-window 文档树；fixture 的 Scene/Game tabs 只存在于 `content_workspace`。
- 更新 host/workbench/project/view-model 测试夹具；严格 payload 测试新增旧字段拒绝，并把深层未知字段检查切到 current `content_workspace`。

## 验证证据

- current-source 递归扫描：`document_workspace:` 字段声明/构造 `0`，`document_workspace_mut` `0`；canonical page accessor 调用 `24`。
- `rustfmt --edition 2021 --check`：本切片 Rust 文件通过。
- `python tools/tests/test_editor06_workbench_layout_single_source.py`：`5/5`，`19.653s`。
- `python tools/tests/test_editor11_workbench_layout_payload_strictness.py`：`5/5`，`0.011s`。
- `python -m json.tool zircon_editor/fixtures/workbench/default-layout.json`：通过。
- scoped `git diff --check`：通过，仅报告工作树既有 LF/CRLF 提示。
- managed Rust 行为门尚未取得终态，不声明 Cargo GREEN、里程碑完成、性能收益或可提交。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-29 18:13 +08:00 | `source-complete / duplicate-authority-removed / static-verified / managed-validation-pending` | 删除 `MainHostPageLayout::WorkbenchPage.document_workspace`，以 `ActivityWindowLayout.content_workspace` 作为中心文档唯一事实源；迁移 manager/preset/host/snapshot/tab-drag 与全部 tracked 构造夹具，旧 JSON 字段严格拒绝。旧字段构造与旧 mut accessor 扫描均为 0，canonical accessor 调用 24；rustfmt、Editor06 5/5、Editor11 5/5、JSON 与 scoped diff-check 通过。managed Cargo、独立复核、协调器提交与企微量化通知仍待取得。 |
