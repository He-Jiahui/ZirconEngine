---
title: Editor Asset Workspace、Catalog、Provider、Preview、Import、Reimport 与 Generation 当前工作树复审
category: zircon_editor
report_id: Editor267
review_date: 2026-08-31
baseline_head: working-tree
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/snapshot/asset
  - zircon_editor/src/ui/workbench/asset_content_layout
  - zircon_editor/src/ui/retained_host/asset_pointer
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer
  - zircon_editor/src/ui/retained_host/callback_dispatch/asset
plan_sources:
  - docs/plans/optimize/zircon_editor/248-editor-asset-workspace-catalog-provider-preview-import-reimport-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/257-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/260-editor-extension-contribution-store-toolkit-reload-lifecycle-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/205-runtime-resource-lifecycle-load-ticket-cache-residency-generation-reload-cancellation-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/AssetManager.h
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/Fyrox/editor/src/asset
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/godot/editor/filesystem_dock
  - dev/godot/editor/editor_file_system.h
  - dev/godot/core/io/resource_loader.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: current_source_review
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor Asset Workspace、Catalog、Provider、Preview、Import、Reimport 与 Generation 当前工作树复审

## 1. 结论

当前 Editor asset 链已经不是纯 UI mock。Runtime `ProjectCatalogInputGeneration` 是 catalog 输入，Editor 通过 source-sync epoch、Runtime project generation token、catalog revision、publish epoch 和 immutable `EditorAssetCatalogGeneration` 拒绝旧发布；workspace item 使用 64-row chunk generation，catalog/resource payload-only change 可以局部替换 chunk；Browser paint 已有物理 slot virtualization；preview job 已接入 `EditorJobSystem`、bounded admission、token、cancellation check、stale completion fence；普通 import 已有 generation-qualified single-flight、retained flight/byte/age admission，model import 返回 Runtime durable `ProjectImportReceipt`；delete/relocate 也由 Runtime owner 执行并返回 ticket。

这些能力仍没有形成 Unreal 级别的 Content Browser / Asset Tools 工程闭环。最关键的断裂没有变化：

1. Runtime exact asset type 到 `EditorAssetCatalogRecord` 后仍退化为 `ResourceKind`，workspace 又用 `AssetTypeId::from_resource_kind` 反推；plugin type、schema、compatibility 和 unavailable reason 全部丢失。
2. Runtime 已发布 delta，但任意非空 delta 仍重新构造 `EditorAssetIndex`、完整 catalog map、folders、details 和 preview scheduler；paint virtualization 没有消除 data/provider 全量物化。
3. workspace 仍只有一个 selection、folder、query 和 kind filter；Activity 与 Browser 只分开 view mode/utility tab，共享导航和选择 authority，且仅支持单 UUID。
4. `AssetSourceAuthority` 仍按 URI scheme 猜 source/write policy；catalog/folder/item 不携带 provider identity、mount generation、capability、health、trust 或 read-only reason。
5. preview cache 是直接写入 UUID+字符串 variant PNG 的无界目录；没有 content address、byte lease、LRU、atomic publish、generator version、negative cache 或 orphan sweep。
6.普通 import 和 model import 是两套 ticket 语义；API 仍只有 model import、delete、relocate、preview，没有通用 create/rename/duplicate/reimport/bulk/open/activate provider。
7. 主 asset content click 只派发 `SelectItem`，press 只构造 drag payload，context menu 只有 Delete；`AssetToolkitOpenRoute` 与 Browser 的 double-click/Enter/Open With/activation receipt 仍未接通。

本轮不新增唯一 P0，继续继承 Editor226、Editor60/61/247 和 Runtime205/204 的 canonical P0。对 Editor248 的 32 个 `ED6-P1` 逐项复核后为 **22 Open / 10 Partial / 0 Closed**；12 个 `ED6-P2` 为 **8 Open / 4 Partial / 0 Closed**。本篇新增 16 条 currentness/convergence 风险和 24 个工程资格门；它们用于指导重构，不创建第二份实现 backlog。

## 2. 审查范围与证据

