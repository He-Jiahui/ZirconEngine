---
title: Runtime Asset Reference、Identity、Locator、GUID、Subasset、Redirector、Rename/Move、Resolution、Repair、Migration 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime87
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/reference_resolution_error.rs
  - zircon_runtime/src/asset/project/manager/persisted_reference.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/migration
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/resource
  - zircon_editor/src/core/asset
  - zircon_editor/src/ui/host/editor_asset_manager
tests:
  - zircon_runtime/src/asset/tests
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime_interface/src/project/tests
  - zircon_editor/src/core/asset/dirty/save_batch/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-08-18-missing-subasset-parent-fallback.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/SoftObjectPath.cpp
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetRenameManager.h
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetRenameManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/ObjectRedirector.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/ObjectRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/bevy/crates/bevy_asset/src/id.rs
  - dev/bevy/crates/bevy_asset/src/handle.rs
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/godot/core/io/resource_uid.h
  - dev/godot/core/io/resource_uid.cpp
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_importer.h
  - dev/godot/core/io/resource_importer.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/AssetDatabaseHelper.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/SerializableTexture.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/SerializableCubemap.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/SerializableMesh.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Asset Reference、Identity、Locator、GUID、Subasset、Redirector、Rename/Move、Resolution、Repair、Migration 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前资产引用链已经有一组值得保留的真实底座：Runtime引用携带UUID与locator，project引用携带GUID、movable path hint和可选subasset；registry同时按UUID、path、source和reverse dependency查询；full/targeted generation使用候选状态与持久事务；移动source与`.zmeta` sidecar时，projected inventory能保持GUID；缺失subasset label也已在当前源码中拒绝为`DanglingSubasset`，不再静默回退父资产。

但“GUID是稳定身份、path只是提示”这一核心语义尚未成立。当前resolver在GUID不存在时会直接采用path hint命中的条目并生成GUID repair；更严重的是，GUID存在但subasset label不匹配时，也会以path+label命中的另一个条目替换GUID。`Conflict`错误虽然已声明，生产resolver没有任何构造分支。删除旧资产后在同一路径创建另一资产，或subasset标签被另一个对象占用，都会把稳定引用静默改绑到新对象。该行为还被migration复用并可在apply模式自动写回，属于确定性数据完整性P0。

第二项P0来自产品接线。Material/Model/Scene importer会把resolver产生的`ReferenceRepair`放进`AssetImportOutcome.reference_repairs`，合同注释明确允许调用方呈现或持久化；但full generation、targeted generation、pipeline和Editor均没有任何读取点。native importer host adapter还固定返回空repair列表。结果是本次运行可使用已改绑的payload，authoring source却继续保存旧引用，没有审批、事务、回执或重启一致性。

本报告新增2项P0、48项P1、12项P2和48项资格门。目标不是继续增加fallback，而是建立`QualifiedAssetReference + AssetResolutionSnapshot + AssetReferenceResolver + ReferenceRepairPlanner + AssetMutationTransaction + ReferenceCodecCatalog + AssetReferenceGraphSnapshot`。任何改变稳定身份的动作必须从“解析”中剥离，成为有证据、可拒绝、可预览、可回滚、可恢复的显式事务。

## 2. 审查边界、证据等级与冻结快照

### 2.1 证据等级

| 等级 | 本轮使用方式 | 能证明什么 |
|---|---|---|
| E3 | 逐调用链读取Runtime reference DTO/resolver/persist writer、registry、full/targeted import、migration、watch、pipeline，Interface project/resource DTO及Editor reference/save消费面 | 当前身份解析、repair传播、move/rename、迁移和产品接线事实 |
| E2 | 盘点整个`zircon_runtime/src/asset` production/test树，检索全部`reference_repairs`、rename/move和`Conflict`调用点，并与父报告去重 | 没有隐藏consumer、显式move service或第二条Conflict构造路径 |
| E1 | 读取现有resolver、migration、watcher、sidecar move、Editor save/reference测试，但本轮不运行 | 测试意图、现有正向覆盖和缺失的错误绑定矩阵 |
| E0 | 未运行Cargo、Editor、真实项目rename/migration、source control、跨进程、fault、soak或benchmark | 不得宣称实现动态通过或性能优于参考引擎 |

