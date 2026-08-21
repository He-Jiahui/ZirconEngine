---
title: Runtime Asset Type、Schema、Imported Payload、Project Document、Validation、Dependency、Serialization、Versioning 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime86
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/asset/assets
  - zircon_runtime/src/asset/facade
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/importer/ingest
  - zircon_runtime/src/asset/registry/dependency_extractors
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/ui/template
  - zircon_runtime_interface/src/ui/v2
tests:
  - zircon_runtime/src/asset/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/68-runtime-sprite2d-canvas2d-sprite-atlas-tileset-tilemap-batching-sorting-lighting-physics-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/69-runtime-mesh-static-mesh-skeletal-mesh-submesh-lod-instancing-skinning-morph-collision-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/AssetRegistry/AssetIdentifier.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/PackageDependencyData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Serialization/CustomVersion.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PackageFileSummary.h
  - dev/bevy/crates/bevy_asset/src/id.rs
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/reflect.rs
  - dev/Fyrox/fyrox-resource/src/lib.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/godot/core/io/resource_format_binary.cpp
  - dev/godot/core/io/resource_importer.cpp
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_uid.cpp
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Util/SerializationHelper.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Tools/MaterialUpgrader/MaterialUpgrader.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/AssetDatabaseHelper.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Asset Type、Schema、Imported Payload、Project Document、Validation、Dependency、Serialization、Versioning 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前资产数据层已经不是“所有东西都装进一个JSON对象”的空壳。它有31种`ImportedAsset`载荷、27种`ResourceKind`、typed `Asset/Handle/Assets` facade、UUID+locator引用、project GUID+path hint+subasset持久引用、Material/Model/Scene正式project document投影、若干格式级版本和validator，以及带magic、schema、BLAKE3、zstd、chunk边界和解码上限的v5 artifact store。这些是应保留的真实底座。

但系统还没有一份资产类型描述符同时定义stable type identity、schema identity/version、codec、validator、dependency extractor、migration chain、artifact projection和runtime marker。当前31种载荷被手写分派到多个互相独立的`match`；`ResourceKind`又把UiIcon/Texture、UiTheme/UiStyle、UiV2/legacy UI压成同一宽泛kind。结果是Handle、contains、event、insert、artifact manifest、native importer和Editor catalog看到的“类型事实”不同，加载器只能按固定顺序试探downcast。

本轮新增1项P0：Prefab、MaterialGraph、Terrain、TerrainLayerStack、TileSet、TileMap、UiV2 View/Component/Style和UiIcon已经能从载荷计算真实直接引用，但fresh/targeted/restore写入运行时依赖图的handwritten extractor仍只覆盖Scene、Material、Model；这些类型的import outcome又不填dependencies。Editor catalog可从`ImportedAsset::direct_references()`显示引用，Runtime `.zmeta`/`ResourceRecord`却没有同一条边，导致ready、reload、affected referencer和package closure静默漏依赖。Runtime51的P1-056至P1-058继续拥有通用extractor registry缺口；Runtime86登记的是该缺口在已激活资产上的确定性产品正确性升级。

除该P0外，本报告登记48项P1、12项P2和48项资格门。目标不是继续扩展宽泛枚举或补更多试探分支，而是建立唯一`AssetTypeCatalog`、versioned `AssetEnvelope`、typed `AssetDependencyGraph`、统一`ProjectDocumentCodec`和per-type artifact codec，使Runtime、Editor、importer、artifact、plugin和package消费同一份可验证资产事实。

## 2. 审查边界、证据等级与冻结快照

### 2.1 证据等级

| 等级 | 本轮使用方式 | 能证明什么 |
|---|---|---|
| E3 | 逐文件读取`asset/assets` 101个文件、facade、artifact payload/store、ingest、dependency extractor、project publication、typed load、Interface project/resource/UI schema，并沿import -> meta -> registry -> load -> catalog调用链追踪 | 当前类型、版本、序列化、验证、依赖与消费者事实 |
| E2 | 检索全部asset tests、31种variant的分派/marker/downcast、schema/version/unknown-field/reference调用点及父报告 | 静态覆盖矩阵、重复authority和owner边界 |
| E1 | 读取167个asset test文件，但本轮不运行 | 测试意图、局部正向覆盖和缺失的负向/跨版本矩阵 |
| E0 | 未运行Cargo、Editor、真实项目迁移、artifact skew、plugin skew、fuzz、fault、soak或benchmark | 不得宣称动态通过、兼容窗口成立或性能优于Unreal |