### 2.1 Focused production manifest

本轮选择 `core/asset`、Editor asset manager、workspace/snapshot/layout、retained asset pointer 与 asset callback 链；排除 `tests`、`*_tests.rs`、performance/responsive test 文件后，当前 focused manifest 为：

| 维度 | 当前值 |
|---|---:|
| Rust 生产文件 | 145 |
| 总行数 | 17,932 |
| 非空行 | 16,281 |
| 字节数 | 614,403 |
| path + SHA-256 manifest 指纹 | `72f391aeba670e0d126ea7514921dfc13c033e0b043be49bfa192c55508585d6` |

重点逐链读取：

1. catalog/index/type：`EditorAssetIndex`、type registry、source authority、toolkit route、catalog records/generation/build/update；
2. Runtime publication：`sync_from_project_with_runtime_generation`、catalog input delta、commit-if-generation、watch projection、deactivation；
3. workspace：selection/navigation/filter、item chunk generation、folder projection、resource overlay、Browser paint virtualization；
4. preview：cache/key/scheduler、job admission、generation currentness、artifact generation、completion publication；
5. import/mutation：generic import flight、model import receipt、delete preflight、Runtime-owned delete/relocate tickets；
6. product input：retained pointer click/press/drag/context menu 与 asset surface callback dispatch。

### 2.2 证据等级

- **E3**：读取当前生产实现并追踪 input generation、candidate build、winner commit、workspace publication、job completion 和 pointer dispatch。
- **E2**：读取相邻测试声明和本地参考引擎源码；测试存在不等于当前工作树动态通过。
- **E1**：本轮是 review-only，未运行 Cargo、Editor、真实 import/reimport、project switch、plugin unload、fault、scale、soak 或 benchmark。
- **E0**：没有同 workload、同画质、同平台的 correctness 与 p50/p95/p99/p99.9 数据，不能声称当前性能或表现优于 Unreal。

## 3. 当前可保留底座

### 3.1 Runtime generation 是 catalog 输入 owner

`DefaultEditorAssetManager::refresh_from_runtime_project` 获取 `current_project_generation_snapshot`，随后 `sync_from_runtime_project_generation` 在 Runtime `commit_if_project_generation` 内完成 Editor winner commit。source-sync epoch 会拒绝 superseded build，catalog generation 的 revision/publish epoch 使用 checked increment，project deactivation 会推进 epoch、清 import flow 并发布新的空 generation。这些都是正确方向。

### 3.2 Immutable generation 与局部 payload 替换

`EditorAssetCatalogGeneration` 用 `Arc<[...]>` 和索引发布不可变快照；preview completion 不原地改共享 row，而是生成新 publish epoch。`AssetWorkspaceItemGeneration` 用固定 64-row chunk、UUID/locator index 和 selected index；catalog/resource payload-only 更新可替换受影响 chunk。Browser paint 又用 logical paint chunks 和有限 physical slots 绑定可见窗口。应保留这些 publication/structural-sharing 内核。

### 3.3 Preview 有真实 admission 和 currentness fence

`PreviewScheduler` 有 dirty/visible/in-flight 集合和 64 job 上限；每项 admission 有 token。preview job 通过 `EditorJobSystem` 提交 Background job，进入和发布前检查 cancellation；completion 同时校验 admission token、catalog revision、row `Arc` identity、source hash、source digest 和 meta path，Drop 会释放未完成 admission。相较无 fence 的临时异步实现，这部分已具备可扩展内核。

### 3.4 Import/mutation 已经让 Runtime 做 durable owner

普通 import 使用 generation key、single-flight、reason merge、flight/byte/age admission 和 index importing transition；model import 不在 Editor 自行写文件，而是调用 Runtime compound transaction 并返回 `ProjectImportReceipt`。delete/relocate 也通过 interactive job 调用 Runtime `ProjectAssetManager`，并用 project mutation mutex 串行化。这些边界不应退回 UI 直接文件操作。

### 3.5 Change stream 已有 bounded mailbox

