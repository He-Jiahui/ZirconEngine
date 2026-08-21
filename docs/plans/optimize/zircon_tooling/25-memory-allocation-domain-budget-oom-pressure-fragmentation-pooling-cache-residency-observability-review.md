---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/asset/import_flow/state.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/settings/change_log.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/icon_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_runtime/crates/zr_rhi/src/device.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs
  - zircon_runtime/src/asset/artifact/chunk_residency.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/mesh_sdf_cook/budget.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/core/framework/render/backend_types/graph_reports.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane.rs
  - zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/execution_budget.rs
  - zircon_runtime/src/graphics/runtime/render_framework/budget/degrade_ladder.rs
  - zircon_runtime/src/graphics/runtime/render_framework/budget/memory_budget.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/session/artifact.rs
  - zircon_runtime/src/scene/ecs/archetype/index.rs
  - zircon_runtime/src/scene/ecs/archetype/table/column.rs
  - zircon_runtime/src/scene/ecs/commands/inline_command_arena.rs
  - zircon_runtime/src/scene/world/staging_snapshot.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/memory.rs
  - zircon_runtime/src/text/cache/shaped_cache/memory.rs
  - zircon_runtime/src/text/parallel/completion_queue.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime_host/src/foreign_output/budget.rs
  - zircon_runtime_interface/src/serialization/text/canonical_writer.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
tests:
  - zircon_editor/src/tests/editor_message/bus/backpressure/fixture.rs
  - zircon_editor/src/tests/editor_message/bus/backpressure/performance.rs
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_allocation_contract.rs
  - zircon_plugins/sound/runtime/src/tests/kira_graph_sync.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/resource_lifecycle.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/lazy_residency.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/lifecycle.rs
  - zircon_runtime/src/core/runtime/tests/events/benchmark_evidence.rs
  - zircon_runtime/src/diagnostic_log/sink/tests/performance/rss/windows.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance/columnar.rs
  - zircon_runtime/src/text/font/fallback/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/asset_animation.rs
  - tools/tests/render-extract-baseline-report/metrics.Tests.ps1
  - tools/tests/resource-management-baseline-report.Tests.ps1
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
  - docs/plans/optimize/zircon_tooling/22-magic-constant-sentinel-threshold-timeout-capacity-budget-policy-convergence-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/UnrealMemory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/MemoryBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformMemory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/LowLevelMemTracker.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Containers/ContainerAllocationPolicies.h
  - dev/godot/core/os/memory.h
  - dev/godot/core/templates/paged_allocator.h
  - dev/bevy/crates/bevy_ecs/src/storage/blob_array.rs
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphObjectPool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Memory/BuddyAllocator.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceAllocators.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 25 · Memory Allocation Domain、Budget、OOM、Pressure、Fragmentation、Pooling、Cache Residency 与 Observability 审查

## 1. 结论

Zircon并不是没有内存工程基础。ECS archetype column使用连续类型擦除存储并能估算heap bytes；deferred command使用对齐的4 MiB inline arena预算；shaped-text cache按capacity、glyph、Arc header和索引slack保守估算entry bytes；asset chunk cache、Editor visual/icon cache、bounded keyed IO、text raster completion、foreign output和shader prewarm都有局部entry/byte admission；RHI/render graph报告transient与persistent资源；Animation pose、Sound graph、Editor message fanout和UI image cache还存在allocation-count或steady-state no-allocation测试。这些实现说明代码库已经具备可复用的budget、pool、census和hot-path证据原语。

但这些原语没有形成产品级Memory Control Plane。CPU heap、GPU allocation、mapped/staging、foreign allocation、plugin/native、thread-local、cache、queue、arena和process RSS之间没有统一MemoryDomain/Tag、resolved budget、pressure signal、reclamation priority、OOM policy或BuildSet-bound snapshot。产品不能回答“一帧/一次import/一个world/plugin/window实际分配了多少、峰值属于谁、哪些bytes只是cache owner统计、哪些Arc clone仍在外部存活、低内存时先回收谁、回收后能否满足下一次allocation、OOM会typed fail、degrade、quarantine还是abort”。