### 2.2 Zircon冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes | fingerprint |
|---|---:|---|
| Production、contract与consumer去重集合 | **233 / 34,524 / 31,672 / 1,195,124** | `3a4aae807406ceb1a204c65b918107fb8995a58afc9b964754bec10920257a2e` |
| Runtime asset tests | **167 / 35,003 / 32,094 / 1,215,488**，628项`#[test]`、1项ignored | `17ddde25feaf268f782e5b7b73de1041e1ced3811e172b1135a189297feabdb2` |
| 参考引擎去重集合 | **35 / 21,456 / 777,029 bytes** | `2ecdc384d8f51897066db2a118bff4f92f5b8711d627290e6645b777c9b3ee4a` |

fingerprint方法是按规范化相对路径排序，依次串联相对路径、换行、原始文件bytes和分隔换行后计算SHA-256。冻结时production集合有17个dirty/untracked路径，tests集合有7个dirty路径；其中包括`artifact/store.rs`、`project_document/material.rs`及glTF/font/texture/model/mesh/shader importer。报告审查的是当前工作树，不把并行Session中的代码存在写成已通过managed validation或accepted integration。

### 2.3 本轮不重复拥有的边界

| 父owner | 唯一拥有内容 | Runtime86边界 |
|---|---|---|
| Runtime04 | 通用resource/asset serialization、exact type缺失、source schema migration | 只落实每类资产descriptor/codec/validator的具体收敛 |
| Runtime51 | registry row、exact type/schema、dependency extractor registry与coverage manifest | 通用P1不重复计数；已激活类型漏边升级为本轮P0 |
| Runtime61 | Scene无损project I/O、clone/snapshot/schema transaction | Scene root `_rest`/version缺失作为依赖引用，不另登记Scene数据丢失P0 |
| Runtime64 | wrong payload admission、load/reload/lease/cache authority | Runtime86只定义exact type/schema如何进入authority |
| Runtime68/69 | SpriteAtlas/TileMap/Mesh运行时语义与格式产品闭环 | 不把未注册SpriteAtlas、Mesh/Scene格式缺口重复计数 |
| Runtime73/74 | UI style/template/binding/hot reload | Runtime86只拥有UI asset schema与依赖投影一致性 |
| Runtime85 | source/import/build/DDC/cook/package与artifact producer provenance | 本轮只拥有payload schema/codec/version及其dependency truth |
| Editor04/24/32/34/35 | catalog/reference UX与Data/Model/Mesh/2D/Texture authoring | Editor必须消费Runtime唯一图，但专项功能仍由原报告拥有 |

## 3. 当前实现中可保留的工程底座

### 3.1 类型化facade与运行时payload检查

`Asset`把具体数据和`ResourceMarker`绑定，`Handle<T>`在Rust类型层提供静态区分；`ResourceManager::get/acquire`最终仍按`TypeId` downcast，readiness projection也记录payload `TypeId`。因此底层并非完全无类型，只是exact type没有被提升为持久、可查询、可迁移的资产合同。

### 3.2 稳定引用与正式project reference

`AssetReference`同时携带UUID和locator；`PersistedAssetReference`明确区分project与builtin，project `AssetRef`含GUID、movable path hint和可选subasset。Material、Model、Scene正式document wrapper能在保存/加载时映射引用，并在若干层保留未知TOML字段。这条路线正确，应扩展为所有authoring asset共享的codec，而不是退回只存路径。

### 3.3 局部版本、验证与迁移基础

ZMaterial v2、ZShader v2、ZMesh v1、Mesh SDF schema、UI legacy/v2、animation/NavMesh等已有版本边界；Terrain、TileMap、MaterialGraph、Mesh、UiIcon等也有局部validator。`AssetSchemaMigrationReport`和reference repair提供了迁移回执的雏形。问题是这些能力未被统一descriptor强制调用。

### 3.4 Artifact完整性与资源上限