### 2.2 冻结范围

| 范围 | 文件 / 物理行 / 非空行 / bytes / test attributes | fingerprint |
|---|---:|---|
| Runtime asset production | **386 / 63,517 / 58,250 / 2,231,172 / 270** | `84162C789B120A284CF1CDA5CFE31003A0CDF819715A293763B82E954843760C` |
| Runtime asset tests | **167 / 35,003 / 32,094 / 1,215,488 / 628** | `845C2427C6E850968542276E6730721D256CAAB7D0918D54936450A8920BB8A7` |
| Runtime Interface project/resource contracts | **51 / 2,674 / 2,388 / 83,475 / 29** | `F59789AE731D79F65734D66C78115CB93DC171C1909EFEECC2C177281C7D7053` |
| Editor asset/project consumers | **98 / 15,817 / 14,305 / 531,168 / 139** | `64F34F5E47516C9B23AB7FF347A3D74AD7B51294C81BA6C38B1407167130AC9D` |
| 五引擎参考切片 | **22 / 15,116 / 13,147 / 561,638 / 48** | `116E17DDF99930193EB04AEDD4022F78E65B5705D5CCA76594E16BB06DC7D5B7` |

fingerprint按规范化相对路径排序，串联“相对路径、NUL、单文件SHA-256、换行”后再计算SHA-256。冻结对象是2026-08-21当前共享工作树，不是仅HEAD内容；工作树存在其他Session的并行修改，因此实施前必须按`source_recheck_required`重新冻结。

### 2.3 本轮唯一owner边界

| 既有owner | 唯一拥有内容 | Runtime87只负责 |
|---|---|---|
| Interface02、Runtime24 | stable UUID算法、跨平台locator grammar、`AssetReference::new`不变量、双引用模型 | 消费其结果；不重复登记算法与DTO父问题 |
| Runtime25 | notify rename mode、URI escaping、case/Unicode、VFS与原子I/O | 显式asset mutation语义与引用修复事务 |
| Runtime51 | duplicate GUID remint closure、registry generation、redirect/tombstone机制、通用path fallback disposition | resolver冲突语义、repair admission和move产品闭环 |
| Runtime64 | handle/load/reload/lease/cache authority | 引用解析结果进入资源authority前的资格 |
| Runtime85 | stable subasset key、importer纯函数化、build/cook/package | 引用如何指向、迁移和修复subasset identity |
| Runtime86 | exact type/schema、project document codec、typed dependency graph | repair是否允许改变目标及其证据合同 |
| Editor04/10 | Asset Browser引用UX、source control、delete/rename交互、Editor projection | Runtime发布唯一同代resolution/repair/mutation事实 |
| Runtime04 failure | 缺失subasset不得回退父资产；当前源码已修，状态仍`validation_pending` | 引用当前实现证据，不抢占修复与validation owner |

## 3. 当前实现中可保留的工程底座

### 3.1 Project引用已经区分稳定身份与移动提示

`PersistedAssetReference`明确区分project和builtin；`AssetRef`保存GUID、`RelPath` path hint与可选subasset。`persist_runtime_reference`要求registry中的GUID与locator指向同一entry，并把locator转换成project-root-qualified path hint。方向正确：authoring文件不应只保存裸路径。

### 3.2 Registry与generation publication有真实事务外形

full generation先构建`ProjectedMetaInventory`，规范化sidecar并冻结import registry；targeted generation在manager clone上prepare，持久提交成功后才替换live manager。source与sidecar一起移动时，watch/change owner映射和projected inventory可保持GUID；现有测试覆盖split Removed+Added事件下的sidecar identity恢复。

### 3.3 Subasset错误不再降级为父资产

`entry_by_hint`会构造exact labeled locator；找不到时收集同源labeled candidates并返回`DanglingSubasset`。当前resolver测试明确断言`#MissingMesh`失败，而不是退回root。该修复方向必须保留，但候选仍缺GUID、type、stable subasset key和repair policy。

