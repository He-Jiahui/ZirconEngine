---
title: Editor Asset Workspace / Content Browser 当前源码复核
category: zircon_editor
report_id: Editor226
review_date: 2026-08-30
baseline_head: e76240e1299259b8c4abb4def5e3f0537bda5074
baseline_epoch: current-working-tree
verification_head: e76240e1299259b8c4abb4def5e3f0537bda5074
verification_epoch: current-working-tree
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
related_owner_reports:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/56-editor-search-filter-query-index-result-find-usage-reference-navigation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
---

# Editor226 · Asset Workspace / Content Browser 当前源码复核

- `review_date`: 2026-08-30
- `scope`: `zircon_editor` Asset Activity、独立 Asset Browser、catalog/details/preview/reference projection、asset pointer/tree drag、asset import/refactor operations、editor asset events/bindings
- `review_mode`: review-only；本轮没有修改 production Rust/ZUI，也没有运行 Cargo、Editor、真实文件 mutation、watcher conflict、plugin reload、screen reader、fault、soak 或 benchmark
- `canonical_owner`: 本文刷新 Editor57；Editor56 继续拥有 unified query/index/find-usage，Editor55 继续拥有通用 clipboard/duplicate/delete/drag semantics，Editor04/Runtime85/86/87 继续拥有 catalog/import/type/refactor algorithms，Editor02/03/61/225 继续拥有 document/dirty/scene transition/save/recovery
- `focused_source_manifest`: 137 files, 23,220 lines, 836,862 bytes, 186 test attributes, 15 ignored attributes
- `working_tree_fingerprint`: `2385ab34192d6f39845de287900a3d10a1e3e9063e6ab8d4be92539ae9c5af09` (lowercase relative path + NUL + per-file SHA-256 + LF; dirty working tree included)

## 1. 结论

Zircon 当前已经有一组真实的局部底座：Runtime catalog 能发布 generation，EditorAssetManager 能同步 catalog/details/resource generation，preview/import 有 bounded admission 和 job ticket，删除有 referencer preflight，重定位和删除已经进入 Runtime-owned job，default scene reload 也已经从旧的 retained-host 直接替换路径迁移到 `ProjectAuthority` ticket、dirty policy、generation/identity 校验和 admission retry。

但 Asset Workspace 仍不是工程级 Content Browser。可见 UI 仍是固定演示模板；Activity 与 Browser 共享一套可变 folder/selection/query；catalog 到 selection/reference/toolkit 的 exact type 在 `ResourceKind` 转换处丢失；item pointer 只产生 `SelectItem`，没有 double-click/Enter/Open activation；Locate 按钮没有 target identity；source/folder 没有 provider capability；Create/Rename/Duplicate/Reimport/bulk/collections/history 仍缺真实产品闭环。已有后台 job 不等于用户可达的 action、preflight、receipt 和 failure recovery。

旧 Editor57-P0-01 的“watcher 直接替换 authoring world”在当前 `workspace.rs` 路径已不再按旧描述执行：现路径提交 `ProjectSceneLoadTicket`，持有 document identity/generation/dirty policy，并在完成前再次校验。它应改为 Editor61/225 的跨 owner gate，而不是在本报告重复计为当前唯一 P0。其余三项 P0 仍然可由当前生产入口复现，必须先于 UI 扩展关闭。

## 2. 当前真实链路

### 2.1 UI 与事件

