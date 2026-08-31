---
title: Editor Data Table / Structured Data / Schema / Import / Validation / SaveGame / Slot / Migration / Platform / Cloud Storage Authoring 与 Product Integration 当前源码复审
category: zircon_editor
report_id: Editor98
review_date: 2026-08-26
baseline_head: a8eca85cc83008aeb200dce2d2b01e2ae3c157c9
verification_head: 38c0e7f5d48189ac2637ed010e452b19c32f459d
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
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

# Editor98 · Data Table、SaveGame 与 Storage Product Integration 当前源码复审

## 1. 结论

Zircon当前仍没有DataTable产品，也没有SaveGame产品。`DataAsset`是真实可加载资产，但只包含URI、Text/TOML/JSON/YAML/XML格式、完整source text和`serde_json::Value`；它没有table schema、stable row/field identity、typed reference、migration、validation或runtime row lookup。Editor builtin registry给`ResourceKind::Data`生成placeholder presentation，却不提供factory或toolkit。`ResourceHandle<DataMarker>`和`load_data_asset`证明generic Data可被typed asset facade加载，不等于schema-qualified `DataTableHandle<Row>`。

结构化导入有局部进展。builtin TOML/JSON在成功解析前借用`source_str()`，减少一次失败路径String复制，并增加release-only P95 gate；plugin XML由两次child扫描改为一次，保留element namespace。可是`import_from_source`仍先`fs::read`整个文件，Data路径没有source bytes、depth、node、alias、scalar、CPU、allocation或deadline预算。plugin TOML/JSON/YAML/XML仍调用`source_text()`复制全文，raw text与canonical JSON长期双持有。XML仍把text与element拆成两个集合，丢mixed-content顺序；attribute仅用local name，comment和processing instruction也消失。ignored microbenchmark不构成输入安全资格。

SaveGame侧没有任何engine-owned service、participant registry、platform-user/profile identity、Save envelope、migration planner、cloud provider或产品caller。`git grep`的SaveGame产品符号为零；唯一可见Save Data路径是Workbench route。Runtime40是产品合同报告，Runtime52是Session Archive owner报告，它们不是实现。当前Session owner已有568个production文件、dense slot indexes、lineage/revision、sealed artifact、512 MiB上限、manifest、retention和bounded writer，这些是可保留基础；core atomic file又新增`atomic_write_new`/`commit_new`，可保证不覆盖已有目标并在并发发布时只产生一个winner。但Session仍以裸Path/load-all-mutate-save为主，持久checksum用`DefaultHasher`，CAS只存在进程内map，atomic路径缺完整Save slot崩溃协议，restore不是participant transaction，且普通产品consumer为零。

两份Workbench仍是静态第二authority：共468行、55个node、38条route、0 provider。Data Table固定`DT_Items`、`Schema_Item`、Potion/Sword/Armor/Debug rows、128 rows、2 warnings、512 refs；Save Data固定`AutoSave_01`、`Manual_03`、`Cloud_02`、SaveData v4和LZ4。callback继续返回`Save queued`、`Load queued`等预制文本，field handler只修改retained control。近期只移除了Save按钮的错误selected/checked外观，没有增加document、service或receipt。

本轮重判Editor24的 **5项P0为4 Open/1 Partial，60项P1为46 Open/14 Partial，12项P2为11 Open/1 Partial；32项资格门为29 Fail/3 Partial**。Editor98只刷新currentness，不重复登记canonical finding。没有动态、规模、断电、跨进程、平台、云或同内容benchmark，不能声称功能、性能或可靠性优于Unreal。

## 2. 审查边界、统计与currentness

### 2.1 冻结范围