v5 artifact manifest有magic/schema、kind、revision、content hash、raw/compressed size和chunk inventory；读取侧限制manifest/raw payload，固定整数bincode解码，检查trailing bytes、chunk hash和总size。它比直接把任意serde对象写盘可靠得多，未来应保留chunk/content完整性并替换语义不足的全局payload版本。

### 3.5 参考引擎给出的最低工程线

- Unreal的`FAssetIdentifier`可表达package/object/value，Asset Registry依赖有category/property query；`FCustomVersion`用GUID+version+validator注册并比较Missing/Newer/Older/Invalid。
- Bevy的typed `AssetId<A>`把类型进入hash，`UntypedAssetId`显式保存`TypeId`；meta包含格式版本、loader/processor type path、settings、processed hash和process dependency full hash。
- Fyrox资源在typed request/add/try-request时校验Reflect type UUID，不把扩展名或宽泛kind当作最终类型证明。
- Godot二进制resource format独立版本化，external dependency同时记录UID、fallback path和type，并提供get/rename dependencies与cache mode。
- Unity Graphics本地corpus用type serialization info、显式type remap、unknown node保留思路和Material upgrader；它只作包内旁证，不代表完整Unity AssetDatabase实现已在仓内。

## 4. 新增P0正确性阻断

### `ATYPE86-P0-001`：已激活资产的真实引用未进入Runtime依赖真源

**确定性证据链：**

1. `ImportedAsset::direct_references()`明确覆盖MaterialGraph、Terrain、TerrainLayerStack、TileSet、TileMap、Prefab、UiV2 View/Component/Style和UiIcon；对应asset tests也验证这些对象能产出引用。
2. `import_authoring_asset.rs`、`import_ui_v2_asset.rs`和`import_ui_icon_asset.rs`均通过`AssetImportOutcome::new`返回，未填entry dependencies。
3. fresh和targeted import在publication前调用`append_handwritten_dependencies()`，但它只match Scene、Material、Model，其他variant静默返回空数组。
4. restore路径调用`merge_handwritten_dependencies_into_meta()`，底层仍是同一三variant extractor。
5. `.zmeta` entry dependencies随后才被解析为`ResourceRecord.dependency_ids`；ready、reload closure、affected referencer与pack/export closure依赖该图。
6. Editor catalog input却直接从payload读取`ImportedAsset::direct_references()`，因此UI可显示一条Runtime dependency graph不存在的引用。

**受影响类型：** Prefab、MaterialGraph、Terrain、TerrainLayerStack、TileSet、TileMap、UiV2 View、UiV2 Component、UiV2 Style、UiIcon。AnimationClip/Graph/StateMachine也由`direct_references`表达引用，必须进入同一coverage manifest；Shader当前由importer单独填依赖，仍须证明与descriptor extractor等价。

**产品影响：** 被引用资源未就绪或失败时，父资产可被误报ready；依赖变化不会可靠触发父资产reload；删除/rename后的affected referencer不完整；cook/package closure可能漏装资源。Editor详情中的“引用存在”不能证明Runtime会等待、重载或打包它。

**必须修复：** 先以覆盖全部31种payload的单一`AssetTypeDescriptor::extract_dependencies`替代三variant手写入口；每种类型必须显式返回`KnownEmpty`或typed edge集合，禁止默认空。fresh、targeted、restore、catalog、package只能消费该结果；publication前比较payload extraction、importer-declared source dependency与persisted meta，差异必须终止提交或产生明确迁移事务。

**回归门：** 上述每个受影响类型至少有一项fresh、targeted、restore、rename、delete、reload、package closure测试；同一fixture的`.zmeta`、`ResourceRecord`、Editor graph和pack closure边集合必须完全一致，subasset label不得丢失。

## 5. P1工程化差距（48项）