本轮对七个产品/代码family下Git追踪Rust代码做production-like扫描，排除明显tests/benches/examples/fixtures/generated/vendor/target和测试文件，并在首个纯`#[cfg(test)]`处截断。共覆盖11,716个文件、约1,032,404行前缀代码。由于父模块`cfg(test)`、宏展开、generic monomorphization、allocator内部和动态库分配不可由词法恢复，以下数字只用于定位，不是allocation次数或缺陷数：

| 词法信号 | occurrence / 文件 | 本轮解释 |
|---|---:|---|
| `Vec::with_capacity` / Hash capacity / String capacity | 491 / 339；59 / 47；30 / 29 | 预分配广泛，但requested与actual/slack/peak没有统一证据 |
| `Box::new` / `Arc::new` / `Rc::new` | 193 / 95；803 / 363；22 / 10 | owner/lifetime信号；不能由次数推导性能 |
| `collect::<Vec>` / `.to_vec()` / `.clone()` | 1,115 / 629；333 / 229；10,382 / 2,371 | 热路径复核入口；包含大量合法小值clone |
| `reserve` / `try_reserve` / shrink | 26 / 18；6 / 5；1 / 1 | 6处中多项是自定义byte budget，真实fallible heap reserve更少 |
| raw alloc API / `MaybeUninit` / `mem::forget` | 2 / 2；13 / 5；3 / 3 | 核心ECS与FFI/Native owner；由Tooling21继续拥有soundness |
| `OutOfMemory`信号 | 1 / 1 | 主要是I/O error映射，不能代表heap OOM recovery能力 |
| memory/budget词法 | 674 / 90 | 大量集中在Graphics、asset、text、Editor cache和测试尾部；不是全局budget |
| arena/pool/slab词法 | 290 / 47 | 包含真实pool，也包含task pool命名；已人工复核代表owner |
| `size_of` / `.capacity()` | 22 / 11；84 / 29 | 只有少数owner估算retained/slack bytes |

按family，Runtime占4,912文件、523,891行、82处Box、466处Arc、329处Vec capacity和446处memory-budget信号；Editor占4,709文件、321,463行、77处Box、190处Arc和176处budget信号；Plugins占1,470文件、120,209行，但真实allocator failure、native transfer和第三方内存仍跨Rust计数边界。仓库没有production `#[global_allocator]`、直接allocator选择配置或alloc-error hook；测试中存在tracking allocator，不等于产品具备归因和pressure能力。

本篇不重复Runtime04/05、Graphics、Runtime Interface05、Tooling07/15/21/22/23/24拥有的具体asset/GPU/ECS/FFI/性能/unsafe/policy/failure/concurrency P0。**没有新增P0，登记40项P1和12项P2**。本篇拥有跨crate CPU/host memory domain、allocator/OOM、预算协调、pool/cache真实驻留、pressure/reclamation、heap observability和产品资格合同。

## 2. 审查边界与Evidence

| Evidence | 本轮状态 |
|---|---|
| E1 tracked allocation inventory | 已完成；revision `ae2be3d865a937b9ed368bf965592045346c64e3`，branch `main` |
| E2 allocator/cache/pool/budget实现阅读 | 已覆盖ECS、commands、asset chunk、text、UI、render、plugin loader、foreign output与产品RSS工具代表路径 |
| E3 owner/lifetime/pressure语义 | 已确认cache-owner bytes与actual residency差异、arena high-water retention、UI node pool无界及OOM policy缺口 |
| E4 allocation trace/fault injection | 未执行；没有统一MemoryTag/heap snapshot或fallible allocator lane |
| E5 fragmentation/pressure/soak/product baseline | 未建立；当前source dirty且既有动态验证阻断未变化 |

本轮明确不做以下机械推导：