`EditorAssetChangeHub` 为每个 consumer 保持 512 项 bounded mailbox、内部 publish sequence、coalescing、queue age、wake callback；overflow 会收敛成最新 catalog refresh。这是正确的有界 fan-out 起点，但缺口见 `EAW-D07`。

## 4. Canonical owner 与不重复 P0

| Canonical owner | 本篇只承担的 Editor 接线 | 当前状态 |
|---|---|---|
| Editor226 | Browser locate/open/activate、exact type 到 UI、reveal terminal outcome | 继承 P0，不在本篇重编号 |
| Editor60/61/247 | project/document/play/world generation、dirty conflict、retired callback | mutation/preview/catalog 必须消费同一 session fence |
| Runtime205 / Runtime99m | handle/load ticket/cache/residency/reload/cancel/provider generation | Editor 只消费 receipt，不自建第二资源 authority |
| Runtime204 | preview/artifact/source 写入的 root capability、atomic transaction、durability/recovery | Editor cache/operation 迁移到 Runtime I/O owner |
| Editor257 | job owner、priority、cancel acknowledgement、shutdown/drain | preview/import 不复制 scheduler 内核 |
| Editor260 | plugin contribution、toolkit/action owner、reload/unload drain | asset provider 只登记 capability 和 operation factory |
| Tooling | 后续迁移 Rust | 按用户要求排除 |

## 5. Editor248 `ED6-P1` currentness 结算

### 5.1 Catalog、identity 与 source/provider

| ID | 状态 | 当前证据 | 关闭条件 |
|---|---|---|---|
| ED6-P1-001 | Open | catalog、details、workspace 仍只存 `ResourceKind`；`asset_type_id_for_locator` 反向构造 builtin type。 | 全链保留 exact type/schema/version/compatibility，未知类型可 opaque 展示。 |
| ED6-P1-002 | Open | reference/subasset 仍是 String UUID/locator + optional kind，没有 project/source/revision/remap lineage。 | 引入 generation-qualified `QualifiedAssetRef`。 |
| ED6-P1-003 | Partial | immutable catalog generation 与 editor-local dirty/importing set 已存在；但 `EditorAssetIndex` 与 catalog record 仍各存 metadata/reference 事实并整表替换。 | Runtime catalog generation 成为唯一事实，Editor overlay 只保存本地 transient state。 |
| ED6-P1-004 | Partial | item/paint 已 chunked 且 Browser 有物理 slot virtualization；`EditorAssetIndex::rows` 和 workspace filter 仍全量 materialize，没有 page/query cursor。 | provider query session 只解码/物化可见页，continuation token 可失效和取消。 |
| ED6-P1-005 | Open | folder/item 无 provider id、mount generation、health/trust/capability。 | `ContentSourceProvider` publication 成为目录与条目的来源。 |
| ED6-P1-006 | Open | `AssetSourceAuthority::from_locator` 仍按 scheme 推断 Project/Package/Builtin/Library/Derived/Transient。 | capability + mount generation + actionable denial reason 取代 scheme 猜测。 |
| ED6-P1-007 | Partial | Runtime project token、catalog-input pointer、source-sync epoch、catalog revision/publish epoch 已共同做 currentness 检查；仍无统一 publication receipt 和 changed pages。 | 一个 receipt 携带 source/runtime/editor generation、delta pages 和 superseded reason。 |
| ED6-P1-008 | Open | details 的 included files/references/subassets 空集合仍不区分 complete、not-loaded、stale、producer unavailable。 | 每字段发布 provenance/completeness/unavailable diagnostic。 |

### 5.2 Workspace、navigation 与 activation

