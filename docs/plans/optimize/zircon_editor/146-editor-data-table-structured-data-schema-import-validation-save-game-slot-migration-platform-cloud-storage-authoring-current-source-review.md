---
title: Editor Data Table / Structured Data / Schema / Import / Validation / SaveGame / Slot / Migration / Platform / Cloud Storage Authoring 当前源码复审
category: zircon_editor
report_id: Editor146
review_date: 2026-08-26
baseline_head: d4ca9a802ecd19976c653caa58614af0c2fb15f7
verification_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/98-editor-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-product-integration-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/52-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_runtime/src/asset/assets/data.rs
  - zircon_runtime/src/asset/importer/ingest/import_data_asset.rs
  - zircon_runtime/src/asset/artifact
  - zircon_plugins/asset_importers/data
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/platform/preferences
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime_interface/src/serialization
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Factories/DataTableFactory.cpp
  - dev/UnrealEngine/Engine/Source/Editor/DataTableEditor/Private/DataTableEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/DataTable.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SaveGameSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SaveGameSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameFramework/SaveGame.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/AsyncActionHandleSaveGame.h
  - dev/godot/core/io
  - dev/Fyrox/fyrox-core/src/visitor
  - dev/bevy/crates/bevy_scene
  - dev/bevy/crates/bevy_reflect/src/serde
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor146 - Data Table 与 SaveGame Authoring 当前源码复审

## 1. 最终结论

Zircon 当前仍没有 DataTable 产品，也没有 SaveGame 产品。全生产源码对 `DataTable`、`DataTableSchema`、`DataTableHandle`、`SaveGame`、`SaveParticipant`、`SaveEnvelope`、`PlatformSave`和 `CloudStorage`的精确搜索没有命中；唯一 DataTable 文件名仍是 Workbench ZUI。

已有底层不是空壳。`DataAsset`可承载 Text/TOML/JSON/YAML/XML 的 source text 与 canonical JSON，builtin/plugin importer、artifact hash/chunk/compression/atomic publication、typed `DataMarker` load facade 均真实存在。Session Archive 已有 569 个文件、11,400 行，包含 slot、lineage/revision、sealed artifact、manifest/index、retention、512 MiB cap、bounded writer、deadline/cancel-before-start 和 atomic single-file write。core durable transaction 也已深化到 owner lock、BLAKE3、journal frame、commit point、file/parent sync、rollback/recovery 与 fault tests。这些基础必须保留。

但 generic Data 仍只有 `uri/format/text/canonical_json`，没有 schema/row/field identity、typed value、reference、migration、validation、cooked table layout 或 runtime row lookup。`ResourceKind::Data`只有 placeholder presentation，没有 toolkit/factory；`ImportedAsset::direct_references`让 Data 落入默认空引用。它不能因能加载 JSON 就被命名为 DataTable。

SaveGame 侧的产品 owner 仍完全缺席。Session Archive 没有生产 consumer，格式固定为 v1，没有 strong integrity hash；`DefaultHasher`只用于进程内 slot bucket，不是持久 checksum。其 path CAS 是进程内 `HashMap`，没有跨进程/restart generation；restore 仍只写 `serializable && editable`字段，capture 与 restore participation 不一致。Session 虽使用 atomic single-file primitive，却没有接入更强的 durable transaction journal，也没有 player/platform-user/profile identity、participant、migration、cloud 或 transactional restore。

两份 Workbench 仍为 468 行、55 nodes、38 routes、0 provider，固定显示 `DT_Items`、`Schema_Item`、Potion rows、128 rows/2 warnings/512 refs，以及 `AutoSave_01`、`Manual_03`、`Cloud_02`、SaveData v4、LZ4。callback 继续返回 `Save queued`、`Load queued`和 local sample 文本，没有 document/service/job terminal receipt。

