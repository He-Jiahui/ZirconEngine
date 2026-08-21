---
title: Runtime Asset Import、Source Discovery、Importer Recipe、Subasset、Derived Data、Artifact、Cook、Package、Incremental Build、Worker、Determinism 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime85
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/pipeline
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/virtual_geometry_cook
  - zircon_runtime/src/asset/mesh_sdf_cook
  - zircon_runtime/src/asset/pack
  - zircon_runtime/src/bin/zircon_export_pack
  - zircon_editor/src/core/export
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard
tests:
  - zircon_runtime/src/asset
  - zircon_runtime/tests
  - zircon_editor/src/core/export
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildDefinition.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildInputs.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildOutput.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildScheduler.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildSession.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildWorker.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Cooker/CookPackageData.h
  - dev/UnrealEngine/Engine/Source/Developer/IoStoreUtilities/Private/IoStoreWriter.h
  - dev/godot/core/io/resource_importer.h
  - dev/godot/core/io/resource_importer.cpp
  - dev/godot/editor/file_system/editor_file_system.h
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/processor/process.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Asset Import、Source Discovery、Importer Recipe、Subasset、Derived Data、Artifact、Cook、Package、Incremental Build、Worker、Determinism 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前资产链已经超过“按扩展名读文件后塞进HashMap”的临时实现。它具备copy-on-write importer registry、full suffix/extension/plugin索引、v7 sidecar meta、root/labeled subasset、依赖投影、候选代构建、targeted import、durable journal/recovery、BLAKE3内容校验、zstd artifact、64 KiB immutable chunk、resident chunk LRU、bounded/single-flight worker、确定性pack排序与去重、delta/install/staging/promotion receipt，以及Editor CookAssets到Pack的阶段外形。这些底座必须保留，不能因重构目标更高而被误判为“什么都没有”。

但当前链条仍由“同步导入函数 + 文件级sidecar + ResourceId命名artifact + 手写导出manifest”拼接，而不是由不可变source snapshot、versioned recipe、build graph、content-addressed DDC、target cook variant、可抢占worker和canonical pack compiler共同驱动。全量扫描会顺序读取、哈希、导入并发布整个候选代；targeted import虽已真实存在，但与full path并行维护复杂提交逻辑。artifact manifest没有producer、recipe、toolchain、target和dependency provenance，cook又嵌在glTF/OBJ/model importer同步调用内，pack最终读取用户清单指定的raw source bytes，而不是消费同一资产图产生的qualified artifact。

本轮新增1项P0：`.bin`及字体文件被source discovery当作auxiliary排除，单文件`AssetImportSource`又不携带`included_files/included_paths`；restore key只覆盖根文件bytes、settings config hash和importer id/version。与此同时glTF decoder会从根文件目录直接打开external buffer/image URI，font importer也会直接打开manifest指向的外部字体。watch只把辅助文件变化映射成它自己的URI，没有“辅助源 -> 父资产”的反向owner索引。因此外部buffer、image或font bytes改变后，父资产可以不重建并命中旧artifact。这不是性能欠佳，而是可发布内容可能静默陈旧的正确性阻断。

Runtime04、Runtime51、Runtime64、Editor04/32/35及其开放failure继续拥有通用source index、registry、artifact migration/chunk store、watch boundedness、reload、格式语义、VG cook等父问题；Runtime85不重复累计。除上述新P0外，本报告登记48项P1、12项P2和48项资格门，目标是把资产链收敛为一份`AssetBuildGraph`和一套可证明的source-to-install transaction，而不是继续在各格式importer、Editor wizard和pack binary中各补一层临时流程。

## 2. 审查边界、证据等级与冻结快照

### 2.1 证据等级