| ID | 状态 | 当前证据 | 关闭条件 |
|---|---|---|---|
| ED6-P1-009 | Open | Activity/Browser 只有独立 view/utility tab；selection/folder/query/filter 共享同一个 `AssetWorkspaceState`。 | 每个 `AssetBrowserInstanceId` 有独立 navigation/query/sort/selection/focus/expansion。 |
| ED6-P1-010 | Open | `build_folder_tree` 每次把全部 folder 分组、排序并递归 materialize。 | provider page + lazy expanded-branch generation。 |
| ED6-P1-011 | Open | filter 每次对 display/file/locator 做 `to_ascii_lowercase` 和线性 scan。 | indexed query、typed filters、sort、cursor、cancel 和 query telemetry。 |
| ED6-P1-012 | Open | selection 仍是单个 UUID；没有 set/anchor/focus/catalog receipt。 | multi-select model 在 rename/remove/reorder 时原子 reconcile。 |
| ED6-P1-013 | Open | navigation target 仍是 String folder id，仅硬编码 `res://`/`package://` parent 解析。 | typed source/folder/collection/favorite/saved-query/history target。 |
| ED6-P1-014 | Open | toolkit route 仍只有 locator + operation path；selection snapshot 的 toolkit fields 仍为空。 | route 固定 qualified item、toolkit owner/generation、intent 和 expected catalog generation。 |
| ED6-P1-015 | Open | content click 只派发 `SelectItem`，press 只生成 drag payload，未找到主 Browser double-click/Enter/Open With terminal route。 | 所有入口汇入 `AssetActivationIntent -> ActivationReceipt`。 |
| ED6-P1-016 | Open | Locate/Reveal 的 browser instance、scroll/focus/filter adjustment 和 terminal outcome 仍未形成统一请求。 | 完成 Editor226 canonical reveal contract。 |

### 5.3 Preview、toolkit 与 editor operations

| ID | 状态 | 当前证据 | 关闭条件 |
|---|---|---|---|
| ED6-P1-017 | Open | `PreviewCache` 只创建目录并直接写 PNG；无 byte/entry/LRU/project/mount budget 或 lease。 | content-addressed preview artifact cache + accounting/eviction/orphan sweep。 |
| ED6-P1-018 | Open | key 仍是 UUID + `thumbnail-{source_hash}` String，不含 type/schema/generator/platform/scale/color space。 | typed variant key 与 generator/build target version。 |
| ED6-P1-019 | Partial | bounded in-flight、visible admission、job priority、mutex group、cancel check 和 Drop release 已存在；无有序等待队列、deadline、distance、reason 或 cancel acknowledgement。 | 复用 Editor257 的 priority/deadline/cancel authority，并按 viewport budget 排队。 |
| ED6-P1-020 | Partial | completion 已校验 catalog/row/source/meta/admission；但 preview 读取 source image 时没有 Runtime resource snapshot/lease/revision。 | project/catalog/resource/provider generation 同时进入 job input 与 completion receipt。 |
| ED6-P1-021 | Open | Error 只落 `PreviewState::Error` 并返回 `JobError`；无 variant last-good、negative cache、retry/backoff。 | variant state machine 发布 retry/actionable diagnostic 和 last-good pointer。 |
| ED6-P1-022 | Open | exact type registry 有 definition/owner，但 workspace context commands 为空，action availability 未消费 provider capability、busy/permission 或 unload drain。 | `AssetActionProvider` 统一 availability/preflight/invoke/result。 |
| ED6-P1-023 | Partial | project deactivation 会推进 sync epoch、清 import flow、替换空 catalog 和 scheduler；仍没有 workspace session owner token、mount lease、全部 job cancel/drain 和 retirement receipt。 | `ProjectAssetWorkspaceSession` 成为 project-bound child scopes 的唯一 owner。 |
| ED6-P1-024 | Open | asset state/publish/source gates/change mailboxes 遇 poison 仍统一 `into_inner` 继续读写，测试也把此行为当 recovery。 | poison 进入 degraded/fenced generation，验证或重建后才允许写/激活。 |

### 5.4 Import、reimport、mutation 与 external change