### 3.4 Migration已有dry-run、事务与恢复基础

asset migration具备dry-run/apply、sidecar预检、indexed resolver、change/issue report、multi-file transaction和recovery。问题不是没有事务基础，而是事务内容仍由shape-based walker发现，且resolver把“更新提示”与“更换稳定目标”视为同类自动repair。

## 4. 参考引擎差异与适用边界

| 引擎 | 本地源码证据 | 对Zircon的最低要求 | 不应误读 |
|---|---|---|---|
| Unreal | `FSoftObjectPath`分开top-level asset path与subobject path；`AssetRenameManager`查询referencers、检查只读/source control、修复soft path、按失败情况保留redirector；`UObjectRedirector`能在加载与registry中指向destination | move/rename是显式控制面，引用闭包、可写性、redirect与保存结果都必须有决定和回执 | Unreal实现很复杂，不等于所有rename都天然crash-atomic |
| Bevy | `AssetPath`分开source/path/label；typed/untyped `AssetId`保存type identity，index带generation，strong handle保存path/meta；processed hash纳入依赖hash | 路径、类型、运行时代际和稳定UUID必须是不同维度 | Bevy不是完整Editor rename/redirect产品，不能拿来替代该控制面 |
| Fyrox | `ResourceMoveContext`与`ResourceMovementError`显式校验destination、registry和root；`move_resource_by_path`移动资源、options/metadata并更新registry以保持UUID | Zircon至少要有asset-aware move operation，而不是只等filesystem watcher猜意图 | 其顺序文件操作不是Zircon最终的crash-atomic上限 |
| Godot | `ResourceUID`维护UID到path映射；loader/provider暴露`get_dependencies`和`rename_dependencies`，由格式owner改写引用 | reference rewrite必须由知道schema的codec/provider执行，UID与path可独立变化 | 不能由此推断Godot已解决所有source-control或事务问题 |
| Unity Graphics | package内consumer用GUID查path，并在serialized texture/cubemap/mesh中以GUID fallback恢复对象 | GUID/local object identity与path lookup是不同层，consumer需要显式恢复结果 | 本地`dev/Graphics`不含完整Unity AssetDatabase，只作consumer旁证 |

## 5. P0正确性阻断

### `AREF87-P0-001`：path hint可静默替换稳定GUID目标

**确定性证据链：**

1. `resolve_project_reference_from_lookup`先查GUID；GUID存在且label相同时只修path hint，这是正确分支。
2. GUID不存在时，resolver直接调用`entry_by_hint`，用path+subasset命中的entry构造`ReferenceRepairKind::Guid`。
3. GUID存在但label不匹配时，resolver仍调用`entry_by_hint`；只要path+label命中，就以该entry UUID替换原GUID。
4. `ReferenceResolutionError::Conflict`已声明，production resolver没有构造点；migration只有错误映射分支，无法收到真实Conflict。
5. 名为`resolution_reports_guid_path_repair_dangling_and_conflict_states`的测试没有任何Conflict断言，反而把missing GUID + occupied path断言为合法GUID repair。
6. migration的current-reference repair复用同一resolver，apply成功时可把该语义改绑写回authoring document。

**产品后果：** 删除资产A后在同一路径创建资产B，A的旧引用会自动改成B；已有root GUID加陈旧subasset label时，也可被path上的另一subasset GUID取代。类型、schema、content digest、source lineage、redirect、tombstone、operator意图均不参与决定，引用可以“看起来恢复成功”但实际指向完全不同对象。

**必须修复：** resolution只返回事实，不得改写稳定身份。GUID命中时path只能产生`StaleHint`；GUID缺失且path命中必须返回`PathOccupiedCandidate`或经可信redirect得到`Redirected`，默认fail closed。只有`ReferenceRepairPlanner`在校验tombstone/redirect、expected type/schema、source lineage、subasset stable key、registry generation及用户/策略授权后，才能生成semantic repair transaction。

### `AREF87-P0-002`：importer产生的引用修复在产品链中被丢弃

**确定性证据链：**