| 等级 | 本轮使用方式 | 能证明什么 |
|---|---|---|
| E3 | 逐文件读取importer/project/artifact/pipeline/watch/cook/pack及Editor export生产代码，并沿source、meta、restore、worker、pack调用链追踪 | 当前合同、owner、同步边界、持久化事实和具体断链 |
| E2 | 检索importer注册/consumer、dependency、cache key、watch reverse mapping、cook、export manifest与产品stage handoff | 缺失owner、字段无consumer、产品链分叉等静态事实 |
| E1 | 读取runtime asset tests、export tests和3项ignored test，但本轮未运行 | 测试意图、静态覆盖及仍待托管验证的边界 |
| E0 | 未运行Cargo、Editor、真实大项目import/cook/package、fault、soak、跨机determinism或benchmark | 不得宣称动态通过、可恢复性或性能优于Unreal |

### 2.2 Zircon冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | fingerprint |
|---|---:|---|
| Artifact | 18 / 6,235 / 5,737 / 222,002 / 18 | `630d8f142eabde97469f1691fa89f9f7ee332e62140ed907b3546bfb3653335a` |
| Importer | 52 / 9,788 / 9,066 / 346,855 / 55 | `b04c00f055dbe390934ab13752385ba76b3bfa22b1c02e3c2518229ffabd5ab0` |
| Project | 41 / 7,255 / 6,642 / 257,364 / 29 | `02953a4bc537d176bf3e2731ead1b4be9aa90f3450e6225795d8a309bd083620` |
| Pack | 17 / 1,968 / 1,773 / 64,553 / 1 | `21cd847d7a7a097e8b563dc1a145856c649b3d2219171a7e6614d0c5fcefe5ec` |
| Pipeline | 61 / 7,393 / 6,744 / 275,164 / 42，另有3项ignored | `60fe8ac874b038875074b272a9b1ef74dd3bb9d88e66bd19c502d226296a0fcb` |
| Watch | 19 / 817 / 753 / 26,537 / 0 | `cfd1432aefbaa0a42ed90cd2150338da183e84d3d06a1a067a397eb7d20ffce9` |
| Virtual Geometry cook | 6 / 1,237 / 1,132 / 40,029 / 3 | `cbafc3e5ba48beae64b59a35ce6aa0fb4ea3bcafa146f0fc19bf9343d80ba98b` |
| Mesh SDF cook | 8 / 1,084 / 991 / 36,342 / 9 | `b907a1051bdf16ebfaaabafc249718205f3d8eebdadcb84c9404466da3d6bd04` |
| Runtime85 core去重合计 | **222 / 35,777 / 32,838 / 1,268,846 / 157** | `255aca3da8942186a000643717cb7d65c6858005fb9b885626a8f248ba95d35d` |
| Runtime asset tests | **167 / 35,003 / 32,094 / 1,215,488 / 628** | `11e2242205cdbaf5a68c3da2685063b9ebd7a8d3bcc300f95ee7312788f47a69` |
| Runtime export pack binary | **6 / 922 / 869 / 32,795 / 4** | `7c0062ce8990cdc787d85709ac99ae4891bbe29ff5327f924c2413998e62afe3` |
| Editor export touchpoints | **44 / 11,308 / 10,321 / 386,306 / 90**，另有1项ignored | `6d906242cf4825e4d619c1eb7cb40d752414818154dc640099c28088071f2061` |

冻结时相关工作树包含21个dirty/untracked source路径，集中在artifact store、glTF/font/texture/model/mesh/shader importer、project manager/pipeline和Editor wizard execution。本报告审查的是冻结时当前工作树；存在实现不等于并行Session已完成managed validation或accepted integration。

### 2.3 参考冻结范围

| 参考 | 本轮使用边界 |
|---|---|
| Unreal DDC / Cooker / IoStore | immutable build definition/input/output、function/constant/file/bulk/build依赖、scheduler/worker/session、target package cook、block/index/compression/encryption/signing/mount |
| Godot ResourceImporter / EditorFileSystem | importer option/save extension/order/priority、scan/reimport/source-change/group orchestration |
| Bevy AssetProcessor | source到processed asset、Load/Process/Ignore meta、loader/transformer settings、source/full hash、process dependency与write-ahead recovery |
| Fyrox ResourceManager | async loader、typed import options、reload/event/lifecycle；作为中型Rust引擎底座，不作为目标上限 |
| Unity Graphics | `ScriptedImporter` consumer声明source/artifact/custom dependency和subasset，`AssetReimportUtils`批处理；本地corpus不包含完整Unity AssetDatabase/DDC |
| 参考去重合计 | **19个文件 / 15,122行 / 13,151非空行 / 585,539 bytes**；fingerprint `befd038b847096384f8d909bf6ddeda12929c05beaa027cc5ef31d0993e965f4` |