1. `clone`不自动等于heap allocation，`Arc::clone`通常只改计数；必须结合类型和热路径测量。
2. 系统allocator并非天然不合格；mimalloc/jemalloc等替换必须由平台workload数据决定。
3. pool/arena并非越多越好。没有bound/trim/pressure时，pool会把峰值转成长期resident memory。
4. Rust默认OOM abort不一定能安全恢复；关键是untrusted/optional/bulk allocation要在提交前fallible admission，fatal路径要有crash receipt。
5. RSS包含allocator retained pages、mapped files、DLL、stack和部分共享页，不能代替logical allocation tag；tag总和也不能代替RSS。

## 3. 必须保留的工程基础

### 3.1 ECS连续存储与heap estimate

`ArchetypeColumn`使用layout-aware连续body加ticks Vec，grow时检查乘法溢出，Drop逐项析构并释放原始allocation；archetype index向上聚合estimated heap bytes。它是合理的dense ECS基础，重构重点是budget、slack、OOM和trace，不是退回`Vec<Box<dyn Any>>`。

### 3.2 InlineCommandArena是正确的热路径方向

64 KiB对齐block、192 byte inline command、4 MiB owner budget、offset而非裸指针、整block append和checked alignment共同减少deferred command碎片与逐command allocation。需要补pressure/trim和actual high-water证据，不应删除arena。

### 3.3 局部byte budget已经超过简单entry cap

Bounded keyed IO在admit前同时检查entry与retained bytes，处理overflow、deadline、cancel和release reservation；text completion、Editor play output/settings change log、foreign output和asset chunk也有byte限制。它们应成为统一MemoryBudget/Admission的实现样板。

### 3.4 一些cache认真估算capacity与索引开销

Shaped cache不是只用字符串length估算，而是计算glyph Vec capacity、line capacity、font/language capacity、Arc header和八个hash index的保守slack。Visual asset cache同时限制entry与64 MiB pixels。这种definition-local estimator可保留，但要声明误差、external lease和归属domain。

### 3.5 已有产品RSS与hot-path allocation证据雏形

MVP/UI工具可读取Windows WorkingSet/PeakWorkingSet，Session Coordinator soak采集RSS增长；Animation pose、Sound graph和Editor message用test allocator统计allocation，RHI/UI报告reallocation counter。缺口是这些证据没有共同workload、tag、required lane和可信BuildSet绑定，而不是从零开始。

## 4. 已确认的具体内存真相缺口

### 4.1 ArtifactChunkResidency预算不是actual residency上限

cache以`Arc<[u8]>`返回命中值并在map中保存clone；eviction从map移除后就从`resident_bytes`扣除，但consumer持有的Arc仍可能继续存活。超预算的大chunk还直接`return Ok(bytes)`而不计入cache。因此`resident_bytes <= max_resident_bytes`只能证明cache-owned entries受限，不能证明artifact bytes实际resident受限。需要区分CacheOwnedBytes、ExternallyLeasedBytes、InFlightBytes与ActualObservedBytes。

### 4.2 InlineCommandArena reset保留峰值capacity

`reset()`调用`blocks.clear()`，Vec backing allocation保持capacity。4 MiB逻辑push budget限制单轮使用，但一次峰值后arena可以长期保留全部64 KiB blocks。对稳定帧复用这是正向优化；缺的是high-water、idle trim、pressure purge和“保留多少比重新分配更划算”的证据。

### 4.3 UiSurfaceNodePool没有entry/byte/age上限

surface node pool按component/control/path把detached node压入Vec，只有同key复用或整个surface销毁才释放；没有bucket/global cap、byte estimate、age/trim或pressure hook。动态template/path churn可把“避免allocation”变成无界retention。

### 4.4 ECS allocation failure直接进入process abort语义

`ArchetypeColumn::reserve`先让ticks Vec reserve，再raw alloc/realloc body；null进入`handle_alloc_error`，layout overflow用expect。此策略与Bevy dense table类似，可以在不可恢复core mutation上合理，但当前没有pre-admission world/entity/component memory budget、allocation tag、fatal receipt或平台low-memory策略，产品只能在最后一跳终止。

### 4.5 RenderMemoryBudget是固定参考阈值而非全局authority

