# 编辑器资源管理器双模式 UI 接线计划

## Summary
- 用 `EditorAssetManager -> EditorAssetServer -> zircon_editor asset workspace snapshot -> presenter` 的单一路径替换当前 `asset_entries: Vec<String>` 占位链路。
- UI 固定为双模式：
  - `Assets` 活动面板采用 Unity-first：左目录树 + 右内容区 + 底部紧凑预览/引用条。
  - `AssetBrowser` 独占主窗口采用 Unreal-first：顶部工具条 + 左 Sources tree + 中央 Content Browser + 右 Details rail + 底部 utility tabs。
- 两个模式共享同一选择、搜索、过滤和引用数据，但各自保存自己的显示偏好；`Project` 视图不再复用资产列表，而是改成项目概览和入口页。
- 资产权威仍在 `zircon_asset::EditorAssetManager`；`zircon_editor` 只维护 UI 会话状态，不拥有资产数据库，也不解析 locator 字符串拼树。

## Public Interfaces
- `zircon_server::EditorAssetServer` 扩展为真正的编辑器资产 façade，至少公开：
  - `catalog_snapshot()`，返回稳定文件夹树、资产记录、根目录信息和 catalog revision。
  - `asset_details(uuid)`，返回单资产的元数据、预览信息、直接引用、被引用、诊断。
  - `subscribe_editor_asset_changes()`，覆盖 catalog 更新、preview 状态变更、reference graph 变更。
- 新增或固定 editor-facing record：
  - `EditorAssetFolderRecord { folder_id, parent_folder_id, locator_prefix, display_name, child_folder_ids, direct_asset_uuids, recursive_asset_count }`
  - `EditorAssetCatalogRecord` 在现有 `uuid/id/locator/kind/preview_state/...` 基础上补齐 `display_name`、`file_name`、`extension`、`source_mtime_unix_ms`、`source_hash`、`dirty`、`diagnostics`
  - `EditorAssetDetailsRecord` 聚合 `catalog record + direct references + referenced by + preview artifact path + meta path + adapter key`
  - `EditorAssetChangeRecord` 明确 `CatalogChanged | PreviewChanged | ReferenceChanged`
- `zircon_editor` 新增结构化快照和状态，而不是继续散落 `asset_entries`：
  - `AssetWorkspaceSnapshot`
  - `AssetFolderSnapshot`
  - `AssetItemSnapshot`
  - `AssetSelectionSnapshot`
  - `AssetReferenceSnapshot`
  - `AssetSurfaceMode::{Activity, Explorer}`
  - `AssetViewMode::{List, Thumbnail}`
  - `AssetUtilityTab::{Preview, References, Metadata, Plugins}`
- `Project` 视图和 `Assets`/`AssetBrowser` 不再共享同一个 `ProjectAssetsPane` 数据模型；资产浏览只吃 `AssetWorkspaceSnapshot`。

## Implementation Changes
- 后端与 façade：
  - 在 `zircon_asset::EditorAssetManager` 内从 catalog 派生稳定 folder tree；folder id 固定为规范化 `res://` 文件夹前缀，selection 由 folder id 和 asset uuid 稳定恢复，rename/move 只改 locator 不改 uuid。
  - `EditorAssetServer` 负责返回 tree/detail/change 三类数据；`zircon_editor` 只做 façade 聚合，不自行解析 locator 构树，不自行拼 references。
  - `ResourceServer` 继续作为运行时状态来源；editor host 在构建 `AssetWorkspaceSnapshot` 时按 locator 关联 `resource state/revision`，用于 content tile 和 details badge。
- Editor 状态与消息：
  - 用独立 `AssetWorkspaceState` 替换 `EditorState.asset_entries`，集中维护 `selected_folder_id`、`selected_asset_uuid`、`search_query`、`kind_filter`、`activity_view_mode`、`browser_view_mode`、`browser_utility_tab`、`visible_preview_uuids`。
  - 新增资产交互消息：选择文件夹、选择资产、双击资产、切换 list/thumbnail、搜索、kind filter、打开/聚焦 `AssetBrowser`、切换 utility tab、点击引用跳转、请求可见预览刷新。
  - `Assets` 和 `AssetBrowser` 共用 selection/filter 数据，但分开保存布局偏好；默认排序固定为 `folder first -> display_name`，v1 不做用户自定义排序。
  - `Project` 视图改成项目概览：manifest、root/library 路径、默认 scene、library 状态，以及 “Open Assets / Open Asset Browser” 入口，不再渲染资产列表。
- `Assets` 活动面板，Unity-first：
  - 左侧是紧凑 folder tree，仅浏览 `Project/assets`，显示文件夹名和递归计数。
  - 右侧是当前文件夹内容区，显示子文件夹和资产；支持 `List/Thumbnail` 切换、搜索、kind chips、dirty/error/reloading/status badge。
  - 顶部保留现有导入入口，但从裸字符串列表升级为工具条的一部分；在没有文件对话框前，继续使用 path 输入 + Import 按钮。
  - 底部增加紧凑 details strip，固定 tabs 为 `Preview` 和 `References`；单击资产只更新这里，不强制打开全窗口。