不能把参考源码的局部API数量当成优越性证明。Unreal用于定义工程级build/DDC/cook/package上限；Godot、Bevy、Fyrox用于核对较轻量架构也不应丢失的依赖、meta、异步和生命周期合同；Unity Graphics只提供包内consumer旁证。

## 3. 当前链路与可保留底座

### 3.1 Source discovery、meta与candidate publication

`ProjectAssetManager`会递归发现source、按URI排序、建立candidate state，加载或创建v7 sidecar meta，校验不安全link、重复URI/GUID，投影依赖并通过durable transaction发布meta、artifact、registry和catalog。journal/recovery、candidate generation和atomic publication方向正确。targeted import也不再只是full rescan别名：它建立目标source/artifact/meta/registry/dependency closure并提交局部候选代；单一watch事件可进入该路径，rename/batch/reconciliation则保守走full scan。

### 3.2 Importer registry与outcome

`AssetImporterDescriptor`包含id、plugin、priority、extension、full suffix、output/additional kind、importer version和capability；registry使用copy-on-write snapshot及extension/suffix/id/plugin索引，并支持plugin unload。`AssetImportOutcome`能携带root、labeled entries、dependencies和migration diagnostics。这是一份可演进的registry骨架。

### 3.3 Artifact store

v5 artifact会bincode DTO、zstd staging、BLAKE3校验，并把内容切为64 KiB immutable chunk；读取侧校验range/hash，具备64 MiB resident chunk LRU以及2 GiB raw/4 MiB manifest上限。和“每次直接读源文件”相比，这是真实的派生artifact与驻留控制底座。

### 3.4 Worker与cook

当前`AssetWorkerPool`已经按pool capacity限制unique request，支持single-flight、bounded waiter/completion/bytes、TTL、`Arc` payload、panic捕获、cancel flag和diagnostics。旧报告若仍称它“无界且每个waiter深拷贝payload”已经过时。VG与Mesh SDF也有真实设置、预算和确定性测试；问题在于它们仍是import-time同步子过程，尚未进入共享build graph。

### 3.5 Pack与install

`ZrPackWriter`按path排序、内容hash去重并生成v1 JSON manifest；reader检查range/hash。delta、staging、promotion和receipt也已存在。这些机制可作为未来container compiler/install service的最小内核，但当前pack输入authority和内存模型不足以承担大型产品。

## 4. P0归属与新增阻断

### 4.1 既有父owner

| 父owner | 继续开放的边界 | Runtime85处理方式 |
|---|---|---|
| Runtime04 | source/import generation index、chunk store、migration、watch debounce、dynamic reload、typed event、VG generation等开放failure | 作为父依赖引用，不重复计P0 |
| Runtime51 | Asset Registry持久化、增量查询与产品接线 | 不把registry rebuild问题复制为新finding |
| Runtime64 | Resource authority、version lease、dependency/reload/cancel | 只定义build-time与runtime residency的边界 |
| Editor04 | Asset Browser/import/reimport/catalog/reference workflow | Runtime85只拥有runtime build authority和handoff |
| Editor32/35 | mesh/model/texture格式语义、LOD、compression、preview authoring | 格式缺口只列产品影响，不重复父P0 |
| Runtime/Editor export既有报告 | 通用export preset、平台bundle和host operation | tooling实现质量按用户要求不纳入本轮 |

### 4.2 `ASSET85-P0-001`：辅助源未进入父资产构建身份，可静默恢复陈旧artifact

**证据链：**