本轮重判 Editor24/98 的 **5 项 P0 为 4 Open/1 Partial，60 项 P1 为 46 Open/14 Partial，12 项 P2 为 11 Open/1 Partial；32 项资格门为 29 Fail/3 Partial**。Editor146 只刷新 currentness，不重复增加 canonical finding。没有运行 Data import/cook、Save/Load、kill point、跨进程、平台用户、cloud、fuzz、scale、soak 或同内容 benchmark，不能声称功能、性能或可靠性优于 Unreal。

## 2. 审查边界与 currentness

### 2.1 当前物理选择集

各范围说明独立 owner 边界，存在有意重叠，不直接相加为 union。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮用途 |
|---|---:|---|
| Editor Data/Save product boundary | **28 / 6,186 / 5,864 / 251,844 / 17 / 4** | 两份 ZUI、type registry、feedback/navigation/template binding 与 reference analysis |
| Data import/artifact selected | **36 / 9,550 / 8,715 / 342,701 / 54 / 7** | `DataAsset`、builtin/plugin importer、facade、load 与 artifact |
| Runtime Session Archive owner | **569 / 11,400 / 10,203 / 392,394 / 20 / 3** | archive/slot/manifest/index/merge/retention/io/writer 全目录 |
| Preferences/atomic/durable transaction/serialization | **85 / 14,768 / 13,393 / 485,699 / 132 / 6** | storage primitive、journal/recovery 与 versioned serialization |
| Selected reference union | **23 / 9,893 / 8,483 / 365,792 / 10 / 0** | Unreal/Godot/Fyrox/Bevy/Unity Graphics 对照 |

当前维护风险主要集中在共享 owner：artifact 中 `ibl_source_cubemap_staging.rs` 1,414 行、chunk residency 802 行、store 756 行；durable transaction tests 694 行；asset type registry 634 行；Session archive 568 行、artifact 542 行。Data/Save 产品不能继续往这些共享文件堆领域分支，必须以 typed schema/service contract 接入。

### 2.2 冻结点与限制