统计对象是当前working tree物理文件。行、非空行和bytes按物理内容统计；tests/ignored只计Rust属性。fingerprint按repository-relative lowercase path排序，为每个文件拼接path、NUL、lowercase文件SHA-256与LF后再取SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Editor产品、asset registry与Workbench | **28 / 5,818 / 5,523 / 259,927 / 5 / 0** | 439c1c4c51fc6b875a2a7f55f531ce4c260bd92c2f9fda5bd7f897f957cd76e7 |
| Data import与artifact selected | **43 / 12,390 / 11,284 / 446,754 / 107 / 7** | 9729e2e136f3bb771d56cb435a1db58018b10e12ee67e6f93660ff7e9757a97c |
| Session Archive与focused support | **591 / 18,162 / 16,416 / 641,022 / 128 / 3** | 672f53ffdcb4d133bba7269884e3f0fa92cfbccc432438217983e140c323a76d |
| Platform preferences、atomic transaction与serialization | **85 / 14,763 / 13,388 / 485,719 / 132 / 6** | df43b307264f7b0ed036029c62d81606d9d4b99227b3c1d9122ae6a3beaee300 |
| Zircon selected union | **747 / 51,133 / 46,611 / 1,833,422 / 372 / 16** | c24e7281faf098168bd0d7e9ef2c1f0707ddeda3ea0c68c6c1cc59e1e1108527 |
| Unreal selected | **7 / 3,348 / 2,777 / 114,408 / 0 / 0** | d5620d55451cef0d27ee7dc928e2bb66849449e28a234bbf4fd5a1d1a2cc656a |
| Godot selected | **7 / 2,336 / 1,889 / 86,272 / 0 / 0** | 62e355ea492dc1f00133a7a90c9122bb4646ca173c369db670b92a1872fcef02 |
| Fyrox selected | **3 / 1,204 / 1,087 / 46,971 / 2 / 0** | 49702b6c9d9289275e638374a32c9ed0e6ec7684603830623813fbaa78f266a5 |
| Bevy selected | **4 / 2,828 / 2,575 / 112,076 / 8 / 0** | a66620fceaa43a9893ae0284c8d0796e5d4e26da36bd9680a61aae015ee98e06 |
| Unity Graphics selected | **2 / 177 / 155 / 6,065 / 0 / 0** | f2a7ca472afc5a068055275384fa0bbcacc6d74023f66e15ece97f0d3221364d |
| Five-engine reference union | **23 / 9,893 / 8,483 / 365,792 / 10 / 0** | 86e0ffc673d8e1777645c26a2c12ebbf0882dc5f1e9fb573eb759b56852a8520 |

### 2.2 currentness与限制

