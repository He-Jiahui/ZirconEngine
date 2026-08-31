---
title: Runtime Asset Import、Source Discovery、Importer Recipe、Subasset、Derived Data、Cook、Package、Incremental Build、Worker、Determinism 与 Product Integration 当前工作树复审
category: zircon_runtime
report_id: Runtime207
review_date: 2026-08-31
baseline_head: working-tree
doc_type: current-working-tree-review-and-refactor-plan
review_status: current_working_tree_refresh_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
refreshes:
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/188-runtime-asset-resource-lifecycle-locator-registry-load-cache-import-cook-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/205-runtime-resource-lifecycle-load-ticket-cache-residency-generation-reload-cancellation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
related_code:
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/project/manager
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
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildDefinition.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildInputs.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildOutput.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildScheduler.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildSession.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildWorker.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataSerializationTest.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataCacheStoreHierarchyTest.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Tests/ChunkDependencyTests.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Cooker/CookPackageData.h
  - dev/UnrealEngine/Engine/Source/Developer/IoStoreUtilities/Private/IoStoreWriter.h
  - dev/godot/core/io/resource_importer.h
  - dev/godot/core/io/resource_importer.cpp
  - dev/godot/editor/file_system/editor_file_system.h
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/godot/tests/core/io/test_resource_uid.cpp
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/processor/process.rs
  - dev/bevy/crates/bevy_asset/src/processor/tests.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-resource/src/graph.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
---

# Runtime Asset Import、Source Discovery、Importer Recipe、Subasset、Derived Data、Cook、Package、Incremental Build、Worker、Determinism 与 Product Integration 当前工作树复审

## 1. 结论

当前资产链已经明显超过“按扩展名读文件后塞进 HashMap”的临时实现。可保留的底座包括：COW importer registry、full suffix/extension/plugin 索引、v7 sidecar meta、root/labeled subasset、依赖投影、candidate/targeted publication、durable journal/recovery、BLAKE3 内容校验、zstd artifact、64 KiB immutable chunk、resident chunk LRU、bounded/single-flight worker、确定性 pack 排序与去重、delta/install/staging/promotion receipt，以及 Editor `CookAssets -> Pack` 的阶段外形。

但是这些能力没有收敛为一个不可变、可重放、可审计的资产构建系统。当前主链仍是“同步 importer + 文件级 sidecar + ResourceId 命名 artifact + 手写导出 manifest”的拼接：source closure 没有统一 authority，recipe 没有 typed schema，`LibraryCacheKey` 没有进入完整 build identity，full/targeted 维护两套准备与提交流程，VG/SDF cook 仍从 importer 同步调用，pack 仍从用户 JSON 指定的 raw source 读取，而不是消费 canonical registry/artifact/build graph 的 qualified closure。

本轮不新增唯一 P0，既有 `ASSET85-P0-001` 继续作为唯一 source-closure P0 owner：`.bin`、字体和 glTF external image 等辅助字节未进入父资产 source digest/build key，full generation 也没有把 source-file snapshot 传给 importer；glTF/font importer 因而仍能绕过 snapshot 直接读物理路径，watch 也没有辅助源到父 action 的反向 owner。该问题会让 clean、incremental、restore 和发布包产生陈旧 artifact，必须先于吞吐优化关闭。

Runtime207 是 Runtime85 的当前工作树刷新，保留 `ASSET85-*` 编号，不重复 Runtime188/204/205/206 的 registry、I/O、resource lifecycle 和 catalog 父边界。当前稳定账目重判为 **P1 38 Open / 10 Partial / 0 Closed，P2 10 Open / 2 Partial / 0 Closed，资格门 35 Fail / 13 Partial / 0 Pass**。这不是达到或超过 Unreal 的性能证明；没有同内容、同机器、同冷暖缓存和同正确性门的 benchmark，不能作此结论。

## 2. 审查边界、证据等级与冻结快照

### 2.1 证据等级

| 等级 | 本轮使用方式 | 能证明什么 |
|---|---|---|
| E3 | 逐文件读取 importer/project/artifact/pipeline/watch/cook/pack 生产代码，沿 source、meta、restore、worker、pack 调用链追踪 | 当前合同、owner、同步边界、持久化字段和具体断链 |
| E2 | 检索 importer 注册/consumer、dependency、cache key、watch reverse mapping、cook、export manifest 与 stage handoff | 字段无 consumer、owner 分叉、产品链断路和重复 authority |
| E1 | 读取 asset tests、export tests 和 ignored test 定义，本轮未运行 | 测试意图、静态覆盖和仍需托管验证的边界 |
| E0 | 未运行 Cargo、Editor、真实大项目 import/cook/package、fault、soak、跨机 determinism 或 benchmark | 不得宣称动态通过、故障恢复或性能优于参考引擎 |

### 2.2 Zircon 冻结范围