- baseline HEAD 为 `d4ca9a802ecd19976c653caa58614af0c2fb15f7`；本轮以 dirty working tree 的物理内容为准，最终 HEAD 如变化，以 `verification_head`与本报告源码断点为准。
- 选择集含大量用户或其他 Session 在途修改，尤其是 type registry、importer、artifact、Session secondary index 和 durable transaction 目录拆分。本轮不回退、不覆盖，也不把在途实现视为已集成资格。
- 参考 revision：Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`；均 clean。Unreal 无独立 nested Git，以所选文件内容为准。
- 按用户要求未查询、轮询、等待或实时跟踪协调器。
- 本轮只做静态 review，没有运行 Cargo、Editor、Data import/cook、Save/Load、断电、disk full、跨进程、platform user、cloud、fuzz、scale、soak 或 benchmark。

### 2.3 Owner 边界

- Editor146 负责 DataTable factory/toolkit/document/schema UX、import mapping/validation、SaveGame diagnostic/slot inspector 和真实 Workbench projection。
- Runtime85/86 负责通用 import/cook/artifact 与 exact asset type/schema/dependency；Editor 不得创建第二份 Data dependency graph。
- Runtime40 负责 SaveGame/Checkpoint service、participant、envelope、platform/cloud 总合同；Runtime52 只拥有 Session Archive，不是 SaveGame facade。
- Runtime61/63 负责 World snapshot/serialization 与 reflection type schema；Save participant 只能消费修复后的合同。
- Runtime45 与 core resource IO 提供 preferences 和 durable primitive；Save storage 必须通过 Runtime40 建立独立 user/slot/quota/lifecycle 语义。

## 3. 当前源码事实与断点

| 子链 | 当前真实基础 | 仍然断开的工程合同 |
|---|---|---|
| Data model | `DataAsset { uri, format, text, canonical_json }` | 不是 table schema、typed rows、stable identity 或 runtime lookup |
| Runtime load | `DataMarker`与 `load_data_asset`真实存在 | 只有 kind-typed asset，无 schema-qualified immutable handle |
| Registry | builtin Data presentation/placeholder | `builtin_toolkit`只覆盖 UI/Animation；Data 无 factory/toolkit/document |
| References | 通用 asset reference graph 真实存在 | `ImportedAsset::direct_references`中 Data 走 `_ => Vec::new()` |
| Builtin import | TOML/JSON用 `source_str()`借用解析，成功后复制 source；typed parse errors | `import_from_source`先 `fs::read`全文；无 Data bytes/depth/node/alias/scalar/time/memory/deadline budget |
| Plugin import | TOML/JSON/YAML/XML descriptor、priority/version、tests 存在 | plugin 仍复制全文；builtin/plugin 重复拥有 TOML/JSON，安装可改变语义 |
| XML | 单遍 child scan，element namespace 保留 | recursion 无 depth budget；text 与 elements 分栏，mixed order、attribute namespace、comment/PI 丢失 |
| Artifact | content hash、chunk、compression、size/read bounds 与 atomic publication | 无 DataTable schema/importer/compiler/dependency/layout header |
| Session Archive | slot/index/manifest/lineage/revision/retention/bounded writer 真实 | 固定 v1、无 strong hash、全量 canonical bytes、无 Save identity/participant/migration |
| Session write | path normalization、atomic stage/commit、generation check、deadline/admission | CAS 只在进程内 map；不使用 durable journal/recovery；product 提供裸 path |
| Restore | DynamicScene preflight 与 generation 基础 | restore 只应用 `serializable && editable`字段，不是 staged participant transaction |
| Core transaction | owner lock、BLAKE3、journal、commit point、sync、rollback/recovery | 属于正确共享 primitive，但 DataTable/Session/SaveGame 产品没有 consumer |
| Workbench | 稳定 control/route identity | 468 行/55 nodes/38 routes/0 provider，全是固定业务事实与 queued 文本 |
| Product symbols | generic Data/Session/preferences/transaction 可搜索 | DataTable/SaveGame/platform/cloud 生产类型与 caller 精确命中为零 |

ignored tests 主要是 borrowed parse、XML scan、secondary index、partition 和 microbenchmark evidence。它们可证明局部实现方向，不能证明输入安全、产品闭环或端到端性能。尤其不能用 1 MiB parser microbenchmark 替代 alias bomb、深 XML、超大 scalar、deadline/cancel 和 peak RSS 资格。

## 4. 必须保留的工程基础

1. 保留 generic `DataAsset`、builtin/plugin importer registration、typed errors 与 `DataMarker` facade，把 opaque Data 与 schema-bound DataTable 分层。
2. 保留 borrowed `source_str`优化并推广到 plugin，但必须在读取前加 Data-specific admission，成功路径的 bytes/String/DOM/value/artifact 要纳入 peak accounting。
3. 保留 artifact content hash、chunk、compression、bounded read 与 atomic publication，DataTable 另加 exact schema/dependency/layout header。
4. 保留 Session Archive slot、dense index、lineage/revision、sealed artifact、manifest、retention 与 bounded writer，只作为显式 Save participant 或开发期 session product。
5. 保留 core durable transaction 当前 owner lock、BLAKE3、journal、commit point、sync、recovery 和 observation，PlatformSaveStorage 必须复用它。
6. 保留 preferences 的 typed capacity/permission/corrupt/transient error 语义，但不直接承载大存档。
7. 保留 Workbench control/route identity；真实 provider 未安装时显示 Unavailable，禁止 fixture fallback。

## 5. P0：产品真实性、数据安全与输入安全

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| DATAED-P0-01 | Open | 两份 Workbench 固定 table/slot/schema/count/cloud，callback 只改 control 或回 queued 文本 | 删除 production fixture fallback；Data 页只开 canonical document，Save 页只投影 service receipt，缺 provider 时 disable |
| DATAED-P0-02 | Open | generic Data 无 row schema/key/field/type/default/reference/migration/runtime accessor | 建 DataTableSchema/Document/Compiler/CookedArtifact/Handle 独立产品，不在 JSON key 上堆约定 |
| DATAED-P0-03 | Open | engine 内无 SaveGame service、slot repository、participant、migration、platform/cloud authority | Runtime40 先实现唯一 service/capability/receipt，Editor 再接 diagnostic 与 slot inspector |
| DATAED-P0-04 | Open | Session Archive 无 product consumer，restore/durability/identity 不满足 SaveGame；直接接线会静默遗漏或部分恢复 | 以显式 participant 消费 Runtime61/63，建立 Save envelope、transactional restore 与 platform store；禁止改名 |
| DATAED-P0-05 | Partial | borrowed parse 与 XML 单遍扫描降低局部成本；完整读取、无 budget、YAML/XML 有损/递归风险仍在 | 读取前 bytes gate；parse 阶段 depth/node/alias/scalar/time/memory/cancel；XML 要么保序 typed tree，要么 opaque read-only |

## 6. P1：Data Schema、Document、Import 与 Runtime

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| DATAED-P1-01 | Open | 无 stable DataTable schema ID/version/fingerprint/owner；建立 registry 与 unload generation |
| DATAED-P1-02 | Open | field 只有 JSON key；需要 stable ID、alias、type、default、deprecation 与 source span |
| DATAED-P1-03 | Open | row key 无 primary/composite/generated policy、Unicode/case 与 rename redirect |
| DATAED-P1-04 | Open | value 压成 JSON；建立 bool/int/decimal/text/enum/vector/reference/array/map/struct typed value |
| DATAED-P1-05 | Open | missing/default/null/invalid/unknown 混合；各阶段保持五态语义 |
| DATAED-P1-06 | Open | enum/tag/vocabulary 无 versioned registry linkage 与 migration |
| DATAED-P1-07 | Open | asset/row/localization/tag reference 无 typed locator、strength 和 dependency identity |
| DATAED-P1-08 | Open | unknown row/field 无 forward-compatible source preservation |
| DATAED-P1-09 | Open | schema evolution 无 compatibility classification、impact 与 required migration |
| DATAED-P1-10 | Partial | 有 `DataMarker` generation load 基础；缺 schema-qualified immutable row handle、lookup error 和 hot-reload receipt |
| DATAED-P1-11 | Open | Data 无 factory/toolkit；Create/Open/Save/Reimport/Cook 链不存在 |
| DATAED-P1-12 | Open | 无 transactional DataTableDocument、dirty/history/revision/selection/validation |
| DATAED-P1-13 | Open | 无 row add/delete/rename/duplicate 与 reference decision |
| DATAED-P1-14 | Open | 无 typed rectangular clipboard、header mapping、partial-error preview 和 single undo |
| DATAED-P1-15 | Open | 无 sort/filter/search query owner 与 row/column virtualization |
| DATAED-P1-16 | Open | 无 multi-cell fill/replace/convert 及 bounded expression sandbox |
| DATAED-P1-17 | Open | details 不是 schema-driven property editor，无 Reset/unknown/default state |
| DATAED-P1-18 | Open | schema edit 与 row edit 无分离 impact/migration/cross-document transaction |
| DATAED-P1-19 | Open | 无 base revision、external edit、多实例 CAS/merge/save-copy |
| DATAED-P1-20 | Open | 无 stable row/field ID 驱动的 diff、review 与 source-control conflict UX |
| DATAED-P1-21 | Open | 无 CSV/TSV/spreadsheet import/export、encoding/delimiter/schema mapping |
| DATAED-P1-22 | Partial | importer descriptor 有 priority/version 和 selection tests；builtin/plugin 仍重复拥有 TOML/JSON且安装可改语义 |
| DATAED-P1-23 | Open | YAML alias/tag/merge/duplicate-key support 与 budget 未声明 |
| DATAED-P1-24 | Partial | XML 保留 element namespace 并单遍扫描；mixed-content order、attribute namespace、comment/PI 仍丢失 |
| DATAED-P1-25 | Open | TOML/JSON datetime、integer/float precision、duplicate/order policy 压入 JSON 最小公分母 |
| DATAED-P1-26 | Partial | builtin failure path 少一次 String copy；plugin/success path 仍多份常驻，无 peak accounting |
| DATAED-P1-27 | Open | 无 Data validation rule registry、span、fix、cost 与 incremental execution |
| DATAED-P1-28 | Open | Data `direct_references`为空；row/asset/localization/tag dependency 不进入 runtime graph |
| DATAED-P1-29 | Partial | generic artifact 有 hash/kind/revision/chunk/size；缺 DataTable schema/importer/compiler/dependency/layout header |
| DATAED-P1-30 | Open | 无 row/column/key index/chunk/locale strip 等 runtime layout 与 lookup baseline |

## 7. P1：SaveGame、Platform、Cloud 与产品资格

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| DATAED-P1-31 | Open | 无 Save envelope magic/game/build/user/profile/slot/generation/catalog identity |
| DATAED-P1-32 | Open | 无 stable participant/type ID/version/fingerprint 与 owner lease |
| DATAED-P1-33 | Open | 无显式 capture scope/phase/dependency/required/privacy/budget policy |
| DATAED-P1-34 | Partial | DynamicScene 有 isolated preflight/generation；Session restore 仍非 staged participant transaction/rollback |
| DATAED-P1-35 | Partial | generic serialization 与 DynamicScene 有 migration；Save envelope/per-type graph、dry run 与 historical fixture 缺失 |
| DATAED-P1-36 | Open | unknown optional/required participant 无 opaque preserve 与 fail policy |
| DATAED-P1-37 | Open | build/DLC/mod/plugin compatibility catalog 与 load plan 不存在 |
| DATAED-P1-38 | Open | script save schema、field ID、migration 与 VM safety boundary 不存在 |
| DATAED-P1-39 | Open | persistent entity/asset/soft/external reference rebind rule 未定义 |
| DATAED-P1-40 | Open | simulation tick、RNG、timer、clock 与 pending command 保存顺序未定义 |
| DATAED-P1-41 | Open | 无 PlatformUser/Profile/controller mapping 与 login switch lifecycle |
| DATAED-P1-42 | Partial | Session slot ID、metadata 和 path 有局部分层；SaveGame opaque slot/user/backend path contract 仍不存在 |
| DATAED-P1-43 | Partial | Session 有 bounded writer submission；Save capture/encode/write/cloud/load 无统一 cancel/progress/deadline/receipt |
| DATAED-P1-44 | Open | 无 autosave scheduler、coalescing、checkpoint policy 与 lifecycle deadline |
| DATAED-P1-45 | Partial | preferences 有 capacity/permission error；SaveGame 无 slot/count/bytes/temp/free-space preflight |
| DATAED-P1-46 | Partial | core durable transaction 现有 journal/recovery/strong digest 基础；Session 与 SaveGame 未消费，双 generation/LKG 未建立 |
| DATAED-P1-47 | Open | compression 只是 UI 固定 LZ4；无 algorithm/version/chunk/uncompressed-size/bomb policy |
| DATAED-P1-48 | Open | 无 encryption/authentication/tamper/privacy 与 secure key lifecycle |
| DATAED-P1-49 | Open | 无 cloud provider、etag/base generation、offline journal、retry/idempotency |
| DATAED-P1-50 | Open | 无保留 local/remote/base 的 conflict UX 与 schema-aware policy |
| DATAED-P1-51 | Open | opaque Data、DataTable 与 SaveGame 共用 Data 外观，capability/permission boundary 不清 |
| DATAED-P1-52 | Open | static Workbench 与未来 toolkit/service 存在双入口；必须投影同一 owner |
| DATAED-P1-53 | Open | route/feedback 无 request/document/storage generation 与 provenance |
| DATAED-P1-54 | Partial | Session writer 有 task/retained-byte admission；Data import/cook 和 Save capture 无统一 resource admission |
| DATAED-P1-55 | Open | 无 10K/100K/1M row open/filter/edit/validate/save/cook/lookup budget |
| DATAED-P1-56 | Partial | Session 512 MiB cap/writer limits 真实；无 frame slicing、peak、compression/temp-disk qualification |
| DATAED-P1-57 | Partial | asset/archive/preferences 有 typed error 片段；无统一 schema/row/slot/participant/stage diagnostic/fix |
| DATAED-P1-58 | Open | tests 证明 parser/artifact/archive/route/primitive，不证明两条 product closure |
| DATAED-P1-59 | Open | 无 released DataTable/Save format golden corpus 与 N-2/N-1 compatibility matrix |
| DATAED-P1-60 | Open | maturity/Workbench promise 不由 required product/platform/scale gate 生成 |

## 8. P2：完整性、诊断与维护性

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| DATAED-P2-01 | Open | opaque Data、DataTable、runtime config 与 Save payload 术语混用 |
| DATAED-P2-02 | Open | schema/row/field/slot ID 仍以裸 string 传播 |
| DATAED-P2-03 | Partial | plugin manifest 有 capability/priority/version；仍缺 builtin/plugin unified authority matrix 与 compat check |
| DATAED-P2-04 | Open | runtime artifact 默认携带完整 source text，缺 source/debug/shipping retention policy |
| DATAED-P2-05 | Open | validation/participant diagnostic 无 bounded query/pagination/virtualization |
| DATAED-P2-06 | Open | slot thumbnail 无独立 job/generation/failure degradation contract |
| DATAED-P2-07 | Open | schema/slot path/display 的 Unicode/case/reserved policy 未集中 |
| DATAED-P2-08 | Open | wall timestamp 不能代替 generation/etag/device identity |
| DATAED-P2-09 | Open | telemetry 无 row/player/path/token privacy 与 redaction boundary |
| DATAED-P2-10 | Open | 569-file Session facade 组合爆炸，SaveGame 不得再复制一层 |
| DATAED-P2-11 | Open | scope/SaveGame symbol/Data toolkit/fixture 无 automatic stale gate |
| DATAED-P2-12 | Open | serialization/resource storage/scene snapshot 仍易被误写成 DataTable 或 SaveGame product |

## 9. 参考引擎差异与采用路由

| 参考 | 本轮源码证据 | Zircon 采用边界 |
|---|---|---|
| Unreal DataTable | Factory 绑定 `RowStruct`；`UDataTable`持 RowMap 并提供 typed `FindRow<T>`；Editor 映射 copy/paste、row create/rename/delete、transaction 与 reimport | DataTable schema/identity/runtime lookup 与完整 toolkit 主参考 |
| Unreal SaveGame | `ISaveGameSystem`提供 platform user、exists/list/save/load/delete 与 async callback；async action 把 slot/user/success 交回游戏线程 | Save service/user/slot/async/version header 主参考，并在 cancel/cloud/durability 上提高门槛 |
| Godot | ResourceSaver、`user://` FileAccess、ConfigFile/JSON 与 encrypted file 提供 resource/user storage 组合基础 | 证明底层 storage/serialization 不自动等于 SaveGame，不降低 participant/migration 门槛 |
| Fyrox | Visitor 使用 versioned tree、Reader/Writer 与 Region 组织对象图序列化 | 采用 versioned visitor/region 思路，不把 scene serialization 当 player save product |
| Bevy | Scene/ResolvedScene/Reflect serde 依赖 registered types，并有显式 resolve/apply/spawn error | world participant projection 参考，不替代 participation policy、slot service 与 transaction |
| Unity Graphics | 本地只有 SerializableEnum/SerializedDictionary 等 package container | 只作 serialized container 旁证；没有 DataTable/SaveGame 权威源码，不推测闭源产品 |