- `assets_activity.zui` 的搜索、kind filter、view mode、utility tab 与 `OpenAssetBrowser` 有事件声明；独立 `asset_browser.zui` 也有 Locate、ImportModel、filter、view/tab 事件。但 content 区只声明 `WorkbenchAssetBrowserAssetRow01..04` 四个固定 `WorkbenchTableRow`，文本是 `Host UI 12K r42`、`Base Style 8K r41` 等静态样例，没有动态 item template、qualified row payload 或 item activation event（`zircon_editor/assets/ui/editor/asset_browser.zui:215-250`）。
- `EditorAssetEvent` 提供 `OpenAsset`、`SelectFolder`、`SelectItem`、`ActivateReference`、`RelocateAsset`、`DeleteAsset`、`LocateSelectedAsset` 和 `ImportModel`，但 binding/dispatch 只把 string/UUID 直传到 host；`LocateSelectedAsset` 没有 selection token，`SetKindFilter` 仍是 string 到 coarse `ResourceKind` 的转换。
- `asset_content_pointer` 的 click bridge 只按 route 发 `SelectFolder` 或 `SelectItem`；没有 click count、double-click、Enter、focused item 或 `AssetActivationIntent`。press 路径只建立 drag payload，不能打开或预览项目资产。该结论来自 `zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/asset_content.rs:22-52` 和 `zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/click.rs:4-60`。
- `OpenAsset` 执行器本身能解析 `AssetUri`、通过 `asset_type_id_for_locator` 查类型、获取 enabled toolkit、检查 command registry 并打开 document view；问题是浏览器真实输入没有把 item activation 路由到它。`LocateSelectedAsset` 当前只打开 `editor.assets` 并请求 preview，仍未调用 `navigate_to_asset` 或返回 reveal receipt。

### 2.2 Workspace state、catalog 与 identity

- `AssetWorkspaceState` 只有一份 `selected_folder_id`、一份 `selected_asset_uuid`、一份 query/kind filter；Activity/Browser 仅对 view mode 与 utility tab 分开，`build_surface_snapshots` 直接 clone Activity snapshot（`zircon_editor/src/ui/workbench/project/asset_workspace_state.rs:43-77,332-340`）。没有 `AssetBrowserInstanceId`、history、source selection、sort/column、expanded set、anchor/focus 或 multi-selection。
- `build_snapshot` 每次递归 materialize 整个 folder tree；visible assets 仍对 `catalog.assets` 线性过滤，按 display/file/locator 做 ASCII substring，单一 `ResourceKind` filter（`asset_workspace_state.rs:271-329,377-410,597-705`）。这不是 Editor56 的 indexed query session，也不能支撑 100k/1M 资产分页。
- `EditorAssetCatalogRecord` 只有 UUID/id/locator/`ResourceKind`/preview/hash/dirty/diagnostics/reference UUID；folder 只有 string id/prefix/name/children/count。没有 exact `AssetTypeId`、schema/version、compatibility、provider/source id、source generation、capability、currentness、mount health 或 operation generation（`zircon_editor/src/ui/host/editor_asset_manager/records.rs:8-76`）。
- `asset_type_id_for_locator`、selection、subasset、reference 和 toolkit projection 都调用 `AssetTypeId::from_resource_kind`。registry enrichment 发生在 coarse projection 之后，不能恢复第三方类型、plugin toolkit、creation template 或 context commands（`asset_workspace_state.rs:204-210,489-584`; `zircon_editor/src/ui/host/editor_asset_manager/asset_access.rs:298-390`）。未知类型会被静默投影为 builtin kind，而不是保留 opaque identity/Unavailable 诊断。
- folder builder 能从 `res://` 与 `package://id` 构造中间目录、递归计数和排序，但 package root 仍只有 id/display name；没有多 content roots、remote/generated/cooked provider、mount availability、trust/offline 状态或 source-owned action。

### 2.3 Mutation、import 与 external change