生产范围排除 `tests`/`test` 目录和 test-named 文件，但保留生产文件中的 test attribute 计数。统计脚本按规范化相对路径、文件 SHA-256 排序串联后计算 fingerprint。

| 范围 | 文件 | 行 | 非空行 | bytes | test attrs | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Artifact | 57 | 12,666 | 11,581 | 443,551 | 17 | 2 | `14499471504e66327ee8defba6b55118cf56345d18d34a6d49ff2bf0a0c2054d` |
| Importer | 57 | 15,984 | 14,730 | 575,396 | 128 | 33 | `66925c4cc4aafc3d04c15953c64c5300194f1fb89301559ad13c831af29880c0` |
| Project | 49 | 10,222 | 9,335 | 366,061 | 39 | 7 | `e1c7e1dcacef1ecd647866f80695600cdd78012769192faa83a42dfc98cb5e77` |
| Pack | 17 | 2,351 | 2,115 | 76,751 | 4 | 1 | `1264affab2f55109cd3cf9dd2582f1aac45b6442a87fe1373b98f5409a322ac5` |
| Pipeline | 67 | 8,839 | 8,086 | 329,146 | 45 | 3 | `7d0812c0558bcc7fc24f8ac5c17c662fc1fbdfbcbcc4c901fd7b041f3006964f` |
| Watch | 19 | 983 | 904 | 32,520 | 3 | 1 | `c83e92d65a71100a03240b02a31da19cfcfc52385fbef7be54087848f4b9175e` |
| Virtual Geometry cook | 6 | 1,344 | 1,229 | 44,091 | 3 | 0 | `2f19032f03bda48e3a23a2705d6e5e3867220b93c72f0865dc60f3edf9e2ec2a` |
| Mesh SDF cook | 7 | 1,023 | 943 | 34,587 | 3 | 0 | `7b468949489e3c41104f5454676ebb5d0abf6bd94b151a44cf2688a34ac2a4d4` |
| Runtime207 core union | **279** | **53,412** | **48,923** | **1,902,103** | **242** | **47** | `db588199b2db2add0424620d9198f79029c45f0f476893484d128423d214ec41` |
| Runtime asset tests/test-named corpus | **247** | **51,246** | **46,795** | **1,783,997** | **1,034** | **59** | `3dabca6e54b3fc803f61847c9056f2c9601417986b554b0224efbaa599272777` |
| Runtime export-pack binary | **6** | **937** | **883** | **33,086** | **4** | **0** | `03cf4a3da994a6907f5152356176c9004b8816efd9ae7f3333b601df624a21af` |
| Editor export touchpoints | **52** | **13,205** | **12,012** | **455,921** | **132** | **16** | `9ab9cdd49c97c15c9223bb7506d354cac062d6553315cbe88d82e41cf7999c69` |

旧 Runtime85 的 222 文件统计不能直接与本轮 279 文件统计比较为“代码倒退”：当前范围新增了 artifact/render-manifest、pipeline worker completion、project generation 和 importer 分层。相反，范围扩大使“已有局部实现”与“是否有完整 authority”必须分开记录。

### 2.3 参考冻结范围

本轮冻结 Unreal DDC/Cooker/IoStore、Godot ResourceImporter/EditorFileSystem、Bevy AssetProcessor、Fyrox ResourceManager/loader/graph、Unity Graphics importer/reimport consumer 共 **25 个文件、19,004 行、16,465 非空行、731,723 bytes、40 个 Rust test attrs**，规范化 fingerprint 为 `71b62fb8a8e3db97cfea59dcd0dbf330c9505648e1e849450b7fdf7c34f946cc`。Unreal 用来定义工程级 build/DDC/cook/package 上限；Godot、Bevy、Fyrox 用来核对依赖、meta、异步、reload 和 identity 合同；Unity Graphics 只提供 importer consumer 旁证，本地 corpus 不包含完整 Unity AssetDatabase/DDC。

## 3. 当前链路与可保留底座

### 3.1 Source discovery、meta 与 candidate publication

`zircon_runtime/src/asset/project/manager/collect_files.rs:9-52` 递归发现文件，并排除 metadata、atomic transaction 路径和 auxiliary source；`sources.rs:18-34` 已有 `AssetImportSource`、compound membership 和 `AssetImportSourceSnapshot` 类型。compound source 会收集成员、排序 URI/path 并把成员 bytes 加入 import bytes（`sources.rs:208-260,307-349`），targeted generated source 还会传递 source-file snapshot（`targeted.rs:324-397`）。candidate generation、sidecar v7、durable journal 和 publication fence 是正确的工程方向。

断点在于这些能力没有成为所有 importer 的必经入口：single source 的 `included_files/included_paths` 为空；full generation 从 `source_bytes_for_import` 取得 bytes 后在 `full_generation.rs:162-166` 只构造 `AssetImportContext`，没有 `.with_source_file_snapshots(...)`。OBJ 的 `.mtl` 由 `source_plan.rs:123-192` 单独复制、快照、staged write 和 watch echo，这是真实进展，但不是一般 source dependency protocol。