其默认值是1080p-mid固定512/256/64/1024 MiB阈值，`warning_count`比较frame profile并驱动固定degrade ladder。它没有从adapter budget、system RAM/VRAM、产品quality、其他process/domain reservation和当前pressure解析；Tooling22拥有常量placement，本篇拥有“阈值不是可分配预算”的语义。

## 5. P1差距：Inventory、Domain 与 Budget Truth

### MEM-P1-001 · 没有canonical AllocationSiteInventory

container growth、raw alloc、Arc payload、cache entry、GPU/mapped/foreign allocation和third-party allocator不能从同一resolved SourceSet重建owner、hot-path、failure policy、lifetime与验证lane。

### MEM-P1-002 · 没有MemoryDomain/MemoryTag

Runtime world/ECS/asset/text/UI/render、Editor document/cache、plugin/native、network、tooling和foreign output没有稳定层级tag。RSS增长无法归因到产品能力、world、plugin generation或operation。

### MEM-P1-003 · 没有ProductMemoryBudget resolver

各owner使用固定bytes、entry count或无budget，产品启动时没有按system RAM/VRAM、platform class、quality、project和插件解析hard/soft/reserve/emergency budget及总和不变量。

### MEM-P1-004 · CPU/GPU/mapped/foreign taxonomy未统一

GPU resident、CPU shadow copy、staging、mapped buffer、asset compressed/decompressed、DLL allocation和OS working set可能重复或遗漏计数。没有ownership与accounting category就不能汇总。

### MEM-P1-005 · logical/accounted/resident/committed概念混用

`resident_bytes`有时指map-owned payload，有时指GPU texture，有时是estimated capacity；working set又是process物理观察。必须在schema中区分并禁止跨语义相加。

### MEM-P1-006 · Budget没有reservation/commit/release receipt

多数owner只在插入后更新counter或事后warning。没有MemoryReservationId、requested/granted/committed/released、generation和failure reason，跨任务取消与回滚无法核账。

### MEM-P1-007 · allocator选择没有BuildSet与平台证据

产品使用Rust/system allocator，未见production global allocator或可解析policy。这不要求立即更换allocator，但需要将选择、版本、配置、page/large allocation行为和benchmark绑定BuildSet。

### MEM-P1-008 · allocator slack/fragmentation没有budget

491处Vec capacity和大量pool/cache会保留slack；只有少数owner读取capacity。没有allocated vs used、size class、realloc、page retention与fragmentation指标。

## 6. P1差距：OOM、Fallible Admission 与 Security

### MEM-P1-009 · 没有统一OOM/failure policy

heap OOM、GPU OOM、budget reject、decode expansion和foreign allocation failure没有Fatal/Reject/Degrade/EvictRetry/Quarantine分类，也没有与Tooling23 FailureDomain闭环。

### MEM-P1-010 · fallible heap allocation只在少数边界出现

native plugin loader对manifest/source/output Vec使用`try_reserve`并typed映射，是正确基础；绝大多数bulk allocation仍依赖infallible container growth。应优先覆盖untrusted length、optional cache和large batch，而非机械替换所有小Vec。

### MEM-P1-011 · ECS growth没有pre-admission

world/entity/component批量创建无法在mutation前估算新增column/tick/index bytes并拒绝或分批；allocation失败发生在transaction内部的最后阶段。

### MEM-P1-012 · untrusted length与decompression expansion缺统一allocation gate

asset/plugin/network/serialization各有局部长度限制，但没有source length、decoded upper bound、element layout、temporary peak和output retention的统一ExpansionPlan。

### MEM-P1-013 · allocation overflow与OOM没有稳定错误身份

checked arithmetic有时返回typed error，有时expect/panic，有时映射成`io::ErrorKind::OutOfMemory`。跨FFI/product边界无法区分input-too-large、budget-exceeded、address-space、allocator OOM和integer overflow。

### MEM-P1-014 · 没有可控allocation failure injection

测试allocator主要计数，不按domain/size/nth allocation注入失败。transaction、cache publish、plugin load、world spawn和shutdown cleanup没有系统性failpoint矩阵。