- baseline与初始verification HEAD均为`a8eca85cc83008aeb200dce2d2b01e2ae3c157c9`；最终verification HEAD为`38c0e7f5d48189ac2637ed010e452b19c32f459d`，其新增路径不在本轮源码选集内；最终currentness以本表物理fingerprint为准。
- selected union有60个用户或其他Session在途项，包含Data importer、asset registry、Session secondary index、atomic transaction目录拆分与`atomic_write_new`发布语义、preferences与serialization优化。本轮不回退、不覆盖，也不把在途代码当已集成资格。
- 参考revision：Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`；Unreal以所选文件fingerprint冻结。
- 按用户要求未查询、轮询或等待协调器；Tooling排除。
- 本轮仅静态review，未运行Cargo、Editor、Data import/cook、Save/Load、断电、磁盘满、跨进程、platform user、cloud、fuzz、scale、soak或benchmark。

### 2.3 Owner边界

- Editor98唯一负责DataTable factory/toolkit/document/schema UX、import mapping/validation、SaveGame diagnostics/slot inspector和真实Workbench projection。
- Runtime85/86唯一负责通用import/cook/artifact、exact asset type/schema/dependency；Editor不得创建第二份Data dependency graph。
- Runtime40唯一负责SaveGame/Checkpoint service、participant、envelope、platform/cloud总合同；Runtime52只拥有Session Archive，不是SaveGame facade。
- Runtime61/63分别拥有World snapshot/serialization和reflection type schema；SaveGame participant只能消费修复后的合同。
- Runtime45和core atomic transaction提供小型preferences与通用durability primitive；SaveGame storage必须通过Runtime40建立独立容量、用户、slot和生命周期语义。

## 3. 当前产品链事实

| 层 | 当前事实 | 判定 |
|---|---|---|
| Data model | `DataAsset { uri, format, text, canonical_json }` | generic opaque Data，不是DataTable |
| Data runtime load | `ResourceHandle<DataMarker>`与`load_data_asset`真实存在 | kind-typed asset基础，不是row schema/lookup |
| Data registry | builtin Data presentation + placeholder，无toolkit | 产品不可创建/打开编辑 |
| Data references | `ImportedAsset::direct_references`对Data走默认空 | row/asset/localization依赖不可见 |
| Builtin import | TOML/JSON借用source parse，成功后才复制；Text仍复制 | 局部memory改善 |
| Plugin import | TOML/JSON/YAML/XML均复制source；priority 100、version 1 | duplicate authority仍存在 |
| XML projection | 单遍child扫描，element namespace保留 | mixed order/attribute namespace/comment/PI仍丢失 |
| Input admission | `fs::read`完整文件后才构造context | 无Data source/parse budget |
| Artifact | hash/chunk/compression/size/atomic基础真实 | 没有DataTable exact schema/dependency header |
| Atomic publication | `atomic_write_new`/`commit_new`不覆盖已有目标；并发测试验证single winner | 可保留primitive，不是Save slot generation/last-good协议 |
| Session Archive | 568 production files，slot/index/artifact/retention/writer真实 | 孤立组合facade，非SaveGame产品 |
| SaveGame symbols | engine/runtime/product service精确命中为0 | 产品缺席 |
| Data Workbench | 230行、27 nodes、19 routes、0 provider | 固定table/rows/counts |
| Save Workbench | 238行、28 nodes、19 routes、0 provider | 固定slot/schema/compression/cloud |
| Callback | fixed queued/opened/selected strings | 无document/service/job/receipt |

## 4. 必须保留的工程基础

1. 保留generic `DataAsset`、builtin/plugin importer注册、typed errors和`DataMarker` load facade，但把opaque Data与schema-bound DataTable分层。
2. 保留borrowed `source_str`优化，推广到plugin并在读取前增加Data-specific budget；ignored microbenchmark不得替代required safety test。
3. 保留artifact content hash、chunk、compression、bounded read与atomic publication，DataTable另加exact schema、dependency和layout header。
4. 保留Session Archive dense index、lineage/revision、sealed artifact、manifest、retention和bounded writer，只作为显式SaveGame participant或checkpoint payload基础。
5. 保留preferences与core atomic transaction的stage/commit/recovery方向，以及`atomic_write_new`/`commit_new`的no-replace single-winner发布语义；由PlatformSaveStorage消费，不复制弱化版。
6. 保留Workbench稳定control/route identity，但删除fixture authority；provider缺失时必须Unavailable。

## 5. P0：产品真实性、数据安全与输入安全

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| DATAED-P0-01 | Open | 两份Workbench固定table/slot/schema/count/cloud，callback只改control或回queued文本 | 删除生产fixture fallback；Data页只开canonical document，Save页只投影service receipt，缺provider时disable |
| DATAED-P0-02 | Open | generic Data没有row schema/key/field/type/default/reference/migration/runtime accessor | 建DataTableSchema/Document/Compiler/CookedArtifact/Handle完整独立产品，不在JSON key上堆约定 |
| DATAED-P0-03 | Open | engine内无SaveGame service、slot repository、participant、migration、platform/cloud authority | Runtime40先实现唯一service/capability/receipt，Editor再接diagnostic与slot inspector |
| DATAED-P0-04 | Open | Session Archive无产品consumer且restore/durability/identity不满足SaveGame；直接接线会静默遗漏或部分恢复 | 以显式participant消费Runtime61/63，建立Save envelope、transactional restore与platform store；禁止改名 |
| DATAED-P0-05 | Partial | borrowed parse与XML单遍扫描降低部分成本；完整读取、无budget、YAML/XML有损/递归风险仍在 | 读取前bytes gate，parse阶段depth/node/alias/scalar/time/memory/cancel；XML要么保序typed tree，要么opaque read-only |

## 6. P1：Data Schema、Document、Import 与 Runtime

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| DATAED-P1-01 | Open | 无stable DataTable schema ID/version/fingerprint/owner；建立registry与unload generation |
| DATAED-P1-02 | Open | field只有JSON key；需要stable ID、alias、type、default、deprecation和source span |
| DATAED-P1-03 | Open | row key无primary/composite/generated策略、Unicode/case与rename redirect |
| DATAED-P1-04 | Open | value压成JSON；建立bool/int/decimal/text/enum/vector/reference/array/map/struct typed value |
| DATAED-P1-05 | Open | missing/default/null/invalid/unknown混合；各阶段保持五态语义 |
| DATAED-P1-06 | Open | enum/tag/词表无versioned registry linkage与migration |
| DATAED-P1-07 | Open | asset/row/localization/tag reference无typed locator、strength和dependency identity |
| DATAED-P1-08 | Open | unknown row/field无forward-compatible source preservation |
| DATAED-P1-09 | Open | schema evolution无compatibility分类、impact与required migration |
| DATAED-P1-10 | Partial | 有`DataMarker` generation load基础；缺schema-qualified immutable row handle、lookup error和hot reload receipt |
| DATAED-P1-11 | Open | Data无factory/toolkit；Create/Open/Save/Reimport/Cook链不存在 |
| DATAED-P1-12 | Open | 无transactional DataTableDocument、dirty/history/revision/selection/validation |
| DATAED-P1-13 | Open | 无row add/delete/rename/duplicate与引用决策 |
| DATAED-P1-14 | Open | 无typed rectangular clipboard、header mapping、partial-error preview和single undo |
| DATAED-P1-15 | Open | 无sort/filter/search query owner与row/column virtualization |
| DATAED-P1-16 | Open | 无multi-cell fill/replace/convert及bounded expression sandbox |
| DATAED-P1-17 | Open | details不是schema-driven property editor，无Reset/unknown/default状态 |
| DATAED-P1-18 | Open | schema edit与row edit无分离的impact/migration/cross-document transaction |
| DATAED-P1-19 | Open | 无base revision、external edit、多实例CAS/merge/save-copy |
| DATAED-P1-20 | Open | 无stable row/field ID驱动的diff、review与source-control conflict UX |
| DATAED-P1-21 | Open | 无CSV/TSV/spreadsheet import/export、encoding/delimiter/schema mapping |
| DATAED-P1-22 | Partial | importer descriptor有priority/version和selection tests；builtin与plugin仍重复拥有TOML/JSON且安装可改语义 |
| DATAED-P1-23 | Open | YAML alias/tag/merge/duplicate-key支持与预算未声明 |
| DATAED-P1-24 | Partial | XML保留element namespace并单遍扫描；mixed-content顺序、attribute namespace、comment/PI仍丢失 |
| DATAED-P1-25 | Open | TOML/JSON datetime、integer/float precision、duplicate/order策略压入JSON最小公分母 |
| DATAED-P1-26 | Partial | builtin失败路径少一次String复制；plugin和成功路径仍持bytes/String/DOM/value/artifact多份，无peak accounting |
| DATAED-P1-27 | Open | 无Data validation rule registry、span、fix、cost与incremental execution |
| DATAED-P1-28 | Open | Data `direct_references`为空；row/asset/localization/tag依赖不进入runtime graph |
| DATAED-P1-29 | Partial | 通用artifact有hash/kind/revision/chunk/size；缺DataTable schema/importer/compiler/dependency/layout header |
| DATAED-P1-30 | Open | 无row/column/key index/chunk/locale strip等runtime layout与lookup基线 |

## 7. P1：SaveGame、Platform、Cloud 与产品资格

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| DATAED-P1-31 | Open | 无Save envelope magic/game/build/user/profile/slot/generation/catalog identity |
| DATAED-P1-32 | Open | 无stable participant/type ID/version/fingerprint与owner lease |
| DATAED-P1-33 | Open | 无显式capture scope/phase/dependency/required/privacy/budget policy |
| DATAED-P1-34 | Partial | DynamicScene有isolated preflight与generation基础；Session restore仍非staged participant transaction/rollback |
| DATAED-P1-35 | Partial | generic serialization与DynamicScene有局部migration；Save envelope/per-type graph、dry run和历史fixture缺失 |
| DATAED-P1-36 | Open | unknown optional/required participant无opaque preserve与fail policy |
| DATAED-P1-37 | Open | build/DLC/mod/plugin compatibility catalog与load plan不存在 |
| DATAED-P1-38 | Open | script save schema、field ID、migration和VM安全边界不存在 |
| DATAED-P1-39 | Open | persistent entity/asset/soft/external reference重绑定规则未定义 |
| DATAED-P1-40 | Open | simulation tick、RNG、timer、clock与pending command保存顺序未定义 |
| DATAED-P1-41 | Open | 无PlatformUser/Profile/controller mapping与登录切换生命周期 |
| DATAED-P1-42 | Partial | Session slot ID、metadata和path已有局部分层；SaveGame opaque slot/user/backend path合同仍不存在 |
| DATAED-P1-43 | Partial | Session有bounded writer submission；Save capture/encode/write/cloud/load无统一cancel/progress/deadline/terminal receipt |
| DATAED-P1-44 | Open | 无autosave scheduler、coalescing、checkpoint policy与lifecycle deadline |
| DATAED-P1-45 | Partial | preferences有capacity/permission错误；SaveGame无slot/count/bytes/temp/free-space preflight |
| DATAED-P1-46 | Partial | core atomic transaction有journal/recovery在途基础；Session路径仍弱，SaveGame双generation/last-good未建立 |
| DATAED-P1-47 | Open | compression只是UI固定LZ4；无algorithm/version/chunk/uncompressed-size/bomb policy |
| DATAED-P1-48 | Open | 无encryption/authentication/tamper/privacy与secure key lifecycle |
| DATAED-P1-49 | Open | 无cloud provider、etag/base generation、offline journal、retry/idempotency |
| DATAED-P1-50 | Open | 无保留local/remote双方的conflict UX与schema-aware policy |
| DATAED-P1-51 | Open | opaque Data、DataTable与SaveGame共用Data外观，capability和权限边界不清 |
| DATAED-P1-52 | Open | static Workbench与未来toolkit/service存在双入口；必须投影同一owner |
| DATAED-P1-53 | Open | route/feedback无request/document/storage generation与provenance |
| DATAED-P1-54 | Partial | Session writer有task/retained-byte admission；Data import/cook和Save capture仍无统一资源准入 |
| DATAED-P1-55 | Open | 无10K/100K/1M row open/filter/edit/validate/save/cook/lookup预算 |
| DATAED-P1-56 | Partial | Session artifact 512 MiB cap和writer limits是真实底座；无帧分片、peak、compression/temp-disk资格 |
| DATAED-P1-57 | Partial | asset/archive/preferences已有typed error片段；无统一schema/row/slot/participant/stage diagnostic与fix |
| DATAED-P1-58 | Open | 现有tests证明parser/artifact/archive/route结构，不证明两条产品闭环 |
| DATAED-P1-59 | Open | 无released DataTable/Save format golden corpus和N-2/N-1兼容矩阵 |
| DATAED-P1-60 | Open | maturity/Workbench承诺不由required product/platform/scale gate生成 |

## 8. P2：完整性、诊断与维护性

| ID | 状态 | 差距 / 重构要求 |
|---|---|---|
| DATAED-P2-01 | Open | opaque Data、DataTable、runtime config与Save payload术语混用 |
| DATAED-P2-02 | Open | schema/row/field/slot ID仍以裸字符串传播 |
| DATAED-P2-03 | Partial | plugin manifest列出capability/priority/version且由声明生成；仍缺builtin/plugin统一authority矩阵和compat check |
| DATAED-P2-04 | Open | runtime artifact默认携带完整source text，缺source/debug/shipping retention policy |
| DATAED-P2-05 | Open | validation/participant diagnostics无bounded query/pagination/virtualization |
| DATAED-P2-06 | Open | slot thumbnail无独立job/generation/failure降级合同 |
| DATAED-P2-07 | Open | schema/slot path/display的Unicode/case/reserved policy未集中 |
| DATAED-P2-08 | Open | wall timestamp不能代替generation/etag/device identity |
| DATAED-P2-09 | Open | telemetry无row/player/path/token privacy和redaction边界 |
| DATAED-P2-10 | Open | 568文件Session facade组合爆炸，SaveGame不得再复制一层 |
| DATAED-P2-11 | Open | fingerprint、SaveGame symbol、Data toolkit、fixture没有自动stale gate |
| DATAED-P2-12 | Open | serialization/resource storage/scene snapshot仍易被误写成DataTable或SaveGame产品 |

## 9. 参考引擎差异与采用路由

| 参考 | 当前源码证据 | Zircon采用边界 |
|---|---|---|
| Unreal DataTable | Factory绑定`RowStruct`；`UDataTable`持RowMap并提供typed `FindRow<T>`；Editor支持transaction、add/remove/rename、row/spreadsheet copy-paste和reimport | DataTable schema/identity/runtime lookup与完整toolkit的主参考 |
| Unreal SaveGame | `ISaveGameSystem`提供platform user、exists/list/save/load/delete与async回调；async action把slot/user/success交回游戏线程 | Save service/user/slot/async/version header主参考，并在cancel/cloud/durability上提高门槛 |
| Godot | ResourceSaver、`user://` FileAccess、ConfigFile/JSON提供resource/user storage与encrypted file组合基础 | 证明底层storage/serialization不自动等于SaveGame，不降低participant/migration门槛 |
| Fyrox | Visitor是versioned tree intermediate，Reader/Writer与Region适合长期对象图迁移 | 采用versioned visitor/region思路，不把scene serialization当玩家存档 |
| Bevy | Scene/ResolvedScene/Reflect serde依赖注册类型和明确apply/spawn | 作为world participant投影基础，不替代保存参与策略和slot service |
| Unity Graphics | 本地只有SerializableEnum/SerializedDictionary等包内容器 | 只作serialized container旁证；没有完整DataTable/SaveGame源码，不推测闭源产品 |