### 3.2 Importer contract、registry 与识别

`importer/contract.rs:14-28` 的 descriptor 已包含 plugin、priority、extension/full suffix、output kind、version 和 capability；`contract.rs:118-171` 的 context 包含 root bytes、TOML settings、project resolver 和可选 source-file snapshots；`contract.rs:353-409` 的 outcome 能返回 root/labeled entries、logical dependencies、migration、diagnostics 和 reference repairs。`registry.rs:29-105` 采用 COW generation，并按 full suffix 再按 extension 选择；`registry.rs:127-166` 支持 descriptor/capability report 和 plugin remove。

这些字段仍不足以表达 `AssetImportRecipe`：没有 recipe schema/default/migration、target/profile/engine ABI/toolchain、declared input closure、deadline/cancel/budget、provider lease、output schema/provenance 和 streamed source view。识别没有 MIME/magic/sniff 结果或冲突解释，plugin unload 也没有 in-flight generation lease/quiescence receipt。`import_from_source.rs:34-87` 直接 `fs::read` 完整 bytes，并按 `.scene.toml`、`.model.toml`、`.zmaterial` 文件名特殊判断 project resolver；这不是可扩展的 build context。

### 3.3 External source closure 的具体断链

`importer/ingest/gltf_decode.rs:47-53,75-103,121-152` 从 `context.source_path.parent()` 推导物理目录，并让 gltf crate 从该目录直接读取 external buffer/image；`import_font_asset/mod.rs:15-33,96-132` 对 manifest 指向字体调用 `metadata`/`read`，之后才返回一个逻辑 dependency URI。逻辑 URI 不等于已经捕获的原始 bytes、digest、size、role 或 reverse owner。

因此“只改辅助文件”的操作可能不触发父 source/action：`collect_files.rs:64-75` 把 `.bin`、字体等当 auxiliary，watch 只映射文件自身 URI；没有 `auxiliary source -> parent action` 索引，也没有 undeclared read fail-closed。该事实保留为 `ASSET85-P0-001`，验收必须覆盖 glTF external buffer、external image、font blob 的 change/delete/rename/permission failure，以及 clean/incremental/restore 相同 build key。

### 3.4 Artifact、cache key 与 residency

`artifact/cache_key.rs:7-30` 的 `LibraryCacheKey` 只有 `source_hash`、`importer_version`、`config_hash`，fingerprint 仍使用 `DefaultHasher` 64-bit 输出，测试还把 legacy bytes 固定下来（`cache_key.rs:53-72`）。它不能表达 importer function、recipe、transitive source inputs、target/profile、engine ABI、toolchain 或 provider generation，也不能作为跨机 DDC identity。

`artifact/store.rs:31-42,79-126` 已有 `ZRARTM06`/schema 6、BLAKE3、zstd、64 KiB chunk、4 MiB manifest上限和 2 GiB raw payload 上限；`store.rs:137-181,453-605` 读取侧会校验 magic、kind、hash、chunk size、compressed size 和 zstd trailing bytes。可是 manifest 仍只记录 kind/revision/content hash/size/chunks（`store.rs:227-238`），没有 producer receipt、recipe/target/toolchain、dependency provenance、semantic bulk/page index、GC/quota/refcount 或 startup orphan scavenger。chunk 在 manifest 前发布（`store.rs:102-119`），旧 manifest 可保持可读，但 crash 后 orphan 的生命周期没有完整对象事务。

### 3.5 Full/targeted、cook 与 worker

targeted path 的 source snapshot、digest、mtime、descriptor、meta/context 与局部 IBL/dependency preparation 在 `targeted.rs:312-407` 是可复用底座；full path 仍在 `full_generation.rs:89-300` 逐 source 顺序读取、哈希、restore/import、校验和提交。两条路径没有共享 canonical `AssetBuildGraph`、action receipt 和统一 publication transaction。

`import_gltf.rs:35-37,166`、`import_model.rs:15-16`、`import_obj.rs:34-36` 直接从 importer context 取 `virtual_geometry_cook_request` 与 `mesh_sdf_cook_request`；VG/SDF 自身有设置、预算和确定性测试，但仍是 import-time synchronous substep，没有独立 target-aware action、cache key、scheduler node 或 result attestation。

`pipeline/worker_pool.rs:49-230` 已具备 IO TaskPool、unique queue depth、single-flight、bounded waiter/completion entry/bytes、TTL、panic 捕获、diagnostic 和 cancel terminal；`worker_pool/options.rs:5-32` 默认 queue depth 2、waiter 1,024、completion 64 entries/64 MiB。这些限制只覆盖 Texture/Mesh residency decode，request key 仍不是 qualified build action，cancel 会终止 ticket 但不保证正在运行的 decode/cook 在 checkpoint 停止。因此应吸收其 admission/diagnostic 内核，而不是把它误认成全资产 build scheduler。

### 3.6 Pack、install 与 Editor handoff