### MEM-P1-015 · 大allocation安全阈值没有全局审计

Unreal提供large allocation checks；Zircon没有source-bound large allocation inventory与owner exemption。恶意或损坏输入可能在格式局部上限之外叠加temporary copies形成峰值。

## 7. P1差距：Pool、Arena、Cache 与 Residency

### MEM-P1-016 · Pool/Arena没有共同lifetime contract

Inline command、UI node、GPU transient、text、probe/shadow和plugin pools各自定义reset/reuse。缺Frame/Operation/World/Session/Process lifetime、owner generation、trim和destruction规则。

### MEM-P1-017 · InlineCommandArena没有pressure trim

`clear`保留高水位allocation，4 MiB逻辑budget不等于idle resident上限。应记录used/capacity/high-water/realloc并支持idle/pressure trim，而非每帧shrink。

### MEM-P1-018 · UiSurfaceNodePool无界

动态node identity持续变化时bucket和node payload可不断增长；report只有created/reused/recycled/discarded count，没有resident entries/bytes/age/eviction。

### MEM-P1-019 · Cache budget彼此独立

asset chunks、visual assets、icon atlas、shaped text、font fallback、RHI/UI image、shader/PSO和render transient pool无法在同一process pressure下协调让渡预算。

### MEM-P1-020 · Artifact external Arc lease未计费

cache eviction后consumer-held Arc仍占heap；没有lease census或inflight generation。命名为resident bytes会给qualification制造假上限。

### MEM-P1-021 · shared Arc payload容易重复计数或漏计

同一payload进入多个index/cache/consumer时，各owner按logical entry相加会double-count，按unique cache owner又会漏external lease。需要AllocationId或shared payload identity。

### MEM-P1-022 · Cache estimator没有误差/校准receipt

Shaped cache的保守估算方向正确，但allocator header、hash implementation、alignment和shared payload因平台变化。estimated、sampled和actual必须携带method/error bound。

### MEM-P1-023 · eviction只看局部LRU/entry，没有成本与重要性

编译产物、UI pixels、text、asset chunks和GPU资源的重建成本、latency、pinned/dirty状态不同。没有Priority、RebuildCost、LastUse、Lease、PinnedReason和fairness schema。

### MEM-P1-024 · 没有cross-cache purge/trim epoch

Editor关闭project、Runtime unload world、device recreate、plugin reload和系统low-memory时不能向所有cache广播同一generation pressure并收集released bytes receipt。

## 8. P1差距：Pressure、Recovery 与 Lifecycle

### MEM-P1-025 · 没有MemoryPressure service

源码未形成Low/Moderate/Critical/Emergency pressure vocabulary、platform callback、polling、debounce和hysteresis。各cache只能在自己的insert时被动淘汰。

### MEM-P1-026 · 没有分级reclamation ladder

应先清可重建cold cache，再降streaming/quality，再cancel optional work，最后拒绝新operation；当前render degrade、cache eviction和job rejection互不协调。

### MEM-P1-027 · eviction后没有retry/admission闭环

释放动作不返回actual released/committed delta，也不保证allocation retry只执行一次。容易出现反复evict、thundering herd或仍然OOM。

### MEM-P1-028 · dirty/pinned/inflight state没有统一保护

Editor unsaved document、正在上传GPU资源、plugin generation callback、asset transaction和foreign output不能按普通cache淘汰；缺PinLease、DirtyOwner和retirement fence。

### MEM-P1-029 · process singleton/thread-local保留未进入pressure

Tooling24记录OnceLock/thread-local生命周期；本篇要求其cache/allocator bytes进入MemoryDomain和purge策略，避免project close后process RSS不回落。

### MEM-P1-030 · plugin/native allocation无法按generation隔离

Rust host、C ABI与第三方库可能使用不同allocator。没有module generation tag、ownership allocator、unload census与late-free policy，Tooling21的安全证明也缺bytes/pressure视角。

### MEM-P1-031 · foreign allocation census缺全进程关联