1. `AssetImportContext::resolve_project_asset_ref`把resolver的repair推入共享列表。
2. Material、Model、Scene importer把该列表写进`AssetImportOutcome.reference_repairs`。
3. 合同注释明确说明caller可呈现或持久化这些repair。
4. 全仓production检索只有合同、三个ingest importer和native adapter引用该字段；full/targeted generation、pipeline、Editor、migration和save都不读取。
5. full generation在`finish_successful_import`前后只消费entries、direct references和diagnostics；targeted generation同样只消费entries。
6. native importer response转换固定写`reference_repairs: Vec::new()`，第三方native importer无法返回等价repair。

**产品后果：** 导入后的运行时payload可使用resolver给出的新UUID，但源TOML仍保存旧GUID/path/subasset。重启、registry变化或同路径重用后结果可再次变化；用户看不到repair，Editor save也不会把它纳入同一事务。Runtime、Editor、native plugin和migration由此产生不同引用事实。

**必须修复：** import只能产出`ReferenceResolutionObservation`和可选repair proposal；ProjectManager必须在publication前聚合、去重并按source field address形成`ReferenceRepairPlan`。safe hint update与semantic target change使用不同policy；source document、sidecar、registry、artifact、catalog generation和repair receipt必须原子提交或全部回滚。native importer协议必须表达同一bounded typed observation，不得固定清空。

## 6. P1工程化差距（48项）

### 6.1 引用合同与解析状态（P1-001至P1-008）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| AREF87-P1-001 | resolver输入只有borrowed registry和roots，不携带registry/catalog/source generation | `AssetResolutionSnapshotId`与不可变snapshot lease |
| AREF87-P1-002 | exact lookup、诊断、safe repair和semantic repair共用一个自动fallback入口 | `ResolutionPolicy`分开LookupOnly/Diagnose/PlanSafeRepair/PlanSemanticRepair |
| AREF87-P1-003 | repair只以Guid/PathHint/Subasset三选一表示，多个字段同时变化时只记录第一个差异 | 完整field diff、reason set和pre/post condition |
| AREF87-P1-004 | repair没有evidence、confidence、redirect/tombstone命中或candidate provenance | 可审计`ResolutionEvidence`与拒绝理由 |
| AREF87-P1-005 | `Conflict`是死状态；错误枚举与真实状态机不一致 | 每个disposition有可达生产分支和conformance test |
| AREF87-P1-006 | repair admission不消费Runtime86要求的expected exact type/schema | type/schema mismatch必须阻止semantic repair |
| AREF87-P1-007 | 引用不保存source instance/content lineage，path复用无法与移动区分 | source lineage/content proof或可信redirect chain |
| AREF87-P1-008 | project持久writer只接受`res://`与`builtin://`，`lib://`和`package://`资产不能形成正式引用 | provider/package-qualified persistent reference variant |

### 6.2 Resolver snapshot、索引与预算（P1-009至P1-016）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| AREF87-P1-009 | 普通import resolver按每条引用遍历roots并执行filesystem stat/meta load | generation-built path/source/subasset index |
| AREF87-P1-010 | locator转path hint会逐root重开compound `.zmeta` | snapshot内canonical source projection，无热路径I/O |
| AREF87-P1-011 | full generation在所有import前冻结registry，fresh importer新产出的subasset本代不可见 | dependency-ordered discovery/declare/import两阶段snapshot |
| AREF87-P1-012 | targeted import只clone旧registry，当前source新建或改名subasset无法被同次解析 | source replacement overlay与self/cross-source provisional entries |
| AREF87-P1-013 | 同一document重复引用没有batch cache或一次性resolution plan | per-document dedup、vectorized lookup和stable order |
| AREF87-P1-014 | multi-root ambiguity只返回path，不列candidate root/source身份 | bounded structured candidate set |
| AREF87-P1-015 | filesystem resolver没有lookup count、byte/I/O/deadline/cancellation预算 | `ResolutionBudget`和terminal budget receipt |
| AREF87-P1-016 | lookup结束不验证snapshot是否仍是publication candidate | prepare/commit generation precondition与stale retry |