`pack/writer.rs:18-26,76-109` 的 `ZrPackInputAsset`/`ZrPackWriteReport` 仍以 `Vec<u8>` 表达输入和完整 pack；`write_files` 使用 64 KiB 读 buffer、按 path 排序和 source-changed 检测，但 `pack/assembler` 最终把所有 payload 累加到 `bytes`（`writer.rs:123-226`）。这改善了单文件读取，却没有把峰值 RSS 与 pack 总大小解耦，也没有 semantic block/page/layout、target variant、sign/encrypt/root-of-trust、mount group 或 localization/DLC policy。

`zircon_export_pack/run.rs:53-124,257-346` 读取 JSON manifest 的 raw source path，支持 deterministic double-run、delta 和 apply verification；`zircon_editor/.../wizard/plan.rs:249-288` 通过 `source_asset_manifest -> stages/cook_assets/assets.json -> --asset-manifest/--pack-file` 传递阶段。二者都是可保留的 stage 外形，但没有 typed build receipt、source/recipe/action/artifact/pack provenance；tooling 实现本身按用户要求不作为本轮优化对象。

## 4. P0 归属与当前 P1/P2 重判

### 4.1 唯一父 P0

`ASSET85-P0-001` 继续开放，Runtime207 不新增重复 P0。Runtime04/51/64、Runtime88/99w、Runtime188/204/205/206 继续拥有各自 generation、watch、I/O、registry、resource lifecycle 和 catalog 父边界；Editor04/32/35 继续拥有格式语义、preview、authoring 与 export workflow 父边界。Runtime207 只拥有 source-to-build graph、recipe、derived data、cook/package authority 及跨边界 receipt。

### 4.2 P1 工程化差距（48 项，当前状态）

