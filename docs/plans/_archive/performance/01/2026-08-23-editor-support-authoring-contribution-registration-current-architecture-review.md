---
related_code:
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_extension/template_contributions.rs
  - zircon_editor/src/core/extension/store/batch.rs
  - zircon_editor/src/core/plugin/registration.rs
  - zircon_plugins/first_party_editor_catalog/src
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-23-first-party-editor-plugin-catalog-instance-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeRegistry.h
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetToolsModule.cpp
tests:
  - current editor_support 1 of 1 Rust file and its inline test reviewed
  - downstream EditorExtensionRegistry and ContributionBatch registration paths reviewed
  - focused rustfmt passed and plugin structure audit passed 39 of 39 manifests
  - focused document paths passed with 0 violations; global docs gate has unrelated drift
  - current-source Cargo, startup benchmark, allocator, WPR and power pending
doc_type: implementation-evidence
status: source_reviewed_no_local_hot_path_dynamic_blocked
---

# Editor support authoring contribution注册复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_plugins/editor_support/src/lib.rs`当前**1/1**个Rust文件、**334物理行、12,860 B、1 test**；
SHA-256为`49c76fd9d51968187683789afd1c68c035e5f9946a4ed92b66076a380eed1298`，目录当前clean。
为避免只看helper名称猜测热点，本轮同时追到`EditorExtensionRegistry`、`ContributionBatch`、
`EditorPluginRegistrationReport::from_plugin`及首方editor catalog调用者。

## 当前源码判定

### 局部通过：不是帧路径，也没有重复发布或主线程等待

`register_authoring_extensions`注册一个drawer、一个UI template和S个surface；每个surface生成一个view、一个open
operation、一个menu item。`register_authoring_contribution_batch`以move消费10类descriptor Vec，逐项写入候选registry。
本模块没有文件I/O、线程创建、锁、channel、sleep、foreign callback、运行时tick或逐帧查询。

下游注册方法只执行ID/path/schema校验和`BTreeMap::entry`唯一插入，不更新generation、不发布快照、不重建全局索引；
因此不能把“batch内部仍有for循环”误判为B次发布，也没有依据在这里引入多线程。单次插件注册的局部规模为
`O(C log C)`，其中C为该插件贡献数；surface helper固定为3次候选插入。

### 结构性待优化：同一贡献经过两棵有序树与两轮验证

插件先把descriptor注册到`EditorExtensionRegistry`，UI host materialization随后调用`into_contribution_batch`，按family
取出值并重新插入`ContributionBatch`。ID/path/schema、graph palette node唯一性和inspector surface等校验因此可能执行
两次，key也在两阶段分别构造；两组结构各含最多18个family map。`register_inspector_customization`还会先创建临时
`ContributionBatch`并clone descriptor做一致性校验，materialization时再将descriptor转为`Arc<dyn
InspectorCustomization>`并验证。

这是真实的冷启动/插件重载额外工作，但当前没有证据证明它是独立P0热点。更上层
`first_party_editor_catalog`会反复调用`EditorPluginRegistrationReport::from_plugin`，每次从零创建候选/final registry、
runtime consumer与mutable editor instance；该重复generation会把这里的两阶段成本按投影次数放大。正确修复owner仍是
PERF-MVP-629的schema/instance generation和一次性materialization，不是在共享helper中换HashMap、跳过校验或缓存完整
registration report。

### API细节：查询面会分配，但不属于本模块调用热路

`EditorExtensionRegistry`的`views()`、`drawers()`、`menu_items()`等多数查询返回新`Vec<&T>`；UI template pane source查询
还重建`BTreeMap<String, Arc<_>>`。`editor_support`注册函数本身不调用这些查询。后续catalog/materialization收敛时应把稳定
投影改为borrowed iterator/range或generation-owned snapshot，并以调用计数确认，不应把该问题计成本模块当前帧开销。

## Unreal源码依据

`EditorModeRegistry.h:35-56,81-85,108-136,168`把mode metadata/factory保存在registry map，明确要求在module
`StartupModule()`注册、`ShutdownModule()`注销，并只在`CreateMode()`创建实例。可转移原则是：descriptor/factory schema
按module/catalog generation构建一次，mutable mode instance按editor session/激活生命周期创建；查询或菜单投影不能反复
重跑模块注册。

`AssetToolsModule.cpp:13-24,26-42`同样把AssetTools与MessageLog注册放在module启动/关闭边界，不把注册工作放进编辑器帧循环。
Zircon的candidate registry -> validated contribution batch提供原子失败语义，应保留；但在schema generation冻结后应由单个
validator/materializer直接产出最终有序owner ranges，避免相同descriptor在两个通用registry中重复建树。

## 结构优化计划与边界

1. PERF-MVP-629/Plan02 M1+M5先建立process级immutable `EditorProviderSchemaGeneration`，同provider/process的plugin
   descriptor、capability、extension schema与consumer factory build不超过1次。
2. Editor session只创建一次`EditorPluginInstanceGeneration`；mutable mirror、mode factory实例、consumer/lifecycle state不跨
   session共享，stable menu/status/capability查询不得重跑`register_editor_extensions`。
3. `EditorExtensionRegistry -> ContributionBatch`收敛为单次validate/materialize transaction：按owner生成family ranges，失败
   publish=0；禁止通过公开未验证字段、compat shim或全局cached report绕开事务。
4. 只有counter证明有序查询不在注册期使用、且稳定顺序可由generation末尾一次排序提供时，才评估flat Vec/dense slot；
   不以容器偏好替代数据。

## 量化验收

矩阵为providers 0/1/2/21/100、每provider contributions 0/1/10/100/1k、sessions 1/2/100、stable projections
1/1k、reload/unload 0/1/100及invalid/duplicate位于首/中/尾。记录schema/instance/registration/validation/materialization
build count、family insert/compare/key bytes、descriptor clone bytes、map node allocations、peak RSS、startup/reload
p50/p95、main-thread CPU、locks、wakeups与energy。

验收要求schema build<=1/provider/process、instance build<=1/provider/session、registration<=1/provider/session、stable
projection registration=0；每个descriptor validation=1、最终family materialization=1、失败publish=0；跨session mutable owner
共享=0、unload后owner contribution/consumer/mode instance=0。复杂度以compare/visit/alloc counter证明接近
`O(C log C)`或generation末尾`O(C log C)`单次排序，不能只报wall time。

本轮不改生产Rust：当前局部代码没有可独立修复的帧级热点，提前删除第二阶段校验会破坏事务边界。current-source Cargo、
真实editor startup/reload benchmark、allocator、F4 WPR/CPU sampling/RSS/power仍pending；该非渲染切片不要求RenderDoc。
focused rustfmt与39/39 plugin manifest结构审计通过；docs gate全局3,094份文档中275份受影响、801条路径违规，本轮两份文档
过滤结果为0。全局漂移不在本切片批量猜修。因此继续留`pending.md`，不迁入`review.md`、不提交milestone、不发送完成企微。