1. `project/manager/collect_files.rs`把`.bin`、`.ttf`、`.otf`、`.woff`和`.woff2`等归为auxiliary，不建立独立可导入root。
2. 单文件`AssetImportSource`的`included_files`和`included_paths`始终为空；full/targeted source digest只哈希根`source_bytes`。
3. restore只校验source digest、settings config hash、importer id/version；没有外部source dependency content digest。
4. `ingest/gltf_decode.rs`会用`source_path.parent()`直接读取external buffer/image URI；font importer也会读取manifest相对路径所指字体bytes。
5. watch把变化路径映射到它自身的asset URI，但系统没有auxiliary URI到父source/import action的反向索引。

**影响：** 修改`.gltf`旁的`.bin`或image、修改font manifest引用的字体后，父root可能既不被targeted import选中，也不因restore key变化而失效；旧artifact可继续发布。CI clean build和开发机增量build因此可能产出不同内容，且没有terminal diagnostic说明“父资产仍使用旧外部bytes”。

**必须修复：** source discovery先生成immutable `AssetSourceSnapshot`，其中每个included file有canonical URI、content digest、size和role；importer只能通过snapshot resolver读取声明依赖。第一次dependency discovery结果进入反向索引和build key，后续watch按父action失效。动态未声明读取必须fail closed或显式转为discovery restart，不允许绕过snapshot直接开文件。

**验收：** glTF external buffer、external image、font external blob分别具备“只改辅助文件”的增量测试；同一输入在clean/incremental、不同枚举顺序和两台机器得到相同build key/artifact hash；删除/rename/权限失败产生typed terminal result且last-known-good策略明确。

## 5. P1工程化差距（48项）

### 5.1 Source、recipe与importer contract

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ASSET85-P1-001 | source dependency只在import outcome表达逻辑资产关系，不是可重放的原始输入图 | typed source dependency graph，区分included file、logical asset、tool和environment |
| ASSET85-P1-002 | importer可按任意filesystem path再次开文件，source snapshot不封闭 | immutable snapshot resolver与declared read receipt |
| ASSET85-P1-003 | settings是untyped TOML，缺schema、默认值来源、迁移和recipe version | versioned typed `AssetImportRecipe`及canonical serialization |
| ASSET85-P1-004 | `LibraryCacheKey`未进入主链且用`DefaultHasher`，字段只有source/importer version/config | stable cryptographic build key，包含function/recipe/inputs/target/toolchain |
| ASSET85-P1-005 | import context没有target platform、build profile、engine ABI、compiler/tool version | qualified build context与显式compatibility domain |
| ASSET85-P1-006 | handler同步接收完整`Vec<u8>`，大资产必须整份驻留 | bounded streaming/random-access source view与backpressure |
| ASSET85-P1-007 | importer没有deadline、cooperative cancellation、work/memory/I/O budget | operation-scoped budget、cancel token和terminal receipt |
| ASSET85-P1-008 | native importer envelope携带完整JSON metadata/DTO和bytes，reserved artifact bytes未形成协议 | bounded binary protocol、schema negotiation、streamed artifact channel |
| ASSET85-P1-009 | plugin/native importer无进程隔离、watchdog、quarantine和crash attribution | sandboxed worker generation与provider health policy |
| ASSET85-P1-010 | registry虽有COW generation和unload，但调用者没有显式generation lease/quiescence合同 | snapshot lease、in-flight drain、retired provider和unload receipt |
| ASSET85-P1-011 | recognition主要靠extension/full suffix | MIME/magic/sniff及冲突决议记录，suffix只作候选索引 |
| ASSET85-P1-012 | descriptor capability与真实可执行格式存在差异，Sound等路径主要由测试证明 | executable capability probe、settings schema introspection和产品conformance |