| ID | 当前状态 | 当前工作树证据与需要重构的合同 |
|---|---|---|
| ASSET85-P1-001 | Open | outcome dependencies 仍是逻辑关系，不是可重放的 raw source graph；建立 typed edge（included file、logical asset、tool、environment）。 |
| ASSET85-P1-002 | Partial | targeted/generated/OBJ 有 snapshot，但 glTF/font/full path 仍可物理读；建立 snapshot resolver 和 declared read receipt。 |
| ASSET85-P1-003 | Open | settings 是 untyped TOML；引入 versioned typed `AssetImportRecipe`、defaults、migration 和 canonical encoding。 |
| ASSET85-P1-004 | Open | `LibraryCacheKey` 使用 `DefaultHasher` 且未进入主链；改为 cryptographic action key，覆盖 function/recipe/all inputs/target/toolchain/ABI。 |
| ASSET85-P1-005 | Open | context 缺 target/profile/ABI/compiler/tool version；建立 qualified build context。 |
| ASSET85-P1-006 | Open | handler 接收完整 `Vec<u8>`；改为 bounded streaming/random-access source view 和 backpressure。 |
| ASSET85-P1-007 | Open | importer 无 deadline/cooperative cancel/work-memory-I/O budget；操作必须产生 terminal receipt。 |
| ASSET85-P1-008 | Open | native envelope 仍是 JSON metadata/DTO + bytes；定义 bounded binary schema negotiation 和 streamed artifact channel。 |
| ASSET85-P1-009 | Open | plugin/native 无隔离、watchdog、quarantine、crash attribution；引入 provider health 与 isolated worker generation。 |
| ASSET85-P1-010 | Partial | registry 有 COW generation/remove，但无显式 generation lease、in-flight drain 和 unload receipt。 |
| ASSET85-P1-011 | Open | recognition 仍以 suffix/extension 为主；增加 MIME/magic/sniff 与冲突决议记录，suffix 仅作候选。 |
| ASSET85-P1-012 | Partial | descriptor/capability/DiagnosticOnly 存在，但没有 executable conformance、settings schema probe 和产品验证。 |
| ASSET85-P1-013 | Open | glTF index label 仍可能充当 subasset identity；引入 source UID/content lineage，label 只作 display。 |
| ASSET85-P1-014 | Open | exact label UUID 没有 redirect/tombstone/remap history；建立 versioned `SubassetIdentityRegistry`。 |
| ASSET85-P1-015 | Open | artifact/output 无 producer receipt、output schema、完整 provenance；输出必须自描述并可验证。 |
| ASSET85-P1-016 | Open | dependency 混合 source/artifact/runtime load 语义；分离 edge kind、strength、phase、target、owner。 |
| ASSET85-P1-017 | Open | restore 仍可能 materialize root artifact 才重建 dependency；发布独立可查询 dependency manifest。 |
| ASSET85-P1-018 | Partial | resource reload 已有局部 last-good/TTL/cancel，但 build action 没有统一 retry、negative cache、last-known-good 状态机。 |
| ASSET85-P1-019 | Open | full/targeted 并行维护复杂准备和提交；统一 graph executor，scope 仅改变输入集合。 |
| ASSET85-P1-020 | Partial | outcome 能携带 reference repairs，但 repair 尚未成为独立 validated mutation plan 和原子 publication。 |
| ASSET85-P1-021 | Open | 没有 canonical source/import/transform/cook/package DAG 和 action state；建立 `AssetBuildGraph`。 |
| ASSET85-P1-022 | Open | transitive dependency digest 不进入每个 action key；产生精确 invalidation explanation。 |
| ASSET85-P1-023 | Open | 只有 project-local artifact；建立 local/shared/remote hierarchical DDC 和 trust policy。 |
| ASSET85-P1-024 | Open | manifest 无 producer/recipe/toolchain/target/dependency provenance；加入 self-describing compatibility domain。 |
| ASSET85-P1-025 | Open | artifact path 仍为 `kind/ResourceId.zasset`；逻辑映射与 immutable action/content object 分离。 |
| ASSET85-P1-026 | Open | resident LRU 存在，磁盘 chunk 无 quota/refcount/mark-sweep/pin lease；建立 safe prune receipt。 |
| ASSET85-P1-027 | Open | chunk 早于 manifest 发布，journal 未覆盖完整 object set；建立 staged namespace/root publish/scavenger。 |
| ASSET85-P1-028 | Partial | 64 KiB physical chunk 已存在，但没有 mesh page/mip/streaming section descriptor；补 semantic bulk/page index。 |
| ASSET85-P1-029 | Open | restore 仍需完整 DTO decode；增加 header/index-first 和按 section materialization。 |
| ASSET85-P1-030 | Open | ResourceId 没有 platform/profile/capability variant resolver；定义 `CookVariantKey` 和 fallback policy。 |
| ASSET85-P1-031 | Open | build message/log 没有 cacheability/determinism rule；nondeterministic/error output 必须禁止共享 cache。 |
| ASSET85-P1-032 | Open | telemetry 无法回答为何重建、命中哪层 cache、谁产生 artifact；建立 action trace/input diff/receipt。 |
| ASSET85-P1-033 | Partial | AssetWorkerPool bounded/single-flight 内核真实存在，但只服务 residency decode，与 import/cook/build 分离；统一 execution service。 |
| ASSET85-P1-034 | Partial | worker 有 request/completion key，但不含 content generation/recipe/target/project owner；改成 qualified immutable work key。 |
| ASSET85-P1-035 | Partial | cancel 可终止 ticket/entry，但正在运行的 decode/cook 没有强制 checkpoint/deadline；补 cooperative cancellation receipt。 |
| ASSET85-P1-036 | Open | VG 仍在 glTF/model importer 同步 cook；拆为独立 cached VG build action。 |
| ASSET85-P1-037 | Open | Mesh SDF 同样 import-time synchronous；拆为 target-aware SDF action/artifact。 |
| ASSET85-P1-038 | Open | 没有 remote worker、capability matching、verified result attestation；先定义 provider protocol。 |
| ASSET85-P1-039 | Open | CPU/GPU cook、import、runtime decode 没有统一公平配额；建立 project/session/domain admission。 |
| ASSET85-P1-040 | Open | export pack 消费 hand-written raw manifest，不消费 canonical registry/artifact/graph；roots 必须解析 qualified closure。 |
| ASSET85-P1-041 | Partial | `write_files` 以 64 KiB buffer 读源，但最终 `ZrPackWriteReport.bytes` 保留整包；改为 streaming writer/index builder。 |
| ASSET85-P1-042 | Open | asset/chunk 粒度没有 bulk/page/alignment/layout policy；引入 container block planner。 |
| ASSET85-P1-043 | Open | pack 无 target/profile/device capability variants；pack manifest 必须包含 cook variant closure。 |
| ASSET85-P1-044 | Open | compression policy 存在局部 zstd，但无 encryption/signing/root-of-trust；纳入 container identity 与 mount validation。 |
| ASSET85-P1-045 | Open | mount priority、chunk group、localization、DLC/optional content 未定义；建立 install/mount graph。 |
| ASSET85-P1-046 | Open | delta 仍以完整 pack bytes 和重建为核心；改为 block-level streaming delta、resume、rollback、space preflight。 |
| ASSET85-P1-047 | Open | Editor CookAssets/Pack 只交 `assets.json`/paths；改为 typed stage artifact、input/output digest、qualification receipt。 |
| ASSET85-P1-048 | Open | 没有跨机 reproducible、fault/scale 与参考引擎同负载资格套件；建立公开 corpus 和持续门禁。 |

### 4.3 P2 治理与长期能力（12 项）