## 10. 目标架构与重构顺序

```text
DataTableSource
  -> LosslessDataTableDocument(schema_version, stable row/field ids)
  -> EditorTransaction + source revision CAS
  -> BoundedImport/Validation/DependencyReceipt
  -> ImmutableCookedDataTableGeneration
  -> RuntimeDataTableHandle<RowSchema>

SaveRequest(game, platform_user, profile, slot, base_generation)
  -> ParticipantCaptureTransaction
  -> VersionedEnvelope(manifest, strong hashes, bounds, protection)
  -> PlatformSaveStorage(DurableTransaction, quota, journal, recovery)
  -> CloudConditionalSync(base, local, remote)
  -> StagedMigrationAndRestoreTransaction
  -> TerminalReceipt
```

| 阶段 | 必须交付 | 关闭范围 |
|---|---|---|
| R0 Truth/input gate | Workbench fail-close；Data bytes/depth/node/alias/scalar/time/memory budgets；historical corpus | P0-01/05 前置 |
| R1 Data identity | stable schema/row/field/reference IDs、typed values、unknown preservation、migration 与 artifact header | P0-02、P1-01~10 |
| R2 Data product | factory/toolkit/document/undo/clipboard/virtualization/CAS save、bounded import/validation/cook | P1-11~30 |
| R3 Runtime Data | immutable typed handle、index/hot reload、first gameplay consumer | Data lifecycle gates |
| R4 Save contract | Save service/envelope/participant/schema/migration/catalog、explicit Runtime61/63 adapters | P0-03/04、P1-31~40 |
| R5 Platform storage | user/profile/slot、quota、durable transaction、async enumerate/read/write/delete、transactional load | P1-41~48 |
| R6 Cloud/lifecycle | autosave/coalesce、suspend/quit、protection、etag/offline/conflict/privacy | P1-44、47~50 |
| R7 Product/qualification | real toolkit/slot inspector，删除 fixture；history/fuzz/kill/cross-process/platform/scale/soak | P1-51~60、全部 P2/Gate |