### 5.2 Subasset、dependency与publication

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ASSET85-P1-013 | glTF常用`Texture0`、`Mesh0/Primitive0`等index label，重排会错配identity | stable source UID/content lineage与label仅作display name |
| ASSET85-P1-014 | subasset只按exact label保UUID，没有redirect、tombstone、remap history | versioned `SubassetIdentityRegistry`与引用修复计划 |
| ASSET85-P1-015 | artifact/output没有producer receipt、output schema和完整provenance | immutable build output descriptor与producer attestation |
| ASSET85-P1-016 | dependency集合混合source、artifact和runtime load语义 | typed edge kind、strength、phase、target和owner |
| ASSET85-P1-017 | restore会读取root artifact以重建部分手写dependency | dependency manifest独立、可查询且无需materialize payload |
| ASSET85-P1-018 | failure/last-good/retry/negative cache没有统一状态机 | per-action state、retry policy、last-known-good和failure artifact |
| ASSET85-P1-019 | full与targeted路径并行维护复杂meta/artifact/registry/catalog提交 | 一个build graph和同一publication transaction，scope只是输入集合 |
| ASSET85-P1-020 | reference repair可从importer context进入互斥side effect | importer纯函数化；修复作为独立validated mutation plan提交 |

### 5.3 Build graph、DDC与artifact

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ASSET85-P1-021 | 没有canonical asset build graph和显式node/action state | source/import/transform/cook/package DAG与incremental scheduler |
| ASSET85-P1-022 | 依赖内容hash不进入每个action key | transitive input digest与精确invalidation explanation |
| ASSET85-P1-023 | 只有本地project artifact，没有local/shared/remote分层DDC | hierarchical content-addressed cache与trust policy |
| ASSET85-P1-024 | v5 manifest缺producer、recipe、toolchain、target、dependency provenance | self-describing artifact manifest与compatibility validation |
| ASSET85-P1-025 | artifact路径按`kind/ResourceId.zasset`，不是build action/content identity | logical asset映射到immutable action result/content object |
| ASSET85-P1-026 | resident chunk有LRU，磁盘artifact/chunk无GC、quota、refcount/mark-sweep | project/cache quota、pin/lease、mark-sweep和safe prune receipt |
| ASSET85-P1-027 | chunk先于manifest发布，crash可留orphan；project journal主要追manifest | staged object namespace、atomic root publish和startup scavenger |
| ASSET85-P1-028 | 64 KiB物理切块不知道mesh page/mip/streaming section语义 | semantic bulk/page descriptors与independent residency |
| ASSET85-P1-029 | restore仍会解压/反序列化完整DTO | header/index-first access和按section materialization |
| ASSET85-P1-030 | 同一ResourceId没有平台/profile/capability variant resolver | `CookVariantKey`与runtime selection/fallback policy |
| ASSET85-P1-031 | build message/log与cacheability没有统一确定性规则 | structured action message；nondeterministic/error output禁止共享cache |
| ASSET85-P1-032 | metrics不能回答某资产“为何重建/为何命中/由谁产生” | action trace、input diff、cache tier、worker和publication receipt |

### 5.4 Cook与worker

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ASSET85-P1-033 | `AssetWorkerPool`只处理Texture/Mesh residency decode，与import/cook/build分离 | unified build execution service，保留现有bounded/single-flight内核 |
| ASSET85-P1-034 | worker key主要是string path，不含content generation、recipe、target、project owner | qualified immutable work key |
| ASSET85-P1-035 | cancel flag不能抢占正在执行的decode | cooperative checkpoints、deadline和可终止外部worker |
| ASSET85-P1-036 | VG在启用时由importer同步cook，无共享generation cache/scheduler | 独立VG build action；既有Runtime04 failure继续拥有父阻断 |
| ASSET85-P1-037 | Mesh SDF同样在import-time同步执行 | 独立target-aware SDF action和artifact |
| ASSET85-P1-038 | 没有remote worker/build farm、capability matching和result attestation | local/external/remote worker registry与verified result |
| ASSET85-P1-039 | CPU/GPU cook、import和runtime decode没有统一资源公平性 | project/session/domain配额、priority inheritance和admission |

### 5.5 Pack、install与产品交付