- 删除已有 `AssetDeletePreflight`，会读取 Runtime registry topology、referencers 和 project write policy；通过后提交 Runtime-owned source deletion ticket，完成后同步 workspace 并显示状态（`zircon_editor/src/ui/retained_host/app/assets/deletion.rs:14-54,82-119`）。但浏览器 context menu 目前只有 Delete；没有 rename/move/duplicate/create/reimport/bulk action、open-document/dirty/source-control preflight、trash/undo 或 per-item terminal receipt。
- 重定位已有 Runtime-owned `ProjectSourceRelocationJob` 和 ticket；tree drop 会验证 source UUID、locator、subasset 和目标 folder，并用同名 target locator 调用 relocation。它仍只处理单个左键释放，没有 drag enter/over/leave provider、copy/link modifier、跨 source policy、cycle/conflict preview 或 qualified receipt（`zircon_editor/src/ui/retained_host/app/asset_tree_pointer/events.rs`）。
- `EditorAssetImportFlow` 具备 generation key、UUID lifecycle、flight/byte/age admission、dedup、diagnostics 和 cancellation；`submit_model_source` 是唯一明确的 compound model backend。UI `ImportModel` 仍是固定的 model source 输入，`complete_model_import` 在成功后隐式准备 default material 并调用 `runtime.import_mesh_asset`，把“Import Asset”与“Place in Scene”耦合为一个流程。
- refresh accumulator 已有 quiet/max-deferral 和 per-stream bounded drain，workspace active-scene reload 已有 ticket/retry/conflict/identity/generation fence。但 refresh plan 仍把多个 producer 变化压成 URI/UUID vectors 和布尔 flags，缺 producer/source/document identity、expected generation、原始 reason/range、单一 publication receipt，以及 Browser instance/history/preview/drag teardown。
- source watch projection 能把 Added/Modified/Removed/Renamed 转成 catalog changes，并在 publish gate 下更新 AssetIndex；不完整 rename 会降级为 Added，且 change record 只有 kind/revision/UUID/locator。没有跨 provider rename reconciliation、selection/history/focus/open-document 原子更新或 event-gap currentness。

## 3. P0：当前必须先关闭

### ED226-P0-01 · Locate selected asset 仍没有资产身份，按钮语义与执行行为不一致

`EditorAssetEvent::LocateSelectedAsset` 是无字段 variant。当前执行路径只打开 `editor.assets` 并请求 preview；它不读取当前 Browser/Activity selection，不调用已经存在的 `navigate_to_asset`，也不清除 query/kind filter、展开 source path、滚动和聚焦。没有 selection、stale UUID、隐藏 source 或过滤阻挡时也没有失败原因。

重构为 `RevealAssetRequest { browser_instance, qualified_target, catalog_generation, reason }`，统一返回 `AssetRevealReceipt`（target/folder/query adjustment/scroll/focus/outcome）。在新事件落地前应禁用按钮或改成真实的“Open Assets”语义。Owner：Editor226 consumer，Editor56 query reveal，Runtime86 exact identity。

### ED226-P0-02 · 浏览器 item 入口不可达 Open/Activate，toolkit 内核不是产品功能

真实 content click 只发 `SelectItem`，press 只产生 drag payload，ZUI 固定 row 没有 dynamic qualified item event；没有 double-click、Enter、Open/Open With、Preview 或 reference activation 的统一入口。因此 `OpenAsset` handler 存在，但主 Browser 无法从用户动作调用它。

引入 `AssetActivationIntent`，由 pointer/keyboard/menu/context/reference 共同产生；在 invoke 时用 exact type、toolkit generation、catalog/resource revision 做 stale 校验，并返回 opened/reused/unavailable/failed receipt。单击只能改变 selection，双击/Enter 才能激活。Owner：Editor226 + Editor50 toolkit lifecycle，Editor09 job/receipt。

### ED226-P0-03 · exact AssetTypeId 在 catalog projection 中丢失，插件资产会被伪装成 builtin

`EditorAssetCatalogRecord`、details、selection、subasset、reference 没有 exact type；`asset_type_id_for_locator` 和 `asset_type_projection` 从 `ResourceKind` 反推。启用 registry 只能在错误的 coarse identity 上做二次查找，无法恢复 AI/network/particle/import-settings 等 plugin type 的 toolkit、schema、creation/context action。