### 5.1 Exact type、facade与分派（P1-001至P1-008）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ATYPE86-P1-001 | `ResourceKind`把UiIcon与Texture、UiTheme/UiV2Style与UiStyle、UiV2View与UiLayout等不同payload压成同一kind | stable `AssetTypeId`独立于coarse capability kind |
| ATYPE86-P1-002 | `load_imported_asset`按固定顺序尝试UiIcon/Texture、V2/legacy UI downcast，正确结果依赖探测顺序 | record/artifact直接声明exact type并单次选择codec |
| ATYPE86-P1-003 | `Handle<T>`只序列化`ResourceId`，反序列化不能验证原始type/schema | typed persistent handle envelope或显式untyped handle+resolve receipt |
| ATYPE86-P1-004 | `Assets<T>::contains`只检查coarse kind，可对错误exact payload返回true | contains/readiness统一校验type id、schema与generation |
| ATYPE86-P1-005 | `Assets<T>::insert`只检查record kind，允许共享kind的错误具体类型进入slot | descriptor-driven admission，record和payload exact identity原子匹配 |
| ATYPE86-P1-006 | typed asset event只按coarse kind过滤，UiIcon订阅和Texture订阅可收到彼此事件 | event携带exact type/schema/generation并按descriptor过滤 |
| ATYPE86-P1-007 | `ImportedAsset`是31variant closed enum，第三方plugin不能定义一等新资产类型 | provider-owned type registration、opaque envelope和安全unload generation |
| ATYPE86-P1-008 | kind mapping、resource conversion、cache projection、typed loading分别维护大段手写match | descriptor/codegen生成唯一穷举表并有compile-time coverage |

### 5.2 Schema、version、validation与migration（P1-009至P1-016）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ATYPE86-P1-009 | 没有中央`AssetSchemaId`、current/min readable/min writable版本和codec identity | per-type schema descriptor与support window |
| ATYPE86-P1-010 | 版本只存在于ZMaterial/ZShader/ZMesh/MeshSdf/UI/部分animation/NavMesh，Terrain、Tile、Prefab、Graph、Scene、Model、Data等没有一致边界 | 每个持久DTO必须声明版本或明确`ephemeral-only` |
| ATYPE86-P1-011 | validator仅覆盖部分类型，Deserialize或artifact decode后不保证统一执行 | parse/migrate/validate/cook/load各阶段的强制validation pipeline |
| ATYPE86-P1-012 | unknown field policy在`deny_unknown_fields`、`flatten _rest`和默认忽略之间漂移 | descriptor声明Reject/Preserve/Opaque三态及round-trip保证 |
| ATYPE86-P1-013 | validation error多被压成`AssetImportError::Parse(String)` | stable code、field path、span、severity、suggestion和source provenance |
| ATYPE86-P1-014 | migration report可选且没有migration id/chain digest、field receipts或downgrade policy | deterministic migration chain和可审核receipt |
| ATYPE86-P1-015 | schema字段没有stable field id、reference metadata、default provenance或deprecation window | reflection-ready field descriptor与migration alias |
| ATYPE86-P1-016 | 同一类型的authoring DTO、ImportedAsset、artifact cache DTO和runtime payload兼容关系未声明 | descriptor内显式Authoring -> Canonical -> Cooked -> Runtime投影 |

### 5.3 Project document与reference codec（P1-017至P1-024）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ATYPE86-P1-017 | 只有Material、Model、Scene走`PersistedAssetReference`正式codec | 所有project authoring asset统一GUID/path hint/subasset引用 |
| ATYPE86-P1-018 | Terrain/LayerStack/TileSet/TileMap/Prefab/MaterialGraph直接serde `AssetReference {uuid,url}` | 保存时经resolver生成project/builtin reference，加载时产生resolution receipt |
| ATYPE86-P1-019 | Scene root document没有`_rest`和version，未知root字段可在round-trip丢失 | root-level version、unknown preservation和lossless fixture；数据P0仍归Runtime61 |
| ATYPE86-P1-020 | Model wrapper保留unknown字段但没有document version或migration | versioned model authoring schema与explicit upgrader |
| ATYPE86-P1-021 | `ProjectDocumentArtifact`先解析generic `toml::Value`再转换typed DTO，重写会丢注释、顺序和source spans | lossless syntax tree或明确canonical rewrite receipt |
| ATYPE86-P1-022 | project document error主要包装TOML/字符串，无法定位schema/migration/reference阶段 | typed phase、schema id/version、field path和span chain |
| ATYPE86-P1-023 | `DataAsset`同时保存raw text与canonical JSON，没有不可变关系或版本证明 | 单一canonical authority，raw source只作provenance/blob |
| ATYPE86-P1-024 | UI reference仍是字符串而非正式project reference，移动、GUID冲突和subasset repair合同分叉 | UI schema使用typed resource reference并共享resolver/migration |