| ID | 状态 | 当前证据 | 关闭条件 |
|---|---|---|---|
| ED6-P1-025 | Open | Manager API 仍只有 delete、relocate、model import、preview，没有通用 create/rename/duplicate/reimport/bulk/open/activate。 | operation provider registry 返回 typed preflight/ticket/per-item receipt/undo。 |
| ED6-P1-026 | Open | model flow 仍只接受 source `PathBuf`，由 Runtime 固定 compound model transaction 执行，没有可审计 recipe/output plan 与独立 Place in Scene。 | source snapshot + typed importer recipe + output plan，placement 为独立 undoable operation。 |
| ED6-P1-027 | Partial | generic import 已有 generation single-flight、bounded admission、reason merge 和 completion ticket；请求仍只有 URI+reason，model 又走独立 ticket，缺 target provider/options/output graph/profile。 | 所有 importer 统一 request/recipe/output/dependency receipts 和 cooperative cancel。 |
| ED6-P1-028 | Partial | delete 有 registry preflight，delete/relocate 有 Runtime-owned job/ticket/mutex；请求未冻结 expected generation，未统一 dirty/open-document/source-control/trash/undo/compensation。 | mutation coordinator 绑定 accepted preflight generation 和 durable terminal receipt。 |
| ED6-P1-029 | Partial | change hub 已有内部 sequence、bounded coalescing、queue age、wake 和 overflow full refresh；公开 record/delivery 没有 producer/source sequence/digest/gap/coalesced count。 | qualified external change 与显式 gap/resync receipt。 |
| ED6-P1-030 | Open | workspace 可在 remove 时清单选，局部 replace 要求 locator 不变；folder/history/focus/open toolkit/document/reference/preview 没有统一 identity reconcile。 | `AssetIdentityReconciler` 原子更新所有 browser/document/toolkit consumers。 |
| ED6-P1-031 | Partial | unchanged delta 可早退，shader refresh 只在相关 delta 执行，winner commit 有 generation fence；任何有效 change 仍 full-build index/catalog/folders/details/scheduler。 | delta -> provider/page/record incremental publication，subscriber 副作用解耦。 |
| ED6-P1-032 | Open | 有 chunk/virtualization/stale preview/project close/poison 等局部测试，但不能证明真实 double-click/Enter/Open With/create/rename/bulk/cancel/fault/recovery。 | host E2E 覆盖 input 到 Runtime durable receipt 再到 UI reconcile。 |

### 5.5 P1 机械汇总

| 状态 | 数量 |
|---|---:|
| Open | 22 |
| Partial | 10 |
| Closed | 0 |
| Total | 32 |

## 6. Editor248 `ED6-P2` currentness 结算

| ID | 状态 | 当前证据与剩余工作 |
|---|---|---|
| ED6-P2-001 | Partial | item/catalog/paint 有 Arc chunk 和物理 virtualization，但全 catalog/folder/string indexes 仍物化；缺 100k/1M、多 source memory/p95/p99 基线。 |
| ED6-P2-002 | Open | preview cache 仍无 byte accounting、quota、hit/miss/eviction/stale/negative/orphan metrics。 |
| ED6-P2-003 | Open | PNG 仍直接写最终路径，无 temp/rename/fsync/recovery receipt。 |
| ED6-P2-004 | Open | `MAX_PREVIEW_IN_FLIGHT=64` 仍是全局常量，不按 CPU/GPU/viewport/project profile。 |
| ED6-P2-005 | Open | query 仍逐 row `to_ascii_lowercase`。 |
| ED6-P2-006 | Open | optional details 仍不携带 reason/partial/stale state。 |
| ED6-P2-007 | Partial | change hub 内部已有 publish sequence、queue age、bounded collapse；delivery 不暴露 sequence/gap/coalesced/resync telemetry。 |
| ED6-P2-008 | Open | creation/action availability 仍未与 command registry、provider capability、toolkit generation 共用 cache。 |
| ED6-P2-009 | Open | source/import/preview 错误仍以 String 为主，缺稳定 code/parameters/localization。 |
| ED6-P2-010 | Partial | Browser paint virtualization 和共享 layout 已落地；keyboard navigation、a11y、localization 与 provider data contract 仍未闭环。 |
| ED6-P2-011 | Partial | 已有 stale completion、project deactivation、poison 局部测试；仍缺 multi-instance/plugin unload/crash/soak/fault matrix。 |
| ED6-P2-012 | Open | Runtime/Editor preview/import/catalog 没有统一 correlation id 和端到端 latency trace。 |