| ID | 当前状态 | 收敛方向 |
|---|---|---|
| ASSET85-P2-001 | Open | diagnostic 使用稳定 code、source span、action/asset/provider identity，不再以字符串作为协议。 |
| ASSET85-P2-002 | Open | generation 统一 checked exhaustion、retirement、typed exhausted outcome。 |
| ASSET85-P2-003 | Open | size/time/status 统一 typed units、timestamps、unknown 语义。 |
| ASSET85-P2-004 | Open | VFS 统一 path case、Unicode、separator、mount identity canonical policy。 |
| ASSET85-P2-005 | Open | TOML settings 改为 schema-aware canonical encoding 和 migration invariant。 |
| ASSET85-P2-006 | Open | registry inspection 增加 generation-qualified snapshot、分页和 conflict explanation。 |
| ASSET85-P2-007 | Partial | artifact manifest 已是 bounded binary bincode，但仍是 whole DTO，需 versioned index sections/forward compatibility。 |
| ASSET85-P2-008 | Open | 64 KiB 固定 chunk 只保留为基线，按测量选择 content-defined 或 semantic chunking。 |
| ASSET85-P2-009 | Partial | writer 已有 64 KiB file scan，但 reader/API 仍偏整包 bytes；增加 async range/mmap/streaming reader。 |
| ASSET85-P2-010 | Open | build telemetry 建立 bounded labels、redaction、sampling、retention 和 privacy policy。 |
| ASSET85-P2-011 | Open | recipe/artifact/container 跨版本 rollout 定义 dual-read/single-write、rollback window。 |
| ASSET85-P2-012 | Open | 发布 representative corpus、golden、RSS/latency/throughput 和 correctness budget。 |

## 5. 参考引擎对照

| 参考 | 可直接吸收的工程合同 | Zircon 当前差距 |
|---|---|---|
| Unreal DDC / Cooker / IoStore | `DerivedDataBuildDefinition.h` 将 function/constants/input builds/bulk/files/hashes 组成 immutable build definition；Inputs/Output 是可验证的 immutable records；Scheduler 管理资源、优先级、cache query/store；`ChunkDependencyTests.cpp` 覆盖父/环依赖，DDC serialization/hierarchy tests 覆盖 compact record、cancel 和层级 cache。 | Zircon 没有等价的 immutable definition/input/output record；`LibraryCacheKey` 只有三字段且是 `DefaultHasher`，没有 action DAG、transitive key、资源 scheduler 或独立 dependency manifest；pack 也没有 IoStore 式 block/index/mount identity。 |
| Bevy AssetProcessor | `meta.rs` 的 `ProcessedInfo` 记录 source/full hash 和 process dependencies；`processor/mod.rs` 明确 source+steps 决定最终输出，并用 write-ahead transaction log 恢复；`process.rs` 将 loader/transformer/saver 分成 typed async pipeline；processor tests 覆盖 registration、ambiguous loader、transaction log。 | Zircon dependency 主要在 import outcome，full path 没有 source snapshots；settings/recipe 无 typed schema；cook 不在 processor DAG；已有 journal 只保护 project publication，不保护完整 action object。 |
| Godot ResourceImporter / EditorFileSystem | importer 持有 recognized extensions、save extension、type、priority/order、version、options、threaded/import group；`.import` metadata 持久化 importer/UID/dependencies/feature remap/valid；EditorFileSystem 记录 import mtime/md5、dest paths 并组织 scan/reimport/dependency groups；UID tests 覆盖 invalid/roundtrip。 | Zircon descriptor 只有候选识别与 capability，没有 persisted importer receipt、feature/cook variant、dependency group 或 import validity state；watch/catalog 没有辅助源反向 owner，reimport 仍分裂 full/targeted。 |
| Fyrox ResourceManager | typed `ResourceLoader`/payload/future、async request state Pending/LoadError/Ok、watch-driven reload、dependency graph reflection；manager tests 覆盖 loader lookup、reload 和 graph。 | Zircon AssetWorkerPool 只解决 residency decode，不覆盖 generic import/load/build graph；`AssetImportContext` 同步 Vec bytes，依赖边不区分 source/artifact/runtime；reload 与 build output provenance 仍由不同 authority 管理。 |
| Unity Graphics | `ShaderGraphImporter.cs` 在 import 中声明 source/artifact/custom/export dependencies，并发布 primary/subassets；`AssetReimportUtils.cs` 提供批量 reimport 的 editing/progress/finally 边界。 | Zircon `AssetImportOutcome` 能携带 dependencies/subassets 但没有强制声明所有 filesystem reads，也没有 artifact/custom dependency identity；Editor stage 只交 JSON path，缺 typed receipt 和批量 action graph。 |

## 6. 目标架构与重构顺序