### 5.4 Dependency schema与唯一真源（P1-025至P1-032）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ATYPE86-P1-025 | importer entry dependencies与payload `direct_references`是两套可漂移authority | descriptor extraction产生canonical logical edge，importer只补source/build edge |
| ATYPE86-P1-026 | dependency只保存URI，未表达expected type/schema、hard/soft、editor/runtime、load/build语义 | typed `AssetDependencyEdge`及category/property/strength/provenance |
| ATYPE86-P1-027 | UI `push_reference`重建locator时强制label为None，subasset引用退化为root | 保留并验证subasset identity，解析失败不得静默改绑 |
| ATYPE86-P1-028 | UI引用解析失败直接return，合法性错误不会进入import diagnostic | invalid reference是field-scoped terminal或policy-controlled diagnostic |
| ATYPE86-P1-029 | legacy UI递归扫描任意TOML字符串，只凭`res://`等前缀猜依赖 | schema标注resource-valued field，禁止内容启发式扫描 |
| ATYPE86-P1-030 | UI v2虽有显式imports，但不声明资源expected type、fallback兼容和load policy | typed import slot与fallback compatibility validation |
| ATYPE86-P1-031 | fresh/targeted/restore分别拼装meta与依赖，等价性依赖人工维护 | 单一dependency projection函数和generation receipt |
| ATYPE86-P1-032 | Editor重建第二份ReferenceGraph，locator fallback还可掩盖GUID冲突 | Runtime发布唯一versioned graph snapshot，Editor只投影视图；UX归Editor04 |

### 5.5 Artifact、native importer与plugin schema（P1-033至P1-040）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ATYPE86-P1-033 | v5 manifest只有全局schema和coarse kind，没有exact type/schema/codec version | manifest记录type/schema/codec/producer compatibility domain |
| ATYPE86-P1-034 | `ArtifactCacheAsset`用bincode enum ordinal与字段顺序，任一variant演进共享全局破坏半径 | tagged section directory、stable type id和per-type codec version |
| ATYPE86-P1-035 | 多种cache variant直接嵌入authoring/runtime struct，字段演进会隐式改变wire | 专用artifact DTO与显式upgrade/down-convert |
| ATYPE86-P1-036 | artifact读写只校验coarse kind，不能证明UiIcon/Texture等exact payload一致 | decode前后校验exact identity与schema receipt |
| ATYPE86-P1-037 | 全局manifest version升级会迫使无关资产一起失效或继续误读 | container version与payload schema独立演进 |
| ATYPE86-P1-038 | native importer JSON envelope直接序列化closed `ImportedAsset`，provider无法协商自定义schema | bounded typed envelope、type negotiation与opaque payload channel |
| ATYPE86-P1-039 | native envelope metadata length只与payload比较，没有独立metadata/entry/diagnostic/dependency预算 | protocol级byte/count/depth/time budget |
| ATYPE86-P1-040 | response artifact bytes仍标为reserved并拒绝，plugin无法直接返回qualified artifact | negotiated streamed artifact section与host-side verification |

### 5.6 Product接线、可观测性与测试架构（P1-041至P1-048）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ATYPE86-P1-041 | Runtime registry、artifact、typed facade和Editor catalog暴露的type/ref事实不同 | 单一`AssetRecordSnapshot`供所有产品消费 |
| ATYPE86-P1-042 | loader probe失败只表现为某次downcast error，缺expected/actual type/schema和probe原因 | typed resolution diagnostic与禁止多probe的结构门 |
| ATYPE86-P1-043 | schema too new/too old、migration required、validation failed、provider missing未形成统一load state | explicit compatibility state machine与last-known-good policy |
| ATYPE86-P1-044 | 没有按type/schema/provider统计migration、validation、unknown field、dependency mismatch和artifact rejection | bounded metrics、journal和project health projection |
| ATYPE86-P1-045 | 无31种ImportedAsset x kind x marker x codec x validator x dependency extractor x typed loader覆盖矩阵 | generated conformance manifest和必跑矩阵 |
| ATYPE86-P1-046 | dependency extractor测试只覆盖Scene/Material/Model，其他测试仅证明对象能返回reference | publication-level全variant正/空/错误/restore一致性测试 |
| ATYPE86-P1-047 | 缺跨版本golden corpus、unknown field preservation、upgrade/downgrade和artifact skew测试 | versioned fixture repository与support-window lanes |
| ATYPE86-P1-048 | 缺serde/bincode/TOML/reference/dependency/native envelope fuzz及plugin unload/schema skew测试 | bounded fuzz/property/fault matrix和quarantine receipt |