P2 汇总：**8 Open / 4 Partial / 0 Closed / 12 Total**。

## 7. 当前工作树新增 convergence 风险

| ID | 严重度 | 当前证据 | 归并 owner |
|---|---|---|---|
| EAW-D01 | P1 | exact type 在 catalog/workspace 退化，随后从 kind 反推，plugin asset 可能被路由到错误 toolkit。 | ED6-P1-001/014/022 |
| EAW-D02 | P1 | Runtime delta 只用于 unchanged 早退和 shader 判断，非空 delta 仍 full rebuild。 | ED6-P1-031 |
| EAW-D03 | P1 | `EditorAssetIndex` 的 local runtime registry revision 仍有 `wrapping_add` 路径，长期运行存在 identity reuse。 | ED6-P1-007 |
| EAW-D04 | P1 | preview job token 使用全局 relaxed `fetch_add`，没有 exhaustion/owner generation；不同 project 只能靠 catalog reset 间接隔离。 | ED6-P1-019/023 |
| EAW-D05 | P1 | Activity/Browser snapshot 从一份 state clone，两个 UI 面没有独立 navigation/selection/history。 | ED6-P1-009/012 |
| EAW-D06 | P1 | rendering virtualization 只减少物理节点，logical items、UUID/locator maps、lowercase filter 和 folder tree 仍全量构造。 | ED6-P1-004/010/011 |
| EAW-D07 | P1 | mailbox overflow 会静默折叠成 CatalogChanged；consumer 看不到 gap、丢失数或内部 publish sequence。 | ED6-P1-029 |
| EAW-D08 | P1 | preview image/placeholder 直接写最终 PNG，崩溃可留下截断缓存且 catalog 只看到普通 Error。 | ED6-P1-017/021；Runtime204 |
| EAW-D09 | P1 | preview key 缺 generator/type/profile，代码升级或色彩/DPI变化可命中语义过期文件。 | ED6-P1-018 |
| EAW-D10 | P1 | scheduler 的 visible set 不参与排序或 refill；没有等待队列 owner，admission available 只能依赖外部再次请求。 | ED6-P1-019 |
| EAW-D11 | P1 | generic import request 只有 URI+reason；model ticket 只有 poll/wait，Runtime 同步 import 期间没有 cooperative cancellation。 | ED6-P1-026/027；Editor257 |
| EAW-D12 | P1 | delete/relocate 只在调用 Runtime 前检查 cancel，accepted preflight 没有绑定 expected project/catalog generation。 | ED6-P1-028 |
| EAW-D13 | P1 | source/write authority 由 scheme 推断，Package/Library/remote/offline/mount replacement 无法给出真实 capability。 | ED6-P1-005/006 |
| EAW-D14 | P1 | Browser click/press/context menu 分别只做 select/drag/delete，activation 没有 terminal receipt。 | ED6-P1-014/015/016/025 |
| EAW-D15 | P1 | poisoned authority locks 被当作可继续使用的普通 state，没有 degraded/fence/rebuild。 | ED6-P1-024 |
| EAW-D16 | P2 | selection snapshot 已预留 toolkit/context command 字段，但当前由 workspace 写空字符串/空数组，UI 容易把“未接线”表现为“无能力”。 | ED6-P1-022；ED6-P2-006 |

## 8. 参考引擎差异落点