Runtime catalog 必须发布 exact `AssetTypeId + schema/version + compatibility`，Editor catalog 全链保留；`ResourceKind` 只作分组/图标 fallback。插件卸载、未知 schema、缺 importer 时保留 opaque identity 并显示 Unavailable/Compatibility diagnostic，不得静默降级。Owner：Runtime86/85，Editor04 catalog/import，Editor50 registry，Editor226 projection。

### 3.1 旧 ED57-P0-01 的当前重判

当前 `zircon_editor/src/ui/retained_host/app/assets/workspace.rs` 已通过 `ProjectSceneLoadTicket`、dirty policy、document identity、project generation、admission retry 和 commit-time revalidation 执行 active-scene reload；旧的“watcher 直接 `runtime.replace_world`”描述不再准确。该项转为 Editor61/225 的跨 owner qualification gate：仍需证明 dirty conflict、failure keeps old world、history/document transition 和 watcher event gap 的端到端产品证据，但不在 Editor226 重复计为一个独立 P0。

## 4. P1：工程能力差距与当前状态

下表保留 Editor57 ID，状态按当前源码重判：`Open` 表示产品入口/合同不存在，`Partial` 表示有局部底座但仍不能宣称完成。

| ID | 当前状态 | 差距与重构方向 |
|---|---|---|
| ED57-P1-01..08 | 01/02/03/05/06/07/08 Open；04 Partial | selection/activation、Open With/Preview、真实 context/action provider、creation menu、capability 和 action receipt 未统一；registry projection 不能替代 executable action。 |
| ED57-P1-09..16 | 09/10/11/12/13/15/16 Open；14 Partial | 单 state、无 instance/history/lazy expansion/multiselect/persist；外部移动已有 UUID 时只做局部 selection reconcile，不更新 folder/history/focus。 |
| ED57-P1-17..24 | 全部 Open，23 为 Partial | folder/source 缺 provider identity/capability；package root 无 health/version/trust；exact type/schema/compatibility/subasset activation/source delta 未闭合。已有 subasset/reference DTO 只能展示。 |
| ED57-P1-25..32 | 25/26/29/32 Open；27/28/30/31 Partial | relocation/deletion/import/refresh backend ticket 已存在，但 UI 没有完整 create/rename/duplicate/reimport/bulk action、preflight、destination picker、undo/receipt、last-good 与 per-item result。 |
| ED57-P1-33..40 | 34/35/36/37/38 Open；33/39/40 Partial | 当前可提交 model import 且有 admission/diagnostics，但文案和 backend 仍 model-only；目标目录、importer registry、derived outputs、default material dependency、scene placement 分离、progress/cancel/reveal/retry 未成为一体化 operation plan。 |
| ED57-P1-41..48 | 42/44/45/48 Open；41/43/46/47 Partial | bounded watcher/scene reload/job 底座真实存在，但 change identity、decision/reconciliation/publication receipt、asset currentness 与 teardown lease 仍缺；active-scene 旧直接替换已转为 Editor61/225 gate。 |
| ED57-P1-49..56 | 51/52/53/54/55/56 Open；49/50 Partial | typed drag payload 和单向 folder drop validation 已存在；缺 copy/move/link effect、provider enter/over/leave、cross-source policy、favorites/collections/virtual sources 和统一 command provider。 |

具体重构顺序：先建立 exact item/source/provider contract，再拆 Browser instance state；之后接 activation/action provider 和 mutation coordinator；最后接 importer/create、external reconciler、drag/favorites/collections。不得先增加更多 toolbar button 或右键菜单。

## 5. P2：性能、质量与维护