## 6. P2质量与维护性差距（12项）

| ID | 差距 | 建议 |
|---|---|---|
| ATYPE86-P2-001 | `Asset::LABEL`是display string，不是可本地化或稳定identity | descriptor分离stable id与display metadata |
| ATYPE86-P2-002 | 多处错误文本手工拼接kind/type名，措辞和上下文不一致 | typed diagnostic formatter |
| ATYPE86-P2-003 | `ImportedAsset`、cache enum和load match的长列表易制造review噪声 | descriptor declaration+生成代码 |
| ATYPE86-P2-004 | 版本常量命名、类型和位置不统一 | schema module命名规范 |
| ATYPE86-P2-005 | 部分payload同时保存source URI与record locator，缺一致性断言 | canonical identity projection |
| ATYPE86-P2-006 | runtime payload中保留可重新生成的authoring text/diagnostic字段 | 分离source provenance、debug sidecar与shipping payload |
| ATYPE86-P2-007 | `Vec<String>` diagnostics缺稳定排序、去重和结构化字段 | diagnostic set schema |
| ATYPE86-P2-008 | validator方法命名为`validate_*`但返回不同错误层级 | 统一validation trait/outcome |
| ATYPE86-P2-009 | project wrapper映射函数重复手写每层字段 | schema-driven reference visitor |
| ATYPE86-P2-010 | UI legacy/v2/Theme/Icon marker alias缺一处可查说明 | descriptor snapshot与自动文档 |
| ATYPE86-P2-011 | asset tests大量直接构造struct，容易绕过真实parse/migrate/validate入口 | fixture builder默认走产品入口 |
| ATYPE86-P2-012 | 当前API名把authoring、canonical、cache、runtime都称为`Asset` | 明确阶段后缀与module ownership |

## 7. 目标架构

### 7.1 `AssetTypeCatalog`

每个资产类型必须注册不可变descriptor：`AssetTypeId`、provider/generation、coarse capability kind、current/min schema version、authoring/runtime/artifact codec、validator、migrator、dependency visitor、reference schema、runtime marker和display metadata。catalog冻结后才允许project import/load；plugin unload必须等待该generation的payload、codec和consumer lease归零。

### 7.2 `AssetEnvelope`与阶段化payload

统一envelope至少包含asset identity、exact type、schema/codec version、provider generation、content hash、dependency digest和payload section table。Authoring、Canonical、Cooked、Runtime是显式阶段；转换返回receipt，禁止把同一个serde struct偶然复用于所有阶段。

### 7.3 `ProjectDocumentCodec`

所有project authoring asset通过同一codec完成parse、version admission、migration、reference resolution、validation、canonical/lossless write和diagnostic。未知字段策略由schema声明；GUID、path hint和subasset必须共享`PersistedAssetReference`，不允许某些类型保存`{uuid,url}`、另一些只保存字符串。

### 7.4 `AssetDependencyGraph`

dependency edge必须包含source/target qualified identity、expected type/schema、category、strength、load/build/editor/runtime scope、subasset和provenance。descriptor extraction是logical dependency唯一真源；source/build dependencies由Runtime85 build graph补充。Runtime registry、Editor catalog、reload和package闭包都消费同一generation snapshot。

### 7.5 `AssetCompatibilityService`

load先完成type/provider/schema admission，再决定Direct、Migrate、ReadOnlyLegacy、TooNew、TooOld、ProviderMissing、Corrupt或Invalid。migration在隔离区执行，验证通过后原子发布新generation；失败保留last-known-good并给出可操作receipt，不能靠另一个downcast分支继续试探。