| 能力 | Unreal / Bevy / Fyrox / Godot / Unity Graphics 可见边界 | Zircon 当前差异 |
|---|---|---|
| Content source | Unreal Content Browser data source、Godot filesystem/resource UID、Bevy asset source/loader 将 provider/source 与逻辑路径分开。 | Zircon 仍由 URI scheme 推断 authority，catalog 不发布 mount/provider generation。 |
| Query/virtualization | Unreal Content Browser 以 data source/filter/item payload 提供增量条目；Godot filesystem dock 由文件系统索引驱动。 | Zircon 只 virtualize paint slots，data/query/folder projection 仍全量。 |
| Type/toolkit | Unreal asset definition/tools、Godot resource type/loader、Fyrox loader/resource state 保留 exact type 与操作 owner。 | Zircon type registry 有 exact type，但 catalog/workspace 降级为 kind。 |
| Async load/import | Unreal streamable handle/AssetManager、Godot threaded load token、Bevy AssetServer、Fyrox ResourceManager 都让请求 identity、状态和完成可观察。 | Zircon Editor import/preview 各有局部 ticket，但没有统一 asset operation/request authority。 |
| Reimport/mutation | Unreal AssetTools/ReimportManager 与 Unity Graphics reimport utilities 围绕资产/导入器/结果工作，不把 UI click 当事实 owner。 | Zircon 只有 model import、delete、relocate 三条专用路径，缺通用 options/preflight/undo/per-item receipt。 |
| Preview/cache | Unreal thumbnail/derived data 链围绕版本化生成器和缓存 owner；资源生命周期与 job owner 可被撤销/淘汰。 | Zircon UUID+source-hash PNG 目录无预算、lease、generator version 或 atomic transaction。 |

Unity Graphics 参考仓只覆盖 SRP Editor/Material reimport 等局部 consumer，不能单独作为通用 Content Browser 设计证明；通用 owner 主要由本地 Unreal、Godot、Bevy 和 Fyrox 源码交叉约束。

## 9. 目标架构与重构里程碑

### M267.0：冻结 authority 与 receipt vocabulary

定义 `ProjectAssetWorkspaceSession`、`ContentSourceProviderId`、`MountGeneration`、`QualifiedAssetRef`、`AssetCatalogPublicationReceipt`、`AssetOperationRequest/Receipt`。先规定 identity/currentness/terminal outcome，再改 UI。

### M267.1：exact type 与 provider publication

Runtime catalog record 发布 exact type/schema/compatibility、provider/mount/capability/health；Editor catalog/details/item/reference 全链保留。禁止从 `ResourceKind` 反推 plugin type。

### M267.2：paged catalog/query authority

将 Runtime delta 转成 provider/page/record changes；`CatalogQuerySession` 负责 typed filter/sort/cursor/cancel。workspace 和 paint 只保留可见页 + overscan，不再构造全量 logical item/folder tree。

### M267.3：per-instance workspace state

引入 `AssetBrowserInstanceId`、typed `NavigationTarget`、history、expanded branches、query、sort、multi-selection set/anchor/focus 和 stale receipt。Activity/Browser/弹出选择器不再共享一份 state。

### M267.4：统一 operation provider

把 create/import/reimport/rename/relocate/duplicate/delete/open/activate/bulk 收敛到 provider registry；所有操作执行 capability/preflight、冻结 expected generation、返回 per-item terminal receipt，并接 authoring undo/compensation。

### M267.5：preview artifact service

迁移到 Runtime204/205 的 root capability 和 resource/cache owner：typed variant key、generator version、content address、temp/atomic publish、byte lease、LRU、negative cache、last-good、orphan sweep、priority/deadline/cancel acknowledgement。

### M267.6：activation 与 identity reconcile

pointer、keyboard、menu、reference、drag/drop 共用 `AssetActivationIntent`；toolkit route 固定 exact type/toolkit generation/catalog generation。rename/remove/remap 由一个 reconciler 更新所有 browser、document、toolkit、reference 和 preview consumer。

### M267.7：fault/scale/performance acceptance

建立 100k/1M assets、多 source、多窗口、多 project switch、plugin unload、watch gap、stale completion、disk full、poison/crash/soak 测试；记录 memory、CPU、I/O、cache、queue 和 p50/p95/p99/p99.9。只有同 workload 证据成立后才讨论优于 Unreal。

## 10. 工程资格门