| ID | 当前差距 | 目标合同 |
|---|---|---|
| ASSET85-P1-040 | export pack由手写manifest读取raw source，不消费canonical registry/artifact/build graph | roots解析为qualified asset closure和cooked artifact set |
| ASSET85-P1-041 | writer持有整pack及全部input bytes，大项目内存随内容总量增长 | streaming writer、bounded buffers和external sort/index build |
| ASSET85-P1-042 | 一项asset等于一个chunk，缺bulk/page/alignment/layout policy | container block/page planner与streaming locality profile |
| ASSET85-P1-043 | pack没有target/profile/device capability variants | platform cook manifest与variant closure |
| ASSET85-P1-044 | 缺compression policy、encryption、signing和root-of-trust | authenticated container、key policy和verified mount |
| ASSET85-P1-045 | 缺mount priority、chunk group、localization、DLC/optional content规则 | named install/mount graph和entitlement-aware groups |
| ASSET85-P1-046 | delta流程仍以整pack驻留与重建为核心 | block-level streaming delta、resume、rollback和space preflight |
| ASSET85-P1-047 | Editor `CookAssets -> Pack`通过`assets.json`交接，没有build receipt/provenance | typed stage artifact、input/output digest和qualification receipt |
| ASSET85-P1-048 | 没有跨机reproducible、fault/scale及对参考引擎同负载资格 | hermetic build environment与持续determinism/performance suite |

## 6. P2治理与长期能力（12项）

| ID | 差距 | 收敛方向 |
|---|---|---|
| ASSET85-P2-001 | diagnostic大量依赖字符串 | 稳定code、source span、action/asset/provider identity |
| ASSET85-P2-002 | 多处generation使用整数递增但exhaustion策略不一致 | checked generation、retirement和typed exhausted outcome |
| ASSET85-P2-003 | size/time/status仍有字符串或0占位 | typed units、timestamps、outcome和unknown语义 |
| ASSET85-P2-004 | path case、Unicode、separator和mount identity缺全链canonical policy | VFS-owned canonical path identity及跨平台golden |
| ASSET85-P2-005 | TOML settings hash依赖当前序列化表现 | schema-aware canonical encoding和migration invariant |
| ASSET85-P2-006 | registry inspection缺snapshot分页及冲突解释 | generation-qualified paged catalog与resolution trace |
| ASSET85-P2-007 | artifact manifest用JSON/DTO演进成本高 | bounded versioned binary index与forward-compatible sections |
| ASSET85-P2-008 | 固定切块去重粒度单一 | 在测量后选择content-defined或semantic chunking |
| ASSET85-P2-009 | pack reader偏整包bytes API | async range/mmap/streaming reader及I/O cancellation |
| ASSET85-P2-010 | build telemetry缺cardinality/privacy规则 | bounded labels、redaction、sampling和retention |
| ASSET85-P2-011 | recipe/artifact/container跨版本rollout未定义 | dual-read/single-write migration和rollback window |
| ASSET85-P2-012 | 没有公开竞争性资产corpus和质量预算 | representative project corpus、golden、RSS/latency/throughput budgets |

## 7. 目标架构

```text
VFS / Watch
    -> AssetSourceAuthority
       -> immutable AssetSourceSnapshot + reverse dependency index
    -> AssetImportRecipeCatalog
       -> typed/versioned recipe + importer/provider generation
    -> AssetBuildGraph
       -> source/import/transform/cook/package actions
       -> AssetBuildScheduler
          -> local worker / isolated worker / remote worker
       -> DerivedDataService
          -> local/shared/remote content-addressed cache
       -> ArtifactRepository
          -> provenance manifest + semantic bulk pages + GC
    -> SubassetIdentityRegistry
       -> stable identity + redirect/tombstone/remap
    -> CookVariantResolver
       -> platform/profile/capability closure
    -> ContentPackCompiler
       -> signed container + block delta + install groups
    -> InstallAndMountService
       -> stage/verify/promote/mount/rollback receipt
```

核心不变量：