### 6.3 Import、restore、save与产品repair接线（P1-017至P1-024）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| AREF87-P1-017 | repaired reference不写回schema-owned source field | codec提供field address、patch与lossless/canonical write policy |
| AREF87-P1-018 | 没有repair event、project health item或Editor review queue | generation-bound repair observation stream |
| AREF87-P1-019 | 多importer/多字段repair不去重、不合并，也不检测相互冲突 | deterministic repair planner与conflict set |
| AREF87-P1-020 | repair与artifact/meta/catalog publication没有共同commit/rollback边界 | 单一`ReferenceRepairTransaction` |
| AREF87-P1-021 | native importer wire不携带repair observation，host固定清空 | versioned bounded native repair envelope |
| AREF87-P1-022 | restored artifact不携带原resolution snapshot/receipt，registry变化后可复用旧解析结果 | artifact reference-resolution digest与revalidation policy |
| AREF87-P1-023 | importer parse/validation failure与reference dangling/conflict没有独立状态和last-known-good规则 | typed import/reference compatibility state machine |
| AREF87-P1-024 | Editor save batch有`references_valid`开关，但production没有caller把真实resolution结果设为false | save preflight直接消费同代Runtime reference health snapshot |

### 6.4 Rename、move、copy、delete与redirect控制面（P1-025至P1-032）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| AREF87-P1-025 | Runtime/Editor没有asset-aware move/rename command，只有外部filesystem变化推断 | `AssetMutationService`显式Move/Rename/Copy/Delete请求 |
| AREF87-P1-026 | move前不冻结referencer closure、unknown codec或affected package | registry-generation-bound impact plan |
| AREF87-P1-027 | 无read-only/source-control/checkout/write-authority决策 | per-document writability decision与operator prompt/result |
| AREF87-P1-028 | 无case-only、batch、folder、cross-root、copy-vs-move语义 | typed operation kind与platform-independent destination policy |
| AREF87-P1-029 | source、sidecar、import options、auxiliary files和generated metadata没有统一mutation set | codec/importer声明owned companion set |
| AREF87-P1-030 | move流程不消费Runtime51的redirect/tombstone机制，也没有expiry/chain policy | mutation plan显式CreateRedirect/RewriteAll/Reject策略 |
| AREF87-P1-031 | open dirty document、active import、asset editor session和watch self-write没有冲突协议 | mutation quiescence barrier与dirty-document decision |
| AREF87-P1-032 | 没有prepare/save/rename/rewrite/publish各阶段receipt、补偿和crash recovery状态 | durable mutation journal与terminal receipt |

### 6.5 Migration、codec与repair治理（P1-033至P1-040）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| AREF87-P1-033 | current-reference walker递归把任意含`kind=project/guid/path_hint`的TOML table当引用 | schema/provider-owned visitor，禁止全局shape rewrite |
| AREF87-P1-034 | Material等局部special case与generic walker并存，没有统一codec catalog | `ReferenceCodecCatalog`注册enumerate/patch/validate/migrate |
| AREF87-P1-035 | retired migration和current repair最终复用同一自动resolver语义 | legacy decode、resolution、repair policy三阶段分离 |
| AREF87-P1-036 | apply模式没有safe-only默认值或semantic repair审批，会自动写GUID变化 | semantic change默认阻断，显式policy/approval后才提交 |
| AREF87-P1-037 | report只记录path与reference count，不含field、old/new、reason、evidence | per-reference structured before/after receipt |
| AREF87-P1-038 | report不记录registry/source generation、input fingerprint、codec/schema或policy | reproducible migration manifest |
| AREF87-P1-039 | 无include/exclude、type/provider、safe/semantic、max changes和dry-run diff policy | bounded migration policy与admission summary |
| AREF87-P1-040 | registry变化后没有resume/idempotency precondition，旧dry-run不能安全升级为apply | snapshot token、plan hash和compare-and-commit |