- `AssetBrowser` 独占窗口，Unreal-first：
  - 作为 closeable document/exclusive page 保留现有 `editor.asset_browser` 入口，不新建第二套服务。
  - 顶部工具条固定包含：breadcrumbs、search、kind filter、list/thumbnail toggle、import command、`Open in Assets`/`Locate` 操作。
  - 左侧 `Sources` 树、中央 `Content Browser`、右侧 `Details` rail、底部 `Utility Tabs` 构成主布局。
  - 右侧 `Details` rail 显示：大预览、名称、kind、UUID、locator、resource state/revision、meta path、mtime/hash 摘要、diagnostics、adapter key。
  - 底部 utility tabs 固定为 `Preview`、`References`、`Metadata`、`Plugins`；`Plugins` 先作为扩展宿主槽位，不交付具体插件 UI，但 chrome/snapshot/message 必须允许后续插入。
- 预览、缩略图与引用行为：
  - 内容区优先显示 `preview_artifact_path` 对应缓存缩略图；无缓存时显示 kind icon placeholder。
  - watcher/mtime/hash 变更后，UI 立即显示 dirty badge；若已有旧缩略图则继续显示旧图并叠加 dirty 状态。
  - 只有当前可见的 thumbnail 项、选中资产的大预览、或展开中的 preview tab 才触发 `request_preview_refresh(uuid, true)`；不可见项只保留缓存，不后台刷全库。
  - 引用面板只展示直接 `References` 和 `Used By`，不做传递闭包；每项显示名称 + kind + locator，点击后切换 selection 并自动定位到目标文件夹。
  - `builtin://` 和 `mem://` 资源不出现在资产树根中；如果它们出现在引用列表里，按引用项显示并标记 scheme。
- UI 结构与落地顺序：
  - 先替换 workbench 快照和 presenter 的 `asset_entries` 链路，让 Iced 主路径真正吃结构化 `AssetWorkspaceSnapshot`。
  - 若仓库仍保留 Slint workbench 预览/fixture，同步镜像新的 snapshot schema，但不允许再保留独立的字符串资产模型。
  - 将现有 `ProjectAssetsPane` 职责拆开为 `ProjectOverview`、`AssetsActivityPane`、`AssetBrowserPane` 或等价结构，避免继续在一个 pane 里塞两种完全不同的界面逻辑。

## Test Plan
- `zircon_asset` / `zircon_server`：
  - folder tree 从 locator/catalog 正确生成，folder id 稳定，rename/move 后 uuid 选择仍可恢复。
  - `catalog_snapshot` / `asset_details` / `editor change` 的序列化、排序、dirty/diagnostic 字段正确。
  - preview dirty 后仍保留 last-good artifact path；可见刷新只更新目标资产，不触发全量刷新。
- `zircon_editor` 状态与 presenter：
  - `AssetWorkspaceState` 的 folder selection、asset selection、search/filter、list/thumbnail mode、utility tab 切换全部可重建 snapshot。
  - `Assets` 与 `AssetBrowser` 共享 selection，但各自布局偏好互不污染。
  - `Project` 视图不再依赖 `asset_entries`；代码中不再存在 editor public UI path 上的 `Vec<String>` 资产列表。
- 集成与验收：
  - 打开目录式项目后，`Assets` 面板显示 folder tree、内容区、缩略图/列表切换、dirty/error badge。
  - 打开 `AssetBrowser` 后，显示 Unreal-first explorer 布局，右侧 details 与底部 references/metadata/plugin tabs 正常联动。
  - 修改图片/材质/场景资产后，可见缩略图自动刷新；不可见项保持旧缓存直到被查看。
  - 选中资产时引用面板正确显示 `References` 和 `Used By`；点击引用项能跳转到目标资产。
  - 验收至少覆盖 `cargo test -p zircon_asset -p zircon_server -p zircon_editor`，并补一条 editor workbench 集成测试验证 “project open -> catalog sync -> assets panel -> asset browser -> reference navigation”。

## Assumptions
- 本轮 UI 浏览范围固定为项目 `res://` 资产树；`builtin://`、`mem://` 仅作为引用项和状态信息出现，不作为可浏览根目录。
- v1 只做 `folder first -> name` 排序，不做自定义排序、收藏夹、标签搜索、拖放重命名或多选批处理。
- v1 的可交付预览优先覆盖图片、材质、场景三类；模型和 shader 没有可用缓存时显示类型卡片，不为了 UI 去绕过 `EditorAssetManager` 直接读源文件。
- 未来插件化扩展通过 `AssetBrowser` 的 utility tab/adapter 槽位接入；本轮只把宿主布局、快照和消息边界做稳，不在 editor 内新增第二套资产数据库或插件特判。