### 7.6 Artifact与plugin协议

container version只定义section/index完整性；每个payload section独立声明type/schema/codec。native/plugin importer通过有预算的typed binary envelope协商类型和版本，可返回opaque/streamed artifact；host验证descriptor、hash、依赖和provider generation后才接纳。未知类型可被安全保留、检查或隔离，但不能伪装成某个宽泛kind。

## 8. 重构里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 | 修复`ATYPE86-P0-001`，建立31variant dependency coverage manifest | fresh/targeted/restore/catalog/package边集合一致 |
| M1 | 引入stable `AssetTypeId/SchemaId/CodecId`和descriptor catalog | 所有现有variant有唯一descriptor，禁止默认空项 |
| M2 | 将record、handle、event、typed load和artifact manifest升级为exact type-aware | 删除Ui/Texture和legacy/V2 probe load |
| M3 | 统一ProjectDocumentCodec与PersistedAssetReference | 全部authoring asset通过GUID/path/subasset round-trip |
| M4 | 建立per-type validation/migration/support window | golden corpus覆盖min/current/too-new/invalid |
| M5 | 升级artifact为sectioned per-type codec | unrelated type演进不全局失效，旧v5有明确迁移/清理策略 |
| M6 | 建立plugin type/schema协议和generation lease | 自定义类型加载、失败、卸载、skew可验证 |
| M7 | Runtime发布唯一dependency/type snapshot，Editor删除第二authority | catalog/reload/package均消费同一generation |
| M8 | fault/fuzz/规模/性能与跨机资格 | 48项gate全部有可复验receipt |

依赖顺序必须是M0 -> M1 -> M2/M3 -> M4 -> M5/M6 -> M7 -> M8。不能先写新的Editor schema UI、更多import variant或artifact优化来绕过类型与依赖真源。

## 9. 资格门（48项）

### 9.1 类型、注册与生命周期（G01-G08）

| Gate | 必须证明 |
|---|---|
| G01 | 31种当前payload和所有plugin type均有唯一stable type id |
| G02 | coarse kind collision不能导致contains/event/load/insert误判 |
| G03 | typed与untyped handle round-trip保留并验证type/schema |
| G04 | record、payload、artifact和descriptor exact identity不一致时fail closed |
| G05 | 删除所有ordered downcast probe并有结构守卫 |
| G06 | descriptor register/freeze/unload generation可并发验证 |
| G07 | provider unload等待codec/payload/consumer lease而不永久阻塞 |
| G08 | generated coverage对新增类型漏接线产生编译或必跑测试失败 |

### 9.2 Schema、document、validation与migration（G09-G20）

| Gate | 必须证明 |
|---|---|
| G09 | 每个持久类型声明current/min readable/min writable版本 |
| G10 | too-new、too-old、provider-missing和invalid状态互不混淆 |
| G11 | parse后、migration后、artifact decode后和runtime admission前均验证 |
| G12 | unknown field Reject/Preserve/Opaque策略有round-trip fixture |
| G13 | diagnostic含稳定code、schema、field path、span和provenance |
| G14 | migration chain deterministic且receipt可追溯每一步 |
| G15 | migration失败不覆盖source、meta或last-known-good artifact |
| G16 | Material/Model/Scene及其余authoring asset共享同一document pipeline |
| G17 | 所有project reference保留GUID、path hint和subasset |
| G18 | rename/move/path occupied/GUID conflict得到不同typed状态 |
| G19 | canonical rewrite明确报告comment/order/trivia变化，或真正lossless |
| G20 | downgrade/read-only legacy策略和支持窗口有产品提示与测试 |

### 9.3 Dependency唯一真源（G21-G30）

| Gate | 必须证明 |
|---|---|
| G21 | 每个asset type显式返回KnownEmpty或typed dependency edges |
| G22 | Prefab/Graph/Terrain/LayerStack/TileSet/TileMap/UI v2/Icon不再漏边 |
| G23 | Animation和Shader等独立路径进入同一coverage manifest |
| G24 | subasset label在extract、meta、registry、catalog、package全程保持 |
| G25 | invalid UI/reference不能静默丢弃或改绑root |
| G26 | fresh、targeted和restore得到相同edge set与digest |
| G27 | Runtime ready/failure计算消费typed edge语义 |
| G28 | reload affected closure与Editor referencer视图完全一致 |
| G29 | package/cook closure与Runtime graph同代且无漏装 |
| G30 | dependency cycle、soft edge、editor-only和optional策略可验证 |