### 6.6 Subasset、图地址与资格覆盖（P1-041至P1-048）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| AREF87-P1-041 | `DanglingSubasset.candidates`只有locator，没有GUID、type、stable key或status | typed `SubassetCandidate` |
| AREF87-P1-042 | subasset label rename/split/merge没有专用repair planner | 消费Runtime85 stable key/lineage/redirect的subasset mutation plan |
| AREF87-P1-043 | `AssetRef.sub`只是flat display string，不能表达provider-owned hierarchical/local object identity | versioned `SubassetKey`与可选display path |
| AREF87-P1-044 | resolution/repair没有document field address、source span或reference ordinal | `ReferenceSiteId`与codec-owned patch address |
| AREF87-P1-045 | 没有“删除A、同路径创建B、旧A引用必须拒绝”的回归测试 | path-reuse wrong-object mandatory gate |
| AREF87-P1-046 | 没有full generation中fresh source引用另一fresh source新subasset的测试 | declare/import phase cross-source subasset gate |
| AREF87-P1-047 | 没有证明full/targeted/restore/native路径消费同一repair plan的测试 | four-path parity与restart fixture |
| AREF87-P1-048 | 没有百万引用、批量move、registry churn、fault/crash/source-control的资格基线 | correctness-first scale/fault/soak benchmark matrix |

## 7. P2质量与维护性差距（12项）

| ID | 当前问题 | 整改方向 |
|---|---|---|
| AREF87-P2-001 | GUID、UUID、AssetId、ResourceId在同一引用链混用 | 术语表区分persistent identity、runtime slot与locator |
| AREF87-P2-002 | resolver测试名声称覆盖Conflict，实际没有Conflict断言 | 按disposition拆分命名精确的测试 |
| AREF87-P2-003 | `AssetReference`的Display只输出locator，日志中看不到UUID | diagnostic display同时携带qualified identity |
| AREF87-P2-004 | `AssetImportOutcome`注释承诺caller可持久repair，但未指定owner和commit合同 | 注释链接正式service/receipt合同 |
| AREF87-P2-005 | `ReferenceRepair`字段公开，可构造kind与stale/resolved diff不一致的值 | 私有字段、validated constructor与完整diff |
| AREF87-P2-006 | 单个resolver测试混合exact、GUID repair、subasset、dangling和occupied hint | truth-table fixture与小型focused tests |
| AREF87-P2-007 | labeled locator通过字符串`format!("{base}#{sub}")`重建 | structured `with_subasset_key` API |
| AREF87-P2-008 | `ProjectSourceLookup`注释声称属于一代，trait本身没有代际证明 | 类型上携带snapshot id/lease |
| AREF87-P2-009 | `Registry { message: String }`混合parser、path和registry invariant错误 | stable diagnostic code与typed source chain |
| AREF87-P2-010 | migration文本把Rust `Debug`枚举名当输出协议 | versioned machine report，文本只作projection |
| AREF87-P2-011 | ambiguity、dangling与subasset candidate各自使用ad hoc字段 | 共用bounded `ResolutionCandidateSet` |
| AREF87-P2-012 | 没有一张代码旁truth table定义GUID/path/sub/redirect组合优先级 | executable decision table与文档生成 |

## 8. 目标架构

### 8.1 `QualifiedAssetReference`

持久引用由stable asset identity、provider/package/project qualification、可选stable subasset key、fallback locator/path hint、expected type/schema和reference schema version组成。path永远是诊断与迁移提示，不能在无授权时替换identity。

### 8.2 `AssetResolutionSnapshot`

Project generation冻结UUID、path、source lineage、subasset、redirect、tombstone、type/schema与referencer indexes。所有lookup只读该snapshot，不在每条引用上访问filesystem；结果携带snapshot id和currentness precondition。

### 8.3 `AssetReferenceResolver`

resolver是纯查询，返回`Exact`、`StaleHint`、`Redirected`、`MissingIdentity`、`PathOccupiedCandidate`、`DanglingSubasset`、`Conflict`、`Unsupported`等typed disposition。它不修改source、不选择semantic replacement，也不把candidate伪装成success。

### 8.4 `ReferenceRepairPlanner`

planner消费resolution evidence、policy、expected type/schema、source lineage、redirect/tombstone与codec field address。safe hint update和semantic target change分开；输出plan hash、before/after、reason、approval requirement和commit preconditions。

### 8.5 `AssetMutationTransaction`