- **ED57-P2-01**：preview decoded image cache 的 entry/byte/LRU/project/generation eviction 需要以当前实现复核；不能把 preview ticket admission 当作 decoded image cache budget。
- **ED57-P2-02/03**：presentation 仍可能在 cache miss 同步加载图片；revision/dirty key 会造成旧 Image 保留，失败 fallback 缺 currentness/retry 语义。
- **ED57-P2-04/05**：完整 folder tree 递归 materialize、visible asset 线性扫描和每次 String 分配仍在 `asset_workspace_state.rs`；改为 provider page、expanded branch、indexed query consumer。
- **ED57-P2-06**：Browser snapshot clone Activity 会复制 Vec/String；改为 immutable catalog/page 与 instance-local view projection。
- **ED57-P2-07**：`asset_browser.zui` 当前 817 行，toolbar/source/content/details/preview/reference/plugins 混在一个文件，且 content rows 是硬编码；按 source/content/details/utility 和 dynamic item template 拆分。
- **ED57-P2-08/09**：可见文案多为英文 literal；keyboard/focus/semantic/a11y 没有与 action provider 同一合同。
- **ED57-P2-10**：已有 pointer/layout 单元测试不能证明真实 double-click/Enter/Open With/rename/drop/context/progress/cancel；必须增加真实 host integration/E2E。
- **ED57-P2-11/12**：没有 100k/1M 多 source 深目录、preview churn、多实例 memory/p95/p99/soak、stale receipt/conflict/action latency/currentness telemetry 基线。

## 6. 参考源码对照

| 参考 | 当前源码事实 | Zircon 应吸收的合同 |
|---|---|---|
| Unreal `ContentBrowserDataSource.h` | Data source 自己拥有 enumerate/filter、folder visibility、create、asset access、dirty、edit 和 action capability；item 携带 source-owned opaque payload。 | `ContentSourceProvider`/`AssetActionProvider`/capability/preflight 必须由 source/provider authority 提供，不能由 path scheme 猜测。 |
| Godot `filesystem_dock.h` + `editor_file_system.h` | FileSystem dock 同时覆盖 open、rename/move、duplicate/delete、reimport/rescan、favorites/history、依赖更新；filesystem cache 记录 UID/import md5/import validity，并区分扫描线程与 UI。 | 明确 reimport/rescan、UID/import currentness、依赖修复、用户可见失败和后台扫描状态；不要用物理 path 代替 stable identity。 |
| Fyrox `editor/src/asset/item.rs` + preview cache | item 双击打开；move 前调用 resource manager 的 `can_resource_be_moved`，再以 `MoveTo` 消息执行；preview cache 按 resource UUID 缓存并支持 force update/generator。 | 最小产品闭环必须有 activation、destination preflight、typed resource identity、可失效 preview cache；Fyrox不代表 1M 资产性能上限。 |
| Bevy `asset/path.rs`, `event.rs`, `io/source.rs` | `AssetPath` 分离 source/path/label；`AssetSource` 可提供 reader/writer/unprocessed+processed watcher；事件包含 Added/Modified/Removed/Unused/LoadedWithDependencies。 | source、subasset label、reader/writer/watcher capability 和 qualified change event 进入 Runtime/Editor 合同；Bevy本身没有 Content Browser 产品层，不能照搬其简化 UI。 |
| Unity Graphics `AssetReimportUtils.cs`, `AssetCreationUtil.cs` | 批量 reimport 使用 `StartAssetEditing`/`StopAssetEditing`、progress 和 finally 收尾；创建使用 unique path、name editing、CreateAsset、SaveAssetIfDirty、Refresh、回调 reveal。 | create/reimport 必须有 unique naming、batch boundary、progress、cancel/failure cleanup、save/refresh/reveal；本地 Graphics corpus 只作 consumer 旁证，不推断完整 Unity AssetDatabase。 |

## 7. 目标架构与重构里程碑

```text
AssetWorkspaceService
  -> BrowserInstanceRegistry(instance/history/query/selection/expansion)
  -> ContentSourceProviderRegistry(source/folder/page/capability)
  -> AssetActivationRouter(exact type -> toolkit -> activation receipt)
  -> AssetActionProviderRegistry(availability/preflight/invoke/result)
  -> AssetMutationCoordinator(expected generation -> commit/publish/receipt)
  -> AssetChangeReconciler(qualified delta -> document/instance reconciliation)
  -> CollectionService(favorites/static/dynamic/virtual sources)
```