Runtime allocation registry和host budget提供exactly-once release/encoded bytes基础，但不能与owner heap/GPU/cache tag合并，也不能解释provider内部temporary peak。

### MEM-P1-032 · shutdown只证明drop/join，不证明memory quiescence

线程停止后仍可能有Arc、OnceLock、pool capacity、mapped region或DLL allocation存活。Product shutdown receipt缺domain live bytes、pinned leases和expected process-retained baseline。

## 9. P1差距：Observability、Testing 与 Qualification

### MEM-P1-033 · 没有低开销allocation trace/tag stack

Unreal LLM按platform/default、asset和tag set追踪；Zircon只有局部counter/RSS。需要sampling/feature-gated tag，不应让shipping默认承担全量backtrace成本。

### MEM-P1-034 · 没有heap snapshot/diff artifact

无法比较打开project/world/plugin/window前后live allocations、retained pool、shared payload和callsite，也无法把泄漏/增长绑定source/build/workload。

### MEM-P1-035 · RSS工具没有child/shared/GPU闭环

现有Windows PeakWorkingSet是正向产品观察，但明确不含GPU且常不含child；allocator-retained page和共享页也无法归因。需要与tag、GPU budget、process tree和quiescence联合解释。

### MEM-P1-036 · allocation-free hot-path测试零散且非统一required gate

Animation pose、Sound、Editor message、UI cache有局部测试，但frame tick、ECS query/schedule、render extract/submit、input、audio callback、network poll和script VM没有同一NoAllocationScope/threshold schema。

### MEM-P1-037 · 没有fragmentation/long-soak证据

RSS growth soak不能区分true leak、pool high-water、allocator page retention和fragmentation。缺alternating-size churn、project reload、world churn、plugin reload和window/template churn矩阵。

### MEM-P1-038 · 平台memory class没有进入quality/admission

Unreal按platform memory bucket调整资源；Zircon固定参考预算没有low-memory desktop/handheld/server/editor class、address-space和page-size差异。

### MEM-P1-039 · 内存性能声明没有固定workload与统计

要超过Unreal，必须在相同world/assets/UI/plugin、warmup、allocator、平台和采样下比较used/allocated/RSS/VRAM、alloc rate、tail stall、fragmentation和load/close时间，不能只比较单个cache上限。

### MEM-P1-040 · Memory truth未进入产品资格与release

BuildSet/Capability/Release没有要求domain预算闭合、peak/quiescent snapshot、无失控pool/cache、OOM/pressure recovery、allocation-free hot lanes和soak currentness。

## 10. P2改进项

### MEM-P2-001 · 生成resolved AllocationSiteInventory

以AST/MIR/Cargo product graph识别container growth、raw/FFI/GPU/mapped allocation、pool/cache和bulk decode，允许definition-bound exemption并绑定source digest。

### MEM-P2-002 · 建立`MemoryScope`/`MemoryTag`层级

至少支持product -> subsystem -> world/project/plugin/operation -> resource type，并与TaskId、thread、generation、FailureDomain和trace关联。

### MEM-P2-003 · 建立ProductMemoryBudget resolver

按platform memory class、RAM/VRAM、quality、product role和plugin需求解析hard/soft/reserve/emergency预算，验证子域总和并输出receipt。

### MEM-P2-004 · 建立fallible bulk allocation/ExpansionPlan API

对untrusted或large allocation统一checked elements*stride、temporary/output peak、try reserve、budget ticket和typed error；小型内部容器保留标准API。

### MEM-P2-005 · 建立MemoryPressure Coordinator

汇聚OS/device/allocator/owner pressure，按可重建成本和pin状态执行evict/degrade/cancel/reject ladder，并返回actual released bytes与retry outcome。

### MEM-P2-006 · 为pool/arena建立high-water与trim policy

按frame/idle/operation/critical pressure决定reuse或trim；先覆盖InlineCommandArena、UiSurfaceNodePool、text和render temporary pools。

### MEM-P2-007 · 建立shared payload lease accounting