| Gate | 状态 | 通过条件 |
|---|---|---|
| EAW-G01 | Fail | catalog/workspace/reference 全链保留 exact asset type/schema/compatibility。 |
| EAW-G02 | Fail | provider/mount identity、generation、capability、health 可查询且不靠 scheme 猜测。 |
| EAW-G03 | Partial | Runtime project/catalog/editor publication 有 generation fence；仍需统一 receipt。 |
| EAW-G04 | Fail | 非空 Runtime delta 不再触发全 catalog/folder/details rebuild。 |
| EAW-G05 | Partial | Browser paint slot 已 virtualize；data/query/folder 仍需 page virtualization。 |
| EAW-G06 | Fail | Activity、Browser 和 selector 具独立 instance state。 |
| EAW-G07 | Fail | multi-select/anchor/focus/stale selection receipt 完整。 |
| EAW-G08 | Fail | typed navigation/history/favorites/collections/saved query 完整。 |
| EAW-G09 | Fail | double-click/Enter/Open With/reference 统一 activation receipt。 |
| EAW-G10 | Fail | Locate/Reveal 返回 browser/scroll/focus/filter terminal outcome。 |
| EAW-G11 | Fail | preview key 包含 type/schema/generator/profile/color/scale generation。 |
| EAW-G12 | Fail | preview cache 有 byte lease/LRU/quota/orphan sweep。 |
| EAW-G13 | Fail | preview publish atomic、可恢复且保留 last-good。 |
| EAW-G14 | Partial | preview 有 token/cancel check/stale fence；仍需 priority/deadline/ack。 |
| EAW-G15 | Fail | preview 输入持有 Runtime resource/provider snapshot lease。 |
| EAW-G16 | Fail | asset action availability 消费 provider/toolkit/plugin generation。 |
| EAW-G17 | Fail | create/import/reimport/rename/duplicate/delete/open/bulk 走统一 operation provider。 |
| EAW-G18 | Partial | import 有 bounded single-flight 和 Runtime receipt；仍缺 typed recipe/output/cancel。 |
| EAW-G19 | Partial | delete/relocate 已 Runtime-owned ticket；仍缺 accepted-generation/undo/compensation。 |
| EAW-G20 | Fail | external change delivery 暴露 producer/source sequence/digest/gap/resync。 |
| EAW-G21 | Fail | project close/reload cancel并 drain 所有 workspace child owner，返回 retirement receipt。 |
| EAW-G22 | Fail | poison/fault 进入 degraded/fenced recovery，不继续写半状态。 |
| EAW-G23 | Fail | host E2E 覆盖 input -> operation -> Runtime durable receipt -> UI reconcile。 |
| EAW-G24 | Fail | 100k/1M、多 source/fault/soak benchmark 达到预先定义的 correctness 和 tail-latency budget。 |

Gate 汇总：**19 Fail / 5 Partial / 0 Pass / 24 Total**。

## 11. 禁止的临时修补

1. 禁止在 workspace 用 `ResourceKind`、extension、文件名或 URI scheme 猜 exact type/provider/capability。
2. 禁止把 paint virtualization 宣称为 catalog/query virtualization。
3. 禁止新增第四条专用 import/mutation API 来绕过通用 operation provider。
4. 禁止以更大的 in-flight 常量、HashMap 或全量 clone 掩盖 page/owner/budget 缺失。
5. 禁止 preview 直接覆盖最终文件并把 `Ok(path)` 当 durable receipt。
6. 禁止在 poison 后无验证继续写 authority state。
7. 禁止用静态 success、空 toolkit fields 或空 command list 表示真实 action availability。
8. 禁止复制 Runtime205 load/cache/residency 或 Editor257 job scheduler 内核到 asset browser。

## 12. Review-only 交付与后续执行

- 本篇只修改 review 文档，不修改生产 Rust、Cargo、ABI、测试或 UI。
- Tooling 按用户要求排除；后续另行迁移 Rust。
- 未查询、轮询、等待或实时跟踪协调器；工具不可用点直接绕过。
- 实现前必须重查 focused manifest 指纹和 canonical owner 状态，避免旧结论覆盖并行变更。
- 推荐执行顺序：`M267.0 -> M267.1 -> M267.2 -> M267.3 -> M267.4 -> M267.5 -> M267.6 -> M267.7`。

## 13. 状态与输出

| 字段 | 值 |
|---|---|
| implementation_owner |  |
| implementation_branch |  |
| implementation_commit |  |
| validation_evidence |  |
| terminal_status |  |