MVP `00` 与 F0-F5 未通过前，可以先实施 R0 输入封堵与 truth hard cut，但不能先把 Session Archive 接到按钮，或给 generic Data 增加表格外观后宣称产品完成。

## 11. G01-G32 资格门

| Gate | 状态 | 当前证据与通过条件 |
|---|---|---|
| G01 DataTable lifecycle | Fail | 默认产品无 factory/toolkit/document |
| G02 Opaque vs table | Fail | 只有 coarse Data kind |
| G03 Stable rename | Fail | schema/row/field ID 与 migration 缺失 |
| G04 Value semantics | Fail | JSON 不能保持 missing/default/null/unknown contract |
| G05 Table transactions | Fail | 无 row/cell/schema command |
| G06 External conflict | Fail | 无 DataTable source revision owner |
| G07 Format matrix | Fail | 无 schema-bound support/reject/span matrix |
| G08 Input bombs | Fail | source/depth/node/alias budget 缺失 |
| G09 Import peak/cancel | Fail | full read/multiple copies，无 admission/cancel |
| G10 Importer selection | Partial | priority/version/selection tests 存在；plugin 安装仍可改变 TOML/JSON semantics |
| G11 Validation | Fail | 无 Data rule registry 与 incremental/full job |
| G12 Reference manifest | Fail | Data direct references 为空 |
| G13 Cooked table | Partial | generic artifact header 真实；DataTable exact schema/dependency/generation 缺失 |
| G14 Typed lookup | Fail | 只有 generic DataAsset load |
| G15 Table scale | Fail | 无目标规模动态证据 |
| G16 Save service | Fail | service/product symbols 缺席 |
| G17 Save envelope | Fail | 无 Save magic/identity/catalog/protection header |
| G18 Participants | Fail | registry/phase/policy 缺失 |
| G19 World completeness | Fail | 未证明 plugin/typed component 无损 capture/restore |
| G20 Migration | Fail | 无 Save historical graph/fixture |
| G21 Unknown participant | Fail | required/optional/opaque policy 缺失 |
| G22 Transactional load | Fail | Session restore 不是 participant staging/rollback |
| G23 Reference rebind | Fail | persistent vs process handle rule 缺失 |
| G24 Autosave frame budget | Fail | scheduler/coalesce/frame evidence 缺失 |
| G25 Storage fault | Partial | core journal/recovery/durable primitive 真实；Save slot LKG 全阶段 fault 证据缺失 |
| G26 Quota | Fail | 无 Save capture 前后空间/platform limit contract |
| G27 Compression | Fail | UI 固定 LZ4，无 envelope/bomb policy |
| G28 Protection | Fail | secure provider/key lifecycle 缺失 |
| G29 Cloud conflict | Fail | etag/offline/multi-device authority 缺失 |
| G30 Lifecycle terminal | Fail | suspend/sign-out/quit request state machine 缺失 |
| G31 Required evidence | Fail | 无两条 product E2E/history/fuzz/fault/scale lane |
| G32 Workbench truth | Fail | 55 nodes/38 routes 仍固定业务事实与 success fallback |