为Arc/GPU mirror/foreign buffer登记AllocationId、owner bytes、external leases、pinned generation和retirement，使cache-owned与actual-live不混淆。

### MEM-P2-008 · benchmark驱动allocator/platform配置

对system allocator与候选allocator在Editor、runtime、server、asset import、world churn上比较吞吐、tail stall、RSS和fragmentation，再决定BuildSet配置。

### MEM-P2-009 · 建立低开销Memory Trace与Snapshot service

支持sampling/tag-only/full callstack级别，输出heap/domain/pool/cache/foreign/GPU diff artifact，并控制tracker自身开销。

### MEM-P2-010 · 建立统一HotPathAllocationGate

允许0 allocation、bytes/frame或amortized阈值，明确warmup和fallback；覆盖frame、ECS、render、UI、audio、network、script和plugin callback。

### MEM-P2-011 · 建立OOM/pressure/failure injection suite

按tag、size、nth allocation和platform budget注入reject/OOM，验证transaction回滚、degrade、cache purge、plugin quarantine和crash receipt。

### MEM-P2-012 · 建立fragmentation/scalability qualification

在core count、RAM class、长时churn和多product topology下记录used/allocated/committed/RSS/VRAM、slack、page retention和quiescent recovery趋势。

## 11. 目标架构

```text
Resolved SourceSet / BuildSet / PlatformMemoryClass
  -> AllocationSiteInventory + MemoryDomain/Tag Registry
  -> ProductMemoryBudget (hard / soft / reserve / emergency)
  -> MemoryReservation / ExpansionPlan / AllocationScope
  -> Pool + Cache + SharedPayloadLease + Foreign/GPU ownership
  -> MemoryPressure Coordinator
  -> Reclaim / Degrade / Cancel / Reject / Fatal Decision
  -> MemorySnapshot + AllocationTrace + ShutdownMemoryReceipt
  -> ProductQualification / Release
```

工程约束：

1. MemoryTag是归因，不是让所有代码绕过Rust容器调用中央allocator；标准Vec/Arc仍可使用。
2. Budget必须在分配前admit，事后counter只能用于观测；estimated budget必须声明方法和误差。
3. Cache eviction不等于actual release；shared lease、allocator page retention和GPU fence必须分别证明。
4. Pressure处理必须有优先级、hysteresis和bounded retry，不能在每个owner内无限“清cache再试”。
5. Hot-path零分配只用于经过定义的steady state；初始化、场景切换和稀有错误路径采用不同预算。
6. allocator/pool/lock-free优化必须以同workload数据为准，不以参考引擎API形状作为性能证明。

## 12. 分阶段重构计划

### M0 · Inventory与产品基线

1. 生成cfg/product-aware allocation/domain inventory，采集Editor、Runtime Preview、Hub、PBR、WOC/server的process tree RSS与GPU/foreign现有指标。
2. 给ECS、asset、text/UI、render、plugin、Editor cache建立首批MemoryTag和actual-vs-estimated snapshot。
3. 固化chunk external Arc、inline arena high-water和UI node pool churn回归。

### M1 · Budget与fallible boundary

1. 定义MemoryDomain、PlatformMemoryClass、ProductMemoryBudget和reservation receipt。
2. 先将untrusted/bulk decode、plugin/asset import、world batch和large output接入ExpansionPlan/try reserve。
3. 保留core ECS fatal allocation策略，但在mutation前增加budget preflight和crash identity。

### M2 · Pool/cache真实驻留

1. 区分cache-owned、external lease、inflight和observed bytes。
2. 给InlineCommandArena与UiSurfaceNodePool增加high-water/trim/pressure，不每帧shrink。
3. 统一asset/text/UI/render cache的entry/byte/age/pin/rebuild-cost schema。

### M3 · Pressure与lifecycle

1. 建立OS/device/owner pressure输入和reclamation ladder。
2. project/world/plugin/window close输出released/retained/pinned receipt。
3. 把OnceLock/thread-local、foreign/native和GPU retirement接入generation/quiescence。

### M4 · Trace、failure injection与hot path

