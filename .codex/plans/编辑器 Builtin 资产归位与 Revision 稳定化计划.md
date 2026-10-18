# 编辑器 Builtin 资产归位与 Revision 稳定化计划

## Summary
- 按 2026-04-14 当前代码核对，用户看到的“资产计数每帧累加”实际不是 count，而是资产面板里显示的 runtime `resource revision`，即 `r####` 持续增长。
- 本次先做两条硬修复主线：
  - 把编辑器仍然直接引用 `dev/` 的图标资产迁到 `zircon_editor/assets/`，并纳入引擎 builtin 资产命名空间。
  - 把 editor 资产工作区的“纯 UI 状态变化”和“后端 catalog/resource 同步”彻底拆开，建立 revision 不因 UI 刷新而变化的契约。
- 架构默认采用：
  - 编辑器内置静态 UI 图标源目录：`zircon_editor/assets/icons/ionicons`
  - builtin locator 命名空间：`builtin://editor/icons/<file>.svg`
  - Slint 现阶段继续从 crate 本地 `assets/` 读编译期图片，不再直接指向 `dev/`；builtin registry 同步注册同一批图标，作为统一引擎 builtin 资产来源。

## Key Changes
- **Editor builtin 资产归位**
  - 补齐 `zircon_editor/assets/icons/ionicons` 中当前 `chrome.slint` 还从 `dev/ionicons.designerpack` 取用但缺失的图标文件。
  - 将 `zircon_editor/ui/workbench/chrome.slint` 的所有 `@image-url("../../../dev/...")` 替换为 crate 内 `assets/` 路径，禁止 editor UI 再直接依赖 `dev/`.
  - 在 `zircon_asset` / builtin 注册路径中新增 editor icon builtin 清单，统一暴露为 `builtin://editor/icons/...`，让“编辑器内置资产”在资源系统里也是一等公民，而不是只存在于 Slint 路径字符串里。
  - 增加守卫测试或扫描规则，确保 `zircon_editor/ui` 公共 UI 路径不再出现 `dev/` 资产引用。

- **Asset workspace 同步边界重构**
  - 把 `EditorEventEffect::AssetWorkspaceChanged` 拆成两类语义：
    - 纯 UI 状态变化：选择、搜索、kind filter、list/thumbnail、utility tab、引用跳转，只更新 `EditorState/AssetWorkspaceState` 和 presentation。
    - 后端数据变化：project open/save、asset/import/watch/catalog change、resource change，才允许触发 server snapshot 重同步。
  - 更新 `zircon_editor/src/editor_event/runtime.rs`、`event_bridge.rs`、`slint_callback_dispatch` 相关测试，去掉“资产 UI 交互一定触发 `sync_asset_workspace`”这一错误契约。
  - `SlintEditorHost` 增加 `resource_server.subscribe_resource_changes()` 消费链；catalog 变化和 resource 变化分别驱动局部刷新，不再一律全量 `catalog_snapshot + list_resources + details + preview visible refresh`。

- **Revision 稳定化契约**
  - 在 editor 层建立明确规则：无文件变化、无 importer 变化、无真实 resource record 变化时，UI tick、搜索、筛选、选中、布局切换都不得改变任何 `resource_revision`。
  - 在 asset/resource 层增加防守性 no-op 短路：
    - `sync_project_resources()` 在记录 ready-critical 字段未变化时跳过重复 `register_ready()`
    - 或等价地抽成 `register_ready_if_changed()` / `upsert_ready()` 契约，保证重复同步至多是性能开销，不得变成 revision 变化。
  - 保持 `watch.rs` 的 `.meta.toml` 过滤契约，并把 editor preview/meta 写回路径纳入完整回归，防止“preview/meta 写回 -> watcher -> reimport -> revision bump”再次回归。

- **Editor 预览刷新收口**
  - 维持现有“可见即刷新，否则保留缓存”的策略，但 preview 变化只更新 editor catalog/details/preview 呈现，不顺带拉 runtime resource revision。
  - `refresh_visible_asset_previews()` 只在真实可见 dirty 资产上触发；收到 `PreviewChanged` 时只刷新受影响资产详情或缩略图，不做全量 workspace 重建。

## Test Plan
- `zircon_editor`
  - 增加 UI 资源路径回归：`chrome.slint` 与 workbench UI 不再包含 `dev/` 资产路径。
  - 更新 callback/event bridge 测试：搜索、filter、view mode、utility tab、selection 不再产生 `sync_asset_workspace`。
  - 增加 host/workspace 回归：重复 tick 且无 backend 事件时，资产项显示的 `resource_revision` 保持不变。
  - 增加可见 preview 回归：dirty 资产首帧刷新一次后稳定，后续空闲帧不再持续触发同步。

- `zircon_asset`
  - 增加 builtin editor icon 注册测试：`builtin://editor/icons/...` 可列出、可查询、kind 正确。
  - 增加 no-op sync 回归：同一 project 在无源文件变化下重复 `sync_project_resources()` 不 bump revision。
  - 保留并强化现有 watcher/meta/revision 测试，覆盖 preview sidecar 写回不触发 source reimport loop。

- `zircon_resource`
  - 增加 `register_ready` 幂等回归：完全相同的 `ResourceRecord` 重复注册时 revision 不变、无 Updated 事件。
  - 如果新增 no-op API，则为其补齐 unchanged/changed 两条分支测试。

- 验收命令
  - `cargo test -p zircon_resource -p zircon_asset -p zircon_editor`
  - `cargo test --workspace --locked`

## Assumptions
- builtin editor 图标的权威源目录固定为 `zircon_editor/assets/`，不是仓库根 `dev/`。
- `builtin://editor/icons/...` 本轮先作为统一资源命名空间；Slint 直接显示仍走 crate 本地静态资源路径，不在本轮额外发明 Slint -> builtin locator 图片桥。
- `r####` 的稳定性优先级高于“任何 UI 操作都强制重拉后端快照”；asset workspace 的 UI 状态必须本地化。
- 本轮不重做 preview 系统，只修正其与 watcher/resource revision 的边界，确保 preview/meta 写回不会造成用户可见 revision 漂移。
