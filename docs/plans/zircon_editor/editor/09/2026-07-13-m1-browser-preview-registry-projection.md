---
status: completed
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
milestone: M1
slice: 1.3
related_code:
  - zircon_editor/src/core/asset/type_registry/context_command.rs
  - zircon_editor/src/core/asset/type_registry/contribution.rs
  - zircon_editor/src/core/asset/type_registry/definition.rs
  - zircon_editor/src/core/asset/type_registry/registry.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/editor_asset_manager/
  - zircon_editor/src/ui/workbench/snapshot/asset/
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
tests:
  - zircon_editor/src/tests/editor_asset_type_registry/consumer_projection.rs
  - zircon_editor/src/tests/editor_asset_type_registry/materialization.rs
  - zircon_editor/src/tests/editor_event/runtime/integration.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/tests.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources.rs
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
---

# Editor09 M1.3 Browser、Preview 与操作分派注册表投影产出

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| M1 | 1.3 Browser/Activity/preview/open/create/context registry projection | `COMPLETED` | 2026-07-13 | 新增 typed context command contribution 与 owner 冲突拒绝；Browser、Assets Activity、selection、reference、subasset 统一消费 materialized registry snapshot；preview provider/palette 从 canonical definition 读取；open/create/context 进入 registry + command host；删除 editor sidecar、后缀打开、UI kind helper、adapter 旧命名和 asset-details toolkit fallback。Windows 最新 lib-test 编译成功；focused 22/22、open 2/2、Browser 41/41、Activity 9/9、workspace 4/4、reference drag 9/9、UI Asset Authoring plugin locked 2/2。 |

### 实现摘要

- `AssetContextCommandDescriptor` 进入 folder-backed type registry；contribution、definition 与
  materialization 同时持有 context command，空 id/display、缺少 operation 和跨 owner 重复 id 都返回
  typed error，不静默覆盖。
- host 每次从 enabled capabilities + plugin contributions materialize 单一 registry，并把 presentation、
  toolkit、creation template 与 context command 投影到工作台 snapshot。Asset Browser 四类布局、Assets
  Activity、引用和子资产行只读投影值，不在 UI 层重新按 `ResourceKind` 查询显示名/badge/icon。
- `invoke_asset_creation_template` 与 `invoke_asset_context_command` 先从 materialized definition 定位
  descriptor，再经正常 `EditorOperationInvocation` 进入 command host；调用参数包含 typed asset type、
  descriptor id 与目标 folder/asset locator。
- `OpenAsset` 只接受 catalog 已索引类型；toolkit open operation 必须存在于 command registry 且通过
  `when` 预检，随后才打开 descriptor view。真实存在的 `.zui` 若未入索引也不会按后缀猜测编辑器。
- `EditorAssetDetailsRecord` 不再复制 asset type/toolkit 字段；selection toolkit 只由 enabled registry
  projection 写入。`adapter_key`、`AssetBrowserAdapter*`、`No adapter` 全部硬切为 toolkit 术语。
- preview artifact 生成从 `ThumbnailProviderDescriptor` 读取 source image/icon/operation/placeholder palette；
  `EditorAssetMetaDocument`、`editor_meta_path_for_source` 与 `*.editor.meta.toml` 已删除，`.zmeta` 是唯一
  sidecar owner。
- UI Asset Authoring plugin 通过一个 `AssetTypeContribution` 同时贡献 UI type toolkit、creation
  templates 与 context command；插件清单直接声明 `zircon_runtime_interface`，锁文件同步到当前清单。

### 红绿验证与证据

- 旧集成测试最初仍假定 `.zui` 后缀即可打开，最新实现下 0/2 失败；测试被硬切为“索引 + registry
  toolkit 成功”和“后缀单独拒绝”，最终 2/2 通过。没有恢复 suffix fallback。
- UI Asset Authoring plugin 首次严格 `--locked` 因新直接依赖未进入插件锁文件而拒绝；离线同步当前
  workspace 清单后，再次严格 `--locked` 通过 2/2。锁刷新只把
  `zircon_runtime_interface` 从已移除该依赖的 native-window editor 条目迁移到实际声明它的 UI Asset
  Authoring editor 条目。
- Windows 受管 Cargo job `d271d4b27d2b4d6790936249d8fc5b8c`：
  `cargo test -p zircon_editor --lib --no-run --locked --jobs 1`，退出码 0；最新二进制：
  `.codex/tmp/zircon_editor-editor09-m1-3-full-snapshot-projection-r6-20260713.exe`。
- 最新二进制：`editor_asset_type_registry` 22 passed；`asset_open_event_` 2 passed；
  `ui::layouts::views::asset_browser::` 41 passed；`tests::ui::assets_activity::` 9 passed；
  `tests::editing::asset_workspace::` 4 passed；reference/asset drag 9 passed，均 0 failed。
- Windows 受管 Cargo job `cc1c90534a2f40abb5e85698d15349f6`：
  `cargo test --manifest-path zircon_plugins/Cargo.toml --locked
  -p zircon_plugin_ui_asset_authoring_editor --jobs 1`，2 passed、0 failed；日志：
  `.codex/tmp/editor09-m1-3-ui-asset-authoring-plugin-locked-final-20260713.log`。
- scoped `rustfmt --edition 2021` 与 `git diff --check` 通过。生产 Rust 静态扫描中
  `adapter_key`、`AssetBrowserAdapter`、`No adapter`、`EditorAssetMetaDocument`、
  `.editor.meta.toml`、`resource_kind_label` 均为零命中；Browser/Activity 旧 type-label helper
  守卫通过。
- 全仓 plan-output audit 未报告 Editor09 记录位置/提示语问题，但被 5 个其他计划的既有格式故障阻断；
  已按最低所有权分别转交 Editor01、EditorUI01、EditorUI10（含 `editor_ui/index.md`）与 EditorUI11。
  新建 handoff 在 failure validator 中无新增诊断；validator 仍报告 33 个其他既有 handoff 格式/链接
  问题，本切片不跨 owner 修复。

### 边界与后续

M1.3 完成不代表 Editor09 M1 完成。M1.4 仍需定义 `res://` 可写与
package/builtin/library/derived/transient 只读 source authority，并在 command `when` 与实际 dispatch
两层拒绝写操作；完成后才能执行 M1 全量 `zircon_editor --lib --locked` 验收。