1. importer不能读取snapshot之外的文件；所有输入都可哈希、可审计、可反向失效。
2. action key由函数、recipe、全部输入、target和toolchain构成，逻辑Asset ID不冒充build identity。
3. build output不可变；publication只改变逻辑映射，失败不得破坏last-known-good generation。
4. subasset identity不依赖数组index或display label。
5. full scan、targeted import、watch、CI cook和Editor export使用同一build graph和事务。
6. runtime residency worker与offline build worker共享预算/诊断语义，但不混淆生命周期和artifact authority。
7. pack只能消费qualified cooked artifact closure，不得回退到任意raw source manifest。
8. “性能优于Unreal”只能由同硬件、同内容、同冷暖缓存、同正确性门的benchmark证明。

## 8. 重构里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 正确性封口 | 修复auxiliary source P0，建立snapshot、反向依赖和last-known-good | 三类辅助源增量/clean一致，watch/delete/rename/failure可证明 |
| M1 Import protocol | typed recipe、build context、bounded source view、provider lease、isolated envelope | builtin/native/plugin conformance同一套合同 |
| M2 Subasset identity | stable UID、lineage、redirect/tombstone/remap及引用修复 | reorder/rename/split/merge不静默错绑 |
| M3 Build graph | canonical DAG、scheduler、state、cancel/deadline、full/targeted统一 | 全量、单资产、watch、CI使用同一action executor |
| M4 DDC/artifact | cryptographic key、provenance manifest、hierarchical cache、GC/repair | cold/warm/shared cache及crash recovery资格通过 |
| M5 Cook/worker | VG/SDF/texture/mesh/shader独立target action，本地/隔离/远端worker | 预算、公平、取消、worker crash和result attestation通过 |
| M6 Semantic streaming | artifact header/index、mip/page/bulk section与按需materialize | 峰值RSS不再与最大源/完整artifact线性绑定 |
| M7 Package/install | canonical closure、streaming container、签名/加密、delta/install/mount | target package可验证、可恢复、可回滚 |
| M8 Product qualification | Editor/CLI/CI统一build receipt、跨机determinism、fault/soak/benchmark | 真实项目规模下达到发布门并与参考引擎同负载对比 |

M0必须先于性能重构；否则更快地复用错误artifact只会扩大内容损坏。M1-M4建立authority和cache identity后，M5-M7才能安全并行化。tooling最终迁移Rust不改变这些runtime数据合同，本轮因此不评价Python/CLI实现质量。

## 9. 资格门（48项）

### 9.1 Source与recipe（G01-G08）

| Gate | 必须证明 |
|---|---|
| G01 | 根文件不变、glTF external buffer变化会精确重建父资产 |
| G02 | external image和font blob变化同样进入父build key |
| G03 | delete/rename/permission failure有typed terminal outcome且不发布半代 |
| G04 | importer任何undeclared filesystem read都会fail closed或触发discovery restart |
| G05 | recipe canonical hash跨map顺序、平台路径表现和进程稳定 |
| G06 | recipe migration有golden、rollback和diagnostic |
| G07 | importer registry冲突、unload和in-flight lease可重复验证 |
| G08 | 恶意/超大source受input、memory、I/O、time、deadline和cancel预算约束 |

### 9.2 Subasset与publication（G09-G16）

| Gate | 必须证明 |
|---|---|
| G09 | glTF node/material/mesh/animation重排保持稳定subasset identity |
| G10 | rename、split、merge生成明确remap/tombstone而非随机UUID |
| G11 | dangling reference返回typed状态，禁止静默绑定到错误对象 |
| G12 | full与targeted对同一输入产生相同candidate generation |
| G13 | watch batch、rename和reconciliation不遗漏、不重复发布 |
| G14 | import失败保留last-known-good并记录失败generation |
| G15 | reference repair在validated transaction中原子提交或回滚 |
| G16 | crash发生在meta/artifact/registry/catalog任一阶段都可恢复到完整代 |

### 9.3 Build/DDC/artifact（G17-G28）