1. **M226.0 P0 hard cut**：替换无 target Locate；接入 item double-click/Enter activation；禁止 coarse type 静默伪装，未知类型显示 opaque/unavailable；为三项 P0 各增加真实失败先行测试。
2. **M226.1 exact source contract**：与 Runtime85/86 对齐 exact type/schema/compatibility、`ContentSourceId`、provider generation、folder/item stable id、capability 和 source delta。
3. **M226.2 instance state**：引入 `AssetBrowserInstanceId`，独立 navigation/history/query/sort/column/selection/anchor/focus/expansion/view/utility，支持 lazy tree、multi-selection、persist/restore 和 external move reveal。
4. **M226.3 activation/action**：统一 pointer/keyboard/menu/context/reference；真实 Open/Open With/Preview/Reveal/Create action 消费 toolkit lifecycle、principal capability、busy/lease 和 unavailable diagnostics。
5. **M226.4 mutation**：与 Editor55/Runtime87 建立 Create Folder、Rename、Move、Delete、Duplicate、Reimport、bulk 的 preflight -> expected generation -> reference/document/source checks -> commit/publish -> receipt/compensation。
6. **M226.5 import/create**：importer registry + target folder + options/recipe + derived output batch + progress/cancel + partial policy；`Import Asset` 与 `Place in Scene` 两个可独立撤销的命令。
7. **M226.6 external/scale**：qualified watcher delta、currentness/decision/retry、multi-source teardown；paged provider、bounded decoded preview、100k/1M benchmark、a11y/localization/telemetry。

## 8. 资格门

- **Activation**：item 单击只 selection；double-click/Enter/Open command 只触发一次 activation；exact toolkit 缺失时 Unavailable；Locate 带 qualified target 并能展开/滚动/聚焦；所有入口共享 availability 和 terminal receipt。
- **Instance/source**：两个 Browser 的 folder/query/selection/history/expansion 独立；source/provider、package/remote/read-only 保留 identity/capability；10万 folder fixture 不做全树 clone。
- **Mutation/import**：create/rename/move/delete/duplicate/reimport/bulk 有 per-item preflight、source/dirty/reference/open-document 检查、atomic/partial policy、before/after generation、artifact/reference/document outcome；导入目标可选择 writable folder，scene placement 分离。
- **External/lifecycle**：watch event 保留 producer/source/sequence/reason；gap 触发 bounded resync；catalog/details/preview/document 有 publication receipt；project/plugin/source teardown 取消 preview/drag/action lease，不跨 project 回调。
- **Performance/quality**：decoded preview 有 entry/byte/LRU/project eviction；UI presentation 不做未预算 I/O/decode；100k/1M p95/p99、multi-instance memory、preview churn soak、fault/rollback、keyboard/a11y/localization/telemetry 有报告。

## 9. 当前性与实施规则

1. 实施前重新导出 focused manifest，重算 fingerprint，并复核本报告列出的 dirty 文件；当前指纹包含工作树改动，不代表 HEAD 基线。
2. 重新检查 `EditorAssetEvent::OpenAsset` 的 production constructors、`LocateSelectedAsset`、`AssetTypeId::from_resource_kind`、`asset_browser.zui` dynamic binding、`submit_project_source_*` 与 active-scene reload owner 状态。
3. 不在 ZUI callback 中直接做 filesystem、scan/import、scene replacement 或 reference rewrite；UI 只消费 action/provider/operation/receipt。
4. 不新增 Browser 私有 importer/query/clipboard/dirty/job/reference subsystem；按 Editor04/09/55/56 与 Runtime85/86/87 的 owner contract 接入。
5. 不以 catalog 刷新后行数变化证明 mutation 成功；不得保留旧 string-path action shim；不得把未知 plugin type 降级为 builtin Data。
6. 本轮严格排除 tooling；后续 tooling 迁移 Rust 后，Editor/Runtime 产品资格仍需独立验证。