## 10. 目标架构与里程碑

目标链分成两条，禁止混为一个`Data`页面：

`DataTableSource -> lossless/schema document -> validated transaction -> import/compiler receipt -> immutable cooked table generation -> typed runtime handle`

`SaveRequest -> participant capture transaction -> versioned protected envelope -> PlatformSaveStorage atomic generation -> cloud conditional sync -> staged restore transaction -> terminal receipt`

| Milestone | 交付物 |
|---|---|
| M0 | Workbench fail-close、Data input budgets、historical corpus、current fingerprint gate |
| M1 | stable schema/row/field/reference IDs、typed values、migration与Data artifact header |
| M2 | DataTable factory/toolkit/document/undo/clipboard/virtualization/CAS save |
| M3 | bounded CSV/TSV/TOML/JSON/YAML/XML import、incremental validation、dependency/cook receipt |
| M4 | immutable typed runtime lookup、index/hot reload与首个Gameplay consumer |
| M5 | SaveGame service/envelope/participant/schema/migration/catalog |
| M6 | platform user/profile/slot、async storage、quota、atomic generation、transactional load |
| M7 | autosave/lifecycle/protection/cloud etag/offline/conflict/privacy |
| M8 | real Editor toolkit/diagnostics/slot inspector，删除固定feedback |
| M9 | fault/fuzz/history/platform/scale qualification与authority硬收敛 |