| Gate | 必须证明 |
|---|---|
| G17 | action key覆盖function、recipe、全部inputs、target、toolchain和engine ABI |
| G18 | 任一dependency内容变化只失效正确transitive closure |
| G19 | clean、incremental和shared-cache build产出相同artifact hash |
| G20 | 两台机器和不同目录root产生相同qualified build result |
| G21 | cache命中验证manifest、payload、producer和compatibility domain |
| G22 | poisoned/corrupt remote object被隔离，不能进入publication |
| G23 | chunk/manifest中途崩溃不留下不可回收的永久orphan |
| G24 | disk quota、pin、lease、GC并发下不删除live artifact |
| G25 | header/index读取无需解压完整payload |
| G26 | mip/page/bulk section能独立校验、驻留和驱逐 |
| G27 | 每次rebuild/cache miss都能输出稳定原因链 |
| G28 | nondeterministic/error action不会写入共享cache |

### 9.4 Scheduler、worker与cook（G29-G36）

| Gate | 必须证明 |
|---|---|
| G29 | project/session/domain配额阻止单资产饿死交互任务 |
| G30 | queue、waiter、completion、payload bytes始终有界 |
| G31 | cancel/deadline可中止长decode/cook并产生terminal receipt |
| G32 | worker panic/crash不会毒化scheduler或复用半结果 |
| G33 | local、isolated和remote worker结果hash一致 |
| G34 | VG/SDF是可缓存独立action，不在importer调用栈同步阻塞 |
| G35 | target/profile变化只重建受影响cook actions |
| G36 | worker capability mismatch和tool version drift显式拒绝 |

### 9.5 Pack、install与产品（G37-G48）

| Gate | 必须证明 |
|---|---|
| G37 | pack roots来自canonical asset closure而非raw source清单 |
| G38 | package writer峰值RSS受固定预算约束，不随pack总大小线性增长 |
| G39 | block/page layout满足texture/mesh/audio等streaming locality预算 |
| G40 | compression、encryption和signing policy进入container identity |
| G41 | tamper、wrong key、wrong signer和rollback attack在mount前拒绝 |
| G42 | DLC/optional/localization group安装、卸载和mount priority可证明 |
| G43 | delta支持断点续传、磁盘空间preflight、原子promotion和rollback |
| G44 | Editor、headless CI和发布build消费相同typed stage artifacts |
| G45 | build receipt可追溯source、recipe、worker、artifact、pack和签名 |
| G46 | 百万文件/深依赖/大subasset corpus通过fault和soak门 |
| G47 | cold/warm/incremental/shared-cache/package性能有固定预算与回归阈值 |
| G48 | 与Unreal/Godot/Bevy/Fyrox的对比使用同内容、同硬件、同正确性门和公开方法 |

## 10. 禁止的临时实现

- 禁止只把辅助文件mtime拼进现有key；必须使用canonical content dependency和反向owner图。
- 禁止让importer继续任意开filesystem path后靠watch补洞。
- 禁止恢复一个更大的全局mutex或用full rescan掩盖incremental graph缺失。
- 禁止以`ResourceId`、文件path或display label代替build/subasset identity。
- 禁止把`DefaultHasher`输出、序列化偶然顺序或本机绝对路径写入跨机cache key。
- 禁止在旧artifact路径旁再建一套互不相认的“新DDC cache”。
- 禁止把cook继续堆进每个格式importer的同步函数。
- 禁止把当前bounded worker改回无界queue，或以线程数替代memory/I/O预算。
- 禁止pack从canonical artifact缺失时静默回退读取raw source。
- 禁止以单元测试数量、接口名称或小fixture吞吐宣称达到/超过Unreal。
- 禁止把tooling迁移Rust当作runtime build authority已经成立。
- 禁止在M0正确性门关闭前接受任何只优化吞吐的改动。

## 11. 完成边界

本报告完成的是当前源码静态审查和重构需求登记，不是代码修复。只有M0-M8全部按资格门取得可复验回执，且Runtime04/51/64、Editor04/32/35等父owner完成相应依赖后，Runtime85才能把`implementation_status`改为`complete`。

本轮未修改Rust、Cargo、资源或工具实现，未运行Cargo、Editor、真实import/cook/package、网络DDC、签名、fault、soak或benchmark。用户已要求暂不考虑tooling优化；因此报告只规定runtime/product数据合同和Editor handoff，不评价未来将被Rust替换的工具实现。