1. 建立tag sampling、heap snapshot/diff和tracker overhead基线。
2. 增加allocation failure、OOM、low-memory和eviction retry测试。
3. 将已有Animation/Sound/Editor/UI allocation tests收敛为统一required HotPathAllocationGate。

### M5 · Qualification与性能收敛

1. 跑RAM/platform、world/project churn、plugin reload、window/template churn和长时soak矩阵。
2. 比较allocator、pool trim、cache policy和data layout的tail latency、RSS、fragmentation与CPU成本。
3. MemoryEvidenceReceipt进入Capability Truth、MVP/ProductReceipt和release required lane。

## 13. 验收标准

| Gate | Required evidence |
|---|---|
| Inventory | 当前BuildSet所有large/raw/foreign/GPU/pool/cache/bulk allocation有owner、domain、lifetime和source digest |
| Budget | product hard/soft/reserve总和按platform解析；所有bulk admission有reservation/terminal receipt |
| Accounting | used/allocated/committed/resident/RSS/VRAM语义分离；shared payload与external lease不漏计/重复计 |
| OOM | untrusted/optional/bulk allocationtyped reject或degrade；fatal core OOM有crash identity且不发布半事务 |
| Pool/cache | entry/byte/age/high-water/trim/pin/rebuild cost可见；pressure后actual release可证明 |
| Lifecycle | project/world/plugin/window关闭后domain回到声明quiescent baseline，残留有owner和理由 |
| Hot paths | 定义的frame/ECS/render/UI/audio/network/script steady lanes满足allocation gate |
| Observability | memory snapshot/diff绑定BuildSet/workload/platform，tracker overhead有上限 |
| Stress | OOM、pressure、churn、reload、device loss和shutdown矩阵无泄漏、假成功或永久degraded |
| Performance | 同workload报告alloc rate、tail stall、used/allocated/RSS/VRAM、fragmentation与恢复时间 |

## 14. Reference engines带来的约束

| Reference | 可借鉴约束 | 不应照搬 |
|---|---|---|
| Unreal | FMemory统一allocator入口与allocation hint；PlatformMemory有memory bucket/stats；LLM区分platform/default并支持asset/tag；container slack可追踪 | 不复制C++宏和全局new模式，也不默认承担LLM完整开销 |
| Godot | Memory统一aligned/static alloc，PagedAllocator以page/free list复用固定对象 | 不把所有对象迁入全局page pool；先按生命周期与测量选择 |
| Bevy | BlobArray/Table把所有column capacity与entity Vec驱动capacity绑定，明确OOM/partial allocation abort不变量 | Bevy同样依赖fatal OOM，不能作为Zircon产品pressure/OOM完整答案 |
| Fyrox | generational contiguous Pool提供cache-friendly存储与stale handle防护 | pool本身不解决budget、fragmentation和pressure |
| Unity Graphics | RenderGraph临时对象集中release；BuddyAllocator有固定大allocation/TryAllocate/Dispose；Instance allocator显式Allocate/Free/Trim | 不移植C# NativeArray/Allocator枚举表面，借鉴scope、try allocate、trim和dispose证据 |

共同约束是：内存性能不只是“少分配”，而是分配身份、layout、capacity、lifetime、budget、failure、reclamation和可测证据形成闭环。Zircon已有多个高质量局部实现，工程化方向应把它们接到共同control plane，并用产品数据决定allocator与pool策略。

## 15. 本轮验证与限制

本轮仅新增review与索引准备，没有修改production、tests、manifest或workflow。没有运行Cargo、产品进程、heap profiler、allocator benchmark、OOM injection、GPU capture、stress或soak；已知Editor/Hub/WOC/plugin动态阻断未变化。

静态扫描无法观察monomorphized allocation、allocator metadata/page retention、第三方C/C++库、driver/GPU memory、mapped file、stack和shared pages，也无法证明具体clone分配。实施前必须按source drift重取AST/MIR/Cargo inventory，并用同一BuildSet上的allocation trace、heap/RSS/VRAM snapshot和fault injection完成E4/E5。