## 11. 验收门禁

| Gate | 状态 | 当前证据与通过条件 |
|---|---|---|
| G01 DataTable lifecycle | Fail | 默认产品无factory/toolkit/document |
| G02 Opaque vs table | Fail | 只有coarse Data kind |
| G03 Stable rename | Fail | schema/row/field ID与migration缺失 |
| G04 Value semantics | Fail | JSON无法保持missing/default/null/unknown合同 |
| G05 Table transactions | Fail | 无row/cell/schema command |
| G06 External conflict | Fail | 无DataTable source revision owner |
| G07 Format matrix | Fail | 无schema-bound support/reject/span矩阵 |
| G08 Input bombs | Fail | source/depth/node/alias budget缺失 |
| G09 Import peak/cancel | Fail | 完整read与多份常驻，无admission/cancel |
| G10 Importer selection | Partial | priority/version/selection测试存在；plugin安装仍可改变TOML/JSON语义 |
| G11 Validation | Fail | 无Data rule registry与incremental/full job |
| G12 Reference manifest | Fail | Data direct references为空 |
| G13 Cooked table | Partial | 通用artifact header真实；DataTable exact schema/dependency/generation缺失 |
| G14 Typed lookup | Fail | 只有generic DataAsset load |
| G15 Table scale | Fail | 无目标规模动态证据 |
| G16 Save service | Fail | service/product symbols缺席 |
| G17 Save envelope | Fail | 无Save magic/identity/catalog/protection header |
| G18 Participants | Fail | registry/phase/policy缺失 |
| G19 World completeness | Fail | 未证明任意plugin/typed component无损capture/restore |
| G20 Migration | Fail | 无Save historical graph/fixture |
| G21 Unknown participant | Fail | required/optional/opaque policy缺失 |
| G22 Transactional load | Fail | Session restore不是participant staging/rollback |
| G23 Reference rebind | Fail | persistent vs process handle规则缺失 |
| G24 Autosave frame budget | Fail | scheduler/coalesce/frame evidence缺失 |
| G25 Storage fault | Partial | core atomic/journal/recovery基础存在；Save slot last-good全阶段故障证据缺失 |
| G26 Quota | Fail | 无Save capture前后空间/平台limit合同 |
| G27 Compression | Fail | UI固定LZ4，无envelope/bomb policy |
| G28 Protection | Fail | secure provider/key lifecycle缺失 |
| G29 Cloud conflict | Fail | etag/offline/multi-device authority缺失 |
| G30 Lifecycle terminal | Fail | suspend/sign-out/quit请求状态机缺失 |
| G31 Required evidence | Fail | 无两条产品E2E/history/fuzz/fault/scale lane |
| G32 Workbench truth | Fail | 55 nodes/38 routes仍固定业务事实与success fallback |