## 12. 禁止的临时修补

- 不得给 `ResourceKind::Data`挂 JSON text/grid editor 后宣称 DataTable 完成。
- 不得把 schema、row key、field type 藏在 convention JSON keys 而无 stable identity。
- 不得用 `split(',')`实现 CSV，也不得通过提高 artifact cap 代替 admission budget。
- 不得把有损 XML-to-JSON 称为 canonical reversible representation。
- 不得把 Save Data route 接到 Session Archive 并重命名 SaveGame。
- 不得默认序列化整个 World、VM heap 或 plugin state；participant 必须显式注册并版本化。
- 不得使用 Rust type name、`TypeId`、module path 或 temporary entity handle 作 persistent identity。
- 不得在 UI/game thread 同步 capture world、compress、write 或 cloud upload。
- 不得先删除旧 slot 再写新 slot；新 generation 验证完成后才能切 current。
- 不得用 internal slot hash 冒充 content integrity，也不得用 checksum 冒充 encryption/authentication。
- 不得在 terminal receipt 前显示 Save/Load/Cloud success，也不得保留 fixture fallback。
- 不得以 ignored microbenchmark、test attribute 数量或 generic primitive 证明 product closure。

## 13. 验证边界与裁决

| Canonical 范围 | 当前状态 | 本轮裁决 |
|---|---:|---|
| 5 项 P0 | **4 Open / 1 Partial** | Workbench truth、DataTable、SaveGame、Session misuse 全未关闭；只有 importer 局部优化为 Partial |
| 60 项 P1 | **46 Open / 14 Partial** | Data/Session/storage primitive 真实，但 schema/document/service/participant/cloud/product 链缺席 |
| 12 项 P2 | **11 Open / 1 Partial** | plugin manifest 有局部成熟度，其余 maintenance/diagnostic/privacy/staleness 未闭环 |
| 32 项 Gate | **29 Fail / 3 Partial** | 仅 importer selection、generic artifact 与 core storage primitive 提供局部证据 |

当前应把该域定义为“generic Data、Session Archive 与 durable storage 底层较深，但 DataTable/SaveGame 产品为零”。第一优先级不是添加更多静态按钮，而是 R0 truth/input gate；随后先建立 Data identity/document，再建立 Save identity/participant/envelope，最后通过统一 durable storage、cloud 与产品资格闭环。

只有 32 个 Gate 全部通过，并完成 historical fixture、fuzz、kill point、cross-process、platform/cloud、large table/save、soak 与 same-content benchmark，才可声称该域达到工程级；在此之前不得声称功能、性能或可靠性优于 Unreal。