Move/Rename/Copy/Delete先冻结referencer closure与owned companion set，预检destination、dirty document、source control、unknown codec和active generation，再stage source/meta/reference rewrites/redirects，最后原子publish registry/resource/catalog generation。失败保留last-good并提供恢复journal。

### 8.6 `ReferenceCodecCatalog`与`AssetReferenceGraphSnapshot`

每种project document/provider注册enumerate、validate、patch、migrate和owned companion合同；禁止shape-based全局rewrite。Runtime从同一codec结果发布versioned outgoing/incoming graph，Editor只投影视图和用户决策，不重建第二份authority。

## 9. 重构里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 正确性冻结 | 加入path reuse、GUID/sub conflict、repair dropped与restart reproduction；默认禁止semantic auto-repair | 两项P0有失败fixture且旧行为被结构门锁定 |
| M1 引用合同 | 定义qualified reference、subasset key、expected type/schema与schema version；消费Interface02/Runtime24/85/86依赖 | 新旧reference有显式migration与golden corpus |
| M2 Snapshot resolver | 建立generation-bound indexes、纯resolver、typed dispositions和budget | lookup无filesystem热I/O，Conflict真正可达 |
| M3 Repair planner | 建safe/semantic policy、evidence、plan hash、approval和structured receipt | identity变化不再发生在resolver内部 |
| M4 Import/restore接线 | full/targeted/restore/native统一产生并提交repair plan | source/artifact/meta/registry/catalog同事务或全回滚 |
| M5 Asset mutation | 显式move/rename/copy/delete、referencer closure、companion set、redirect与recovery | batch/case-only/cross-root/source-control fault门通过 |
| M6 Codec migration | provider-owned visitor替换shape walker，建立machine report和resume precondition | unknown codec fail closed，dry-run/apply可复验 |
| M7 Editor产品闭环 | save preflight、repair review、dirty/source-control决策、同代graph projection | Asset Browser显示exact/stale/missing/conflict且不自动改绑 |
| M8 资格与性能 | 跨平台、百万引用、批量mutation、fault/crash/soak及同方法参考benchmark | 48项资格门均有baseline/corpus/hash/receipt |

## 10. 资格门（48项）

### 10.1 身份与解析语义（G01-G10）

| Gate | 必须证明 |
|---|---|
| G01 | GUID存在时path只能更新提示，不能替换稳定目标 |
| G02 | GUID缺失且path被新资产占用时默认fail closed |
| G03 | GUID存在但subasset不匹配时不得采用path上的另一GUID |
| G04 | `Conflict`、`PathOccupiedCandidate`和`DanglingSubasset`均有production可达测试 |
| G05 | redirect/tombstone与普通path fallback得到不同disposition |
| G06 | expected type/schema不匹配阻止semantic repair |
| G07 | source lineage/content proof不匹配阻止path candidate |
| G08 | project/library/package/builtin reference各有明确persist/resolve策略 |
| G09 | locator/path hint/subasset round-trip不依赖宿主OS grammar |
| G10 | reference schema too-new/too-old/unknown provider明确fail closed |

### 10.2 Snapshot、索引与预算（G11-G18）

| Gate | 必须证明 |
|---|---|
| G11 | 每个resolution结果携带registry/catalog/source snapshot id |
| G12 | commit前snapshot失效会重试或拒绝，不发布stale plan |
| G13 | normal import reference lookup不执行per-reference filesystem scan/meta load |
| G14 | repeated references在document内去重并稳定排序 |
| G15 | multi-root ambiguity返回有界candidate及root identity |
| G16 | fresh cross-source subasset在同次generation按声明阶段可解析 |
| G17 | targeted source replacement可解析自身provisional subasset |
| G18 | lookup count、I/O、bytes、time、depth和candidate数量均有预算与receipt |

### 10.3 Repair与import产品链（G19-G28）