```text
VFS / Watch
    -> AssetSourceAuthority
       -> immutable AssetSourceSnapshot + reverse dependency index
    -> AssetImportRecipeCatalog
       -> typed/versioned recipe + importer/provider generation lease
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

1. importer 只能从 immutable snapshot resolver 读；所有输入有 canonical URI、role、size、digest 和 declared read receipt。
2. action key 由 function、recipe、全部 transitive inputs、target、toolchain、engine ABI 和 provider generation 组成，不能用 ResourceId/path/display label 代替。
3. build output 不可变；publication 只改变 logical mapping，失败保留 last-known-good generation 并记录 failure artifact。
4. subasset identity 不依赖数组 index 或 display label；reorder/rename/split/merge 必须产生 lineage/remap/tombstone。
5. full scan、targeted import、watch、CI cook 和 Editor export 使用同一 graph executor 与 publication transaction。
6. residency worker 与 offline build worker 共享预算/诊断语义，但不混淆 runtime lifetime 与 artifact authority。
7. pack 只能消费 qualified cooked artifact closure，缺 artifact 时 fail closed，不得回退 raw source。
8. 性能声明必须由同硬件、同内容、同 cache 温度、同正确性门的公开 benchmark 支撑。

### 6.1 里程碑

| Milestone | 必须重构 | 退出条件 |
|---|---|---|
| M0 正确性封口 | `ASSET85-P0-001`：辅助源 snapshot、digest、role、reverse owner、undeclared read policy、last-good | glTF buffer/image、font blob 只改辅助文件即可精确重建；delete/rename/permission failure 有 typed terminal result |
| M1 Import protocol | typed recipe/context、canonical settings、bounded source view、provider lease、isolated envelope | builtin/native/plugin conformance 使用同一 contract，跨机 recipe hash 稳定 |
| M2 Subasset identity | source UID、content lineage、redirect/tombstone/remap、reference repair plan | glTF reorder/rename/split/merge 不错绑，repair 可原子提交或回滚 |
| M3 Build graph | source/import/transform/cook/package DAG、action state、priority、cancel/deadline、full/targeted 统一 | full、单资产、watch、CI 共享 action executor 和 receipt |
| M4 DDC/artifact | cryptographic key、producer/provenance manifest、hierarchical cache、GC、orphan repair | cold/warm/shared cache、poisoned object、crash recovery 通过门禁 |
| M5 Cook/worker | VG/SDF/texture/mesh/shader 独立 target actions，本地/隔离/远程 worker | budget、fairness、cancel、crash、capability mismatch、result attestation 可复验 |
| M6 Semantic streaming | artifact header/index、mip/page/bulk sections、按 section materialize/evict | 峰值 RSS 不随完整 source/artifact 线性增长，resident 只读取需要的页 |
| M7 Package/install | canonical closure、streaming container、layout、sign/encrypt、delta、mount groups | target package 可验证、可恢复、可回滚，DLC/localization policy 有 receipt |
| M8 Product qualification | Editor/headless CI/发布 build 共享 typed stage artifacts，跨机 determinism、fault、soak、benchmark | 真实项目规模通过 correctness/performance gates，并以相同方法和参考引擎比较 |

M0 必须先于吞吐优化；否则更快地复用错误 artifact 只会扩大内容损坏。M1-M4 建立 authority 和 identity 后，M5-M7 才能并行。未来 tooling 改为 Rust 不改变这些 runtime contracts，因此本轮只记录 Editor handoff，不评价 CLI/Python 实现。

## 7. 资格门（48 项，当前状态）

| Gate | 状态 | 必须证明 |
|---|---|---|
| G01 | Fail | 根文件不变、glTF external buffer 变化会精确重建父资产。 |
| G02 | Fail | external image 和 font blob 变化进入父 build key。 |
| G03 | Partial | delete/rename/permission failure 已有部分 typed error，但尚未保证父 action 不发布半代。 |
| G04 | Fail | importer 的 undeclared filesystem read fail closed 或触发 discovery restart。 |
| G05 | Fail | recipe canonical hash 跨 map 顺序、路径表现和进程稳定。 |
| G06 | Fail | recipe migration 具备 golden、rollback 和 diagnostic。 |
| G07 | Partial | COW registry 可选 provider，但 in-flight generation lease/quiescence 未证明。 |
| G08 | Fail | source 受 input/memory/I/O/time/deadline/cancel 预算约束。 |
| G09 | Fail | glTF node/material/mesh/animation reorder 保持稳定 subasset identity。 |
| G10 | Fail | rename/split/merge 生成 remap/tombstone，不随机换 UUID。 |
| G11 | Fail | dangling reference 返回 typed 状态，禁止静默绑定错误对象。 |
| G12 | Fail | full 与 targeted 对同一输入产生相同 candidate generation。 |
| G13 | Partial | watch 有 batch/coalescing，但辅助源 owner、rename 和 reconciliation 仍不完整。 |
| G14 | Partial | 部分 reload path 有 last-good，但 build action 没有统一 failure generation。 |
| G15 | Partial | outcome 携带 reference repairs，但没有独立 validated transaction plan。 |
| G16 | Partial | durable journal 覆盖部分 publication，但 orphan object 和所有阶段恢复未证明。 |
| G17 | Fail | action key 覆盖 function、recipe、全部 inputs、target、toolchain、engine ABI。 |
| G18 | Fail | dependency 内容变化只失效正确的 transitive closure。 |
| G19 | Fail | clean/incremental/shared-cache 产出相同 artifact hash。 |
| G20 | Fail | 两台机器和不同 root 目录产生相同 qualified result。 |
| G21 | Partial | artifact manifest/payload/hash 有校验，但没有 producer/compatibility domain。 |
| G22 | Fail | poisoned/corrupt remote object 被隔离且不能 publication。 |
| G23 | Partial | manifest atomic write 存在，但 chunk orphan 的 crash scavenger 未闭合。 |
| G24 | Fail | disk quota/pin/lease/GC 并发下不删除 live artifact。 |
| G25 | Fail | header/index 读取无需解压完整 payload。 |
| G26 | Partial | 64 KiB chunk 可校验，但不能按 mip/page/bulk 语义独立驻留。 |
| G27 | Fail | 每次 rebuild/cache miss 有稳定原因链。 |
| G28 | Fail | nondeterministic/error action 不写入 shared cache。 |
| G29 | Fail | project/session/domain 配额阻止单资产饿死交互任务。 |
| G30 | Partial | AssetWorkerPool 的 queue/waiter/completion/payload bytes 有界，但没有覆盖 build actions。 |
| G31 | Partial | cancel 会终止 ticket，但长 decode/cook 没有完整 cooperative checkpoint。 |
| G32 | Partial | worker panic 被捕获，但 isolated/remote crash 与半结果隔离未证明。 |
| G33 | Fail | local/isolated/remote worker 结果 hash 一致。 |
| G34 | Fail | VG/SDF 是可缓存独立 action，不在 importer 调用栈同步阻塞。 |
| G35 | Fail | target/profile 变化只重建受影响 cook actions。 |
| G36 | Fail | worker capability mismatch/tool drift 显式拒绝。 |
| G37 | Fail | pack roots 来自 canonical asset closure，而不是 raw source 清单。 |
| G38 | Fail | package writer 峰值 RSS 受固定预算约束，不随 pack 总量线性增长。 |
| G39 | Fail | block/page layout 满足 texture/mesh/audio streaming locality。 |
| G40 | Fail | compression/encryption/signing policy 进入 container identity。 |
| G41 | Fail | tamper、wrong key、wrong signer、rollback attack 在 mount 前拒绝。 |
| G42 | Fail | DLC/optional/localization group 安装、卸载、mount priority 可证明。 |
| G43 | Partial | delta 已有 write/apply verification，但无 streaming resume、space preflight、原子 rollback。 |
| G44 | Fail | Editor、headless CI、发布 build 消费相同 typed stage artifacts。 |
| G45 | Fail | build receipt 可追溯 source、recipe、worker、artifact、pack、签名。 |
| G46 | Fail | 百万文件、深依赖、大 subasset corpus 通过 fault/soak。 |
| G47 | Fail | cold/warm/incremental/shared-cache/package 有固定性能预算和回归阈值。 |
| G48 | Fail | 与 Unreal/Godot/Bevy/Fyrox 的比较使用同内容、同硬件、同正确性门和公开方法。 |

## 8. 禁止的临时实现

- 禁止只把辅助文件 mtime 拼进现有 key；必须使用 canonical content dependency 和 reverse owner graph。
- 禁止 importer 继续任意打开 filesystem path，再靠 watch 事后补洞。
- 禁止用 full rescan 或全局 mutex 掩盖 incremental graph 缺失。
- 禁止用 ResourceId、文件 path、数组 index 或 display label 代替 action/subasset identity。
- 禁止把 `DefaultHasher`、序列化偶然顺序或本机绝对路径写入跨机 cache key。
- 禁止在旧 artifact 路径旁建立互不相认的“新 DDC cache”。
- 禁止继续把 VG/SDF/cook 堆进每个格式 importer 的同步函数。
- 禁止把 bounded worker 改回无界 queue，或以线程数代替 memory/I/O/CPU budget。
- 禁止 pack 在 canonical artifact 缺失时静默回退 raw source。
- 禁止以单元测试数量、接口名称、小 fixture 吞吐或一次 deterministic double-run 宣称达到/超过 Unreal。
- 禁止把 tooling Rust 迁移当作 runtime build authority 已成立。
- 禁止在 M0 正确性门关闭前接受只优化吞吐的改动。

## 9. 本轮完成边界与下一步

本报告完成的是当前工作树静态审查、稳定问题编号重判、参考引擎对照和分阶段重构计划，不是代码修复。后续实现必须从 M0 的 source snapshot/reverse owner/undeclared read RED 测试开始，然后才进入 recipe、graph、DDC、cook、semantic streaming 和 package/install。

本轮未修改 Rust、Cargo、资源或工具实现，未运行 Cargo、Editor、真实 import/cook/package、网络 DDC、签名、fault、soak、跨机验证或 benchmark。根据用户要求，tooling 优化暂不纳入；Editor 只记录与 runtime 的 typed stage handoff 缺口。只有资格门取得可复验回执，并且 Runtime04/51/64/88/99w/188/204/205/206 与 Editor04/32/35 父边界完成后，Runtime207 才能从 `pending` 进入实现收口。