### 9.4 Artifact、plugin与兼容性（G31-G40）

| Gate | 必须证明 |
|---|---|
| G31 | manifest记录exact type/schema/codec/provider和dependency digest |
| G32 | container version与payload schema可独立演进 |
| G33 | enum reorder/field add/remove不能静默误解旧payload |
| G34 | unrelated asset schema升级不迫使全库无条件重建 |
| G35 | old/current/too-new artifact corpus得到确定compatibility状态 |
| G36 | corrupt/truncated/trailing/oversized/decompression bomb均有界拒绝 |
| G37 | native envelope限制metadata、entry、dependency、diagnostic和payload预算 |
| G38 | plugin自定义类型可导入、缓存、加载、查询并安全卸载 |
| G39 | unknown/provider-missing payload可隔离或保留，不伪装成coarse kind |
| G40 | plugin/engine schema skew、panic、timeout和malformed response有terminal receipt |

### 9.5 产品、回归与性能（G41-G48）

| Gate | 必须证明 |
|---|---|
| G41 | Runtime、Editor、App/headless和package工具消费同一type/dependency snapshot |
| G42 | project health可列出迁移、invalid、unknown、provider missing和edge mismatch |
| G43 | 新增资产类型只改descriptor/provider，不手工修改多处中央match |
| G44 | 31type conformance、golden version、unknown-field和round-trip矩阵进入required lane |
| G45 | serde/bincode/TOML/reference/native/plugin fuzz在固定预算内运行 |
| G46 | 百万asset/deep graph/large document下catalog freeze、migration和query有固定预算 |
| G47 | cold/warm/load/migrate/reload/package性能在正确性门后与参考引擎同方法测量 |
| G48 | 所有资格结果记录baseline、hardware、corpus、hash、failure和source fingerprint |

## 10. 禁止的临时实现

- 禁止再给`ResourceKind`增加一个别名后靠downcast顺序区分payload。
- 禁止以Rust `TypeId`、类型名字符串、enum ordinal或文件扩展名充当跨版本stable asset type id。
- 禁止新增`ImportedAsset` variant时手工复制更多match而没有descriptor coverage gate。
- 禁止dependency extractor的默认分支返回空；无依赖必须是显式`KnownEmpty`。
- 禁止让Editor继续从payload重建第二份权威图后用locator fallback掩盖GUID冲突。
- 禁止剥离subasset label、吞掉非法URI或把解析失败降级为“没有依赖”。
- 禁止只给全局artifact version加一；必须区分container和per-type payload schema。
- 禁止直接把任意authoring struct塞进bincode并把serde字段顺序当wire contract。
- 禁止用“反序列化成功”替代schema admission、migration和validation。
- 禁止只为Material/Model/Scene维护正式reference codec，其他资产继续保存`{uuid,url}`或字符串。
- 禁止用更多source guard测试代替真实fresh/targeted/restore/reload/package行为矩阵。
- 禁止在M0-M7正确性门关闭前声称资产系统达到或超过Unreal。
- 禁止把未来tooling迁移Rust当作Runtime asset type/schema authority已经成立。

## 11. 完成边界

本报告完成的是当前源码静态审查、父owner去重和重构需求登记，不是实现完成。只有M0-M8全部按资格门取得可复验回执，Runtime04/51/61/64/68/69/73/74/85与Editor04/24/32/34/35完成相应依赖，且Runtime、Editor和package真正消费同一type/schema/dependency generation后，`implementation_status`才可改为`complete`。

本轮未修改Rust、Cargo、资源、plugin或工具实现，未运行Cargo、Editor、真实迁移、artifact skew、plugin unload、fuzz、fault、soak或benchmark。用户已要求暂不考虑tooling优化；因此本文只规定Runtime数据合同、产品消费边界和Editor handoff，不评价后续将迁移为Rust的工具实现。