| Gate | 必须证明 |
|---|---|
| G19 | resolver是纯查询，源文件和registry无副作用 |
| G20 | safe hint repair与semantic identity repair使用不同类型和policy |
| G21 | repair记录field address、old/new、reason、evidence、snapshot和plan hash |
| G22 | full generation消费repair plan而非丢弃列表 |
| G23 | targeted generation与full得到相同repair disposition |
| G24 | artifact restore验证resolution digest或重新解析 |
| G25 | native importer可返回同一typed bounded observation |
| G26 | 多字段/多importer repair可确定去重并检测冲突 |
| G27 | source、meta、artifact、registry、resource和catalog原子提交或回滚 |
| G28 | restart后resolution与提交前receipt一致，不因path reuse漂移 |

### 10.4 Move、rename、redirect与migration（G29-G40）

| Gate | 必须证明 |
|---|---|
| G29 | 产品move/rename只能经过`AssetMutationService` |
| G30 | move前冻结完整referencer closure并拒绝unknown codec |
| G31 | source、sidecar、options、auxiliary和generated metadata同plan处理 |
| G32 | case-only、batch、folder、cross-root、copy与move语义可区分 |
| G33 | destination collision/overwrite/read-only/source-control均有typed decision |
| G34 | dirty open document和active import在用户决策前不会被覆盖 |
| G35 | redirect创建、全量rewrite、expiry与chain collapse有明确policy |
| G36 | crash发生在任一mutation阶段都能恢复last-good或继续journal |
| G37 | current migration只访问codec声明的reference sites |
| G38 | semantic GUID变化在apply前必须有显式授权 |
| G39 | dry-run report可逐引用复验before/after/evidence |
| G40 | dry-run升级apply时校验snapshot token与plan hash |

### 10.5 Editor、回归与性能（G41-G48）

| Gate | 必须证明 |
|---|---|
| G41 | Editor save preflight消费Runtime同代reference health |
| G42 | Asset Browser区分Exact/StaleHint/Missing/Occupied/Conflict/Redirected |
| G43 | Editor不再用locator fallback替换未知UUID |
| G44 | Runtime与Editor outgoing/incoming graph在同代完全一致 |
| G45 | path reuse、subasset rename/split/merge、delete/create race进入required lane |
| G46 | full/targeted/restore/native/migration五路径有parity与restart矩阵 |
| G47 | 百万引用、批量move与registry churn在正确性门后满足固定预算 |
| G48 | benchmark/fault/soak记录hardware、corpus、hash、source fingerprint和terminal receipt |

## 11. 禁止的临时实现

- 禁止继续把path hint命中直接写成GUID repair。
- 禁止在resolver中自动选择“最像”的资产、同名subasset或父资产。
- 禁止用更多`Conflict`枚举成员掩盖生产分支仍不可达。
- 禁止只在Editor弹提示，而Runtime import/migration仍静默改绑。
- 禁止继续收集`reference_repairs`后丢弃，或让native adapter固定清空。
- 禁止通过全量`reimport_all`代替asset move/delete transaction。
- 禁止要求用户手工同时移动source与`.zmeta`，再把watcher推断当正式rename协议。
- 禁止用path、display label、数组index或content相似度充当stable subasset identity。
- 禁止generic TOML/JSON walker按字段形状扫描和改写所有document。
- 禁止未知codec、只读referencer或dirty document被当作“无引用”继续提交。
- 禁止把redirect永久保留且没有owner、expiry、chain和tombstone策略。
- 禁止在48项正确性门关闭前宣称引用系统达到或超过Unreal。
- 禁止把未来tooling迁移Rust当作Runtime reference authority已经成立。

## 12. 完成边界

本报告完成的是当前源码静态审查、参考引擎对照、父owner去重和重构需求登记，不是代码修复完成。只有M0-M8按48项资格门取得可复验回执，Interface02、Runtime24/25/51/64/85/86、Runtime04 failure与Editor04/10的依赖边界完成，且Runtime、Editor、native importer和migration消费同一resolution snapshot、repair planner与mutation transaction后，`implementation_status`才可改为`complete`。

本轮未修改Rust、Cargo、资源、plugin或工具实现，未运行Cargo、Editor、真实rename/migration、source-control、fault、soak或benchmark。用户已要求暂不考虑tooling优化；本文只规定Runtime合同、Editor handoff与产品资格，不评价未来将迁移为Rust的工具实现。