## 12. 禁止的临时修补

- 不得给`ResourceKind::Data`挂JSON文本编辑器后宣称DataTable完成。
- 不得把schema、row key、field type藏在约定JSON key而无stable identity。
- 不得用`split(',')`实现CSV，也不得通过提高artifact cap替代读取前预算。
- 不得把有损XML-to-JSON称为canonical可逆表示。
- 不得把Save Data route接到Session Archive并重命名SaveGame。
- 不得默认序列化整个World、VM heap或plugin state；participant必须显式注册并版本化。
- 不得用Rust type name、`TypeId`、module path或临时entity handle作持久identity。
- 不得在UI/游戏线程同步capture world、压缩、写盘或云上传。
- 不得先删旧slot再写新slot；必须新generation验证后切current。
- 不得用checksum冒充加密/防篡改，不得以wall-clock newest静默解决cloud冲突。
- 不得在storage terminal receipt前显示Save/Load/Cloud成功，也不得保留fixture fallback。
- 不得以ignored microbenchmark或静态route断言替代产品闭环。

## 13. 本轮产出边界

本文只做current-source静态review与重构计划，不修改production或tests，不把任何finding标记为implemented。Editor24仍是canonical finding owner，Editor98只刷新currentness。实施必须先关闭M0输入/真实性门和M1身份合同，不能先给两份Workbench增加按钮。Runtime40/52/61/63/85/86继续持有runtime owner；Tooling按要求排除，且本轮没有查询或实时跟踪协调器。
