---
title: Runtime memory allocation domain, budget, OOM, pressure, pooling, cache residency and observability current-working-tree review
date: 2026-08-31
status: review_only
scope: zircon_runtime, zircon_runtime_host, zircon_app, zircon_editor product consumers, RHI and current reference slices
canonical_parent: docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
---

# Runtime196 · Memory Allocation Domain、Budget、OOM、Pressure、Fragmentation、Pooling、Cache Residency 与 Observability 当前工作树复核

## 1. 结论

这不是一份新的 Tooling 内存优化计划。Tooling25 已经建立 `MEM-P1-001..040` 与 `MEM-P2-001..012` 的跨仓库唯一编号；本篇只在当前工作树重新核实这些编号在 Runtime 产品边界的状态，避免把同一缺口重复登记。用户明确暂不优化 tooling，因此本篇不扩展 `zircon_tooling` 的实现范围，也不把 Editor job admission 当成 Runtime 内存控制面。

当前树的真实进展比 Tooling25 的初始快照更完整，但仍是“多个局部预算器”而不是工程级 Memory Control Plane：

- RHI 已有 `GpuMemoryClass`、`GpuMemoryBudget`、`GpuMemorySnapshot`，WGPU 设备创建 buffer/texture 前做按类 admission，并在 `resources::allocate_zeroed_contents` 使用 `try_reserve_exact`。这只覆盖 RHI-owned backing 与 pending upload，不能代表 CPU heap、mapped memory、driver allocation、foreign/plugin allocation 或 process RSS。
- RenderGraph transient pool 已有 u64 字节估算、frame report、stale eviction、按 texture/buffer budget 的 budget eviction 和 device epoch；`BudgetDegradeLadder` 也能按连续帧超预算降低 render scale/mip/feature。这仍是 renderer-local policy，没有向其他 Runtime owners 发布 pressure 或统一 reclamation receipt。
- Artifact chunk residency 已能区分 cache-owned `resident_bytes`、caller-held external leases、tracked payload lower bound、eviction 与 disk-read 统计，并提供显式 trim。外部 lease tracker 有固定 metadata cap，溢出后计数明确表示诊断是下界；caller-held payload 仍不受全局产品预算约束。
- `UiSurfaceNodePool` 已有 256 bucket、每 bucket 4 节点、总计 1,024 节点上限和显式 trim；`InlineCommandArena` 已有 4 MiB 逻辑上限及 idle-only backing trim。两者没有统一 high-water、byte receipt 或 pressure coordinator。
- Dynamic session allocation registry 已按 session 统计 outstanding/high-water allocation bytes；foreign-output host 也有 encoded-byte/item/decode-time budget 和 lock-free counters。但二者不覆盖任意 Rust heap、第三方 DLL、GPU 映射、child process 或 allocator retained pages，且 allocation kind 没有 MemoryDomain/MemoryTag 层级。
- VM plugin memory policy 目前只是可选 soft/hard limit 的配置校验；GC budget 是 microsecond pause budget，不是 heap-byte enforcement。插件 native/VM 分配仍不能按 generation 与产品预算隔离。

因此本轮**不新增唯一 P0**，继承 Tooling25 的 40 个 P1 与 12 个 P2 作为唯一键。当前重判为：P1 `2 Closed / 21 Partial / 17 Open`；P2 `0 Closed / 4 Partial / 8 Open`；32 个资格门 `2 Pass / 13 Partial / 17 Fail`。Closed 仅适用于局部无界/峰值 retention 的两个具体缺口，不能推导出跨系统 pressure 已完成。所有数字来自当前工作树静态源码；未运行 Cargo、真实 GPU/OS、fault、fragmentation、long-soak 或产品 benchmark。

## 2. 审查边界与可复现指纹

### 2.1 Zircon 选择集

选择集包含当前工作树下以下 Rust production-like roots：

- `zircon_runtime/src`
- `zircon_runtime/crates/zr_rhi/src`
- `zircon_runtime/crates/zr_rhi_wgpu/src`
- `zircon_runtime_host/src`
- `zircon_app/src`
- `zircon_editor/src`

排除路径段 `tests`、`test`、`benches`、`bench`、`examples`、`fixtures`、`generated`、`vendor`、`target` 及 `test.rs`/`*_test.rs` 文件。选择集按相对路径小写排序，逐文件 SHA-256 后以 `path|hash` LF 连接再 SHA-256：

| 选择集 | files | lines | bytes | test/ignored markers | fingerprint |
|---|---:|---:|---:|---:|---|
| Runtime/RHI/Host/App/Editor production-like Rust | 12,190 | 1,657,818 | 58,585,249 | not counted / 0 | `0f4f0e0d70404d006f49faa92446347a55bbc944a4e88001774cd5849a663172` |

该指纹是当前工作树快照，不是 Git HEAD 指纹。报告只使用源码中可定位的 contract、生产调用点和相邻测试作为 evidence；词法出现次数不被解释为分配次数或性能结果。

### 2.2 重点 owner 与产品消费者

| owner | 当前证据 | 仍缺的控制面责任 |
|---|---|---|
| RHI/WGPU | `zircon_runtime/crates/zr_rhi/src/memory.rs:9-129`、`zr_rhi_wgpu/src/device/resources.rs:13-47` | allocator/driver/mapped/GPU 统一账本、platform profile、reclamation callback、commit receipt |
| RenderGraph | `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs:20-25,392-454` | 与 CPU/asset/text/UI cache 的统一 pressure、importance/pinning、retry 语义 |
| Asset chunks | `zircon_runtime/src/asset/artifact/chunk_residency.rs:115-147,308-355,418-444` | external lease 的全局收费、allocator slack、decompression/decoded residency、跨 cache trim epoch |
| ECS/Commands | `zircon_runtime/src/scene/ecs/commands/inline_command_arena.rs:8-12,197-215` | allocation receipt、worker/World pressure subscription、ECS growth pre-admission |
| UI surface | `zircon_runtime/src/ui/surface/node_pool.rs:13-17,104-145,172-196` | bytes/high-water、cross-surface owner、统一 trim priority |
| Dynamic/ABI | `zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs:11-41,64-128,131-185` | process-wide census、domain/tag、generation/plugin/native/foreign correlation |
| Host foreign output | `zircon_runtime_host/src/foreign_output/budget.rs:11-92`、`state.rs:24-329` | output byte budget 与所有 runtime allocations 的合并、child/shared/GPU correlation |
| VM/plugin | `zircon_runtime/src/script/vm/plugin/management_policy/memory.rs:5-39`、`gc_bridge/budget.rs:5-130` | hard limit enforcement、allocator hooks、generation retirement、OOM/quarantine action |
| App/Editor | Runtime session/product composition、Editor cache/job consumers | 产品级 budget resolver、memory health/pressure UI、qualification/release gate |

## 3. 必须保留的局部工程基础

以下能力是真实可复用底座，不应在重构中被抹平为一个万能 allocator API：

1. RHI 的 `GpuMemoryClass` 维度和 `GpuMemorySnapshot` 的 active/retired/pending 分离，为 GPU physical backing 建立了正确方向；后续应把它接入统一 domain registry，而不是再次造一套 texture-only budget。
2. RenderGraph 对 transient texture/buffer 使用 requested byte size、stale age 和 budget eviction；Unity RenderGraphResourcePool 同时记录 frame allocation、release、stale lifetime 和 exception cleanup，Zircon 应吸收“资源释放责任可验证”的部分。
3. Artifact chunk 的 cache-owned 与 external lease 分离，并在 tracker 溢出时显式报告 lower bound；这比只报告 LRU entry count 更诚实，但必须进入统一 shared-payload accounting。
4. Inline command arena 的 idle-only trim 保持 normal frame reuse，不把每帧 reset 变成 allocator churn；UI pool 的 bucket/total bounds 同样是可接受的局部防护。
5. Dynamic allocation registry 的 session ownership、release validation、outstanding/high-water counters 是 ABI ownership 的起点；它不能冒充全进程 allocator census。
6. Text cache、completion queue、asset import、foreign output 等局部 byte admission 说明工程中已经有 `try_reserve`、entry estimator、rolling diagnostics 等原语，应统一 receipt/authority，而不是继续复制常量。

## 4. 当前状态重判规则

- **Closed**：具体原始命题已由生产代码和 focused regression 覆盖，不再作为同一命题的未完成项；仍可能被更高层父项继承。
- **Partial**：有真实局部 owner、字节/计数边界或 trim，但缺少跨域 authority、失败语义、完整 residency 或产品消费。
- **Open**：当前没有足够生产 contract，或仅有测试/配置/诊断字段，不能支撑工程级保证。

## 5. P1 差距逐项重判

### 5.1 Inventory、Domain 与 Budget Truth

| ID | 当前状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MEM-P1-001 | Partial | session allocation registry 有 allocation census；RHI、asset、text、UI 各自报告 | Runtime-owned `AllocationSiteInventory`，生成 stable site id、owner、domain、build/profile provenance，并能和 product snapshot 对齐 |
| MEM-P1-002 | Open | `git grep` 对 Runtime/App/Editor 无 `MemoryDomain`/`MemoryTag`；仅有 `GpuMemoryClass` | 分层 `MemoryDomain/MemoryTag`（CPU heap、GPU local、mapped/staging、asset、scene、text、UI、plugin、foreign、process）并禁止字符串自由命名 |
| MEM-P1-003 | Partial | `GpuMemoryBudget` 只在 RHI/render path；asset chunk/UI/text/foreign 各有独立上限 | 单一 `ProductMemoryBudget` resolver，输入 platform/device/profile/product mode，输出各 owner hard/soft/reserve budgets |
| MEM-P1-004 | Partial | RHI class、foreign output bytes、asset resident/external、session bytes 可分别读取 | 统一 logical/accounted/resident/committed/mapped/driver/foreign taxonomy，明确 overlap 与 owner ledger |
| MEM-P1-005 | Partial | RHI active/retired/pending、asset cache/external、text resident 等概念已出现，但名称语义不一致 | 每个 receipt 同时声明 requested/accounted/resident/committed/released，禁止把 cache-owner bytes 当 RSS |
| MEM-P1-006 | Open | 当前没有 `AllocationReceipt`/reservation commit release contract；RHI 只做同步 admission | 可移动/可取消 reservation receipt，commit/rollback/release 必须幂等并绑定 owner/generation |
| MEM-P1-007 | Open | 未发现 allocator BuildSet/platform selection evidence；RHI 预算是固定 reference 1080p mid | 平台 allocator matrix、build feature、alignment/quantization、fallback 和 workload evidence 纳入 build artifact |
| MEM-P1-008 | Open | `Fragmentation` 无 Runtime 命中；只有 pool capacity/retained bytes | allocator slack、page/span fragmentation、GPU heap holes、arena peak-to-live 形成可比较指标和预算 |

### 5.2 OOM、Fallible Admission 与安全边界

| ID | 当前状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MEM-P1-009 | Partial | RHI 有 `MemoryBudgetExceeded/ResourceAllocationFailed`；artifact/I/O、ECS、plugin、text 使用不同 error | 统一 OOM disposition：retry-after-reclaim、degrade、reject optional、quarantine plugin、fatal crash receipt；保留 domain error source |
| MEM-P1-010 | Partial | WGPU `try_reserve_exact`、text completion `try_reserve`、少数 plugin/asset边界；大量 `Vec::with_capacity/Box::new/Arc::new` 无 admission | bulk/untrusted/optional 分配前统一 fallible API，并对 infallible allocation 建 lint/allowlist |
| MEM-P1-011 | Open | ECS table/column 连续增长和 command vectors 没有 MemoryDomain admission；arena 只限制自身 4 MiB | ECS `ExpansionPlan` 先估算所有列/索引/rollback bytes，再以 receipt 原子提交，失败不得进入 process abort-only 路径 |
| MEM-P1-012 | Partial | artifact raw payload/decompression limit、manifest limit、foreign JSON limit 已存在 | 所有 untrusted length、zstd/bincode expansion、network/plugin payload 共享 checked arithmetic + admission gate + typed limit identity |
| MEM-P1-013 | Partial | `RhiError` 和若干 `io::ErrorKind::OutOfMemory` 可定位，但跨层仍压成字符串/通用 error | 稳定 machine-readable allocation failure identity（domain/site/requested/limit/recoverability），ABI/App/Editor 保留字段 |
| MEM-P1-014 | Open | 未发现可控制 heap allocation failure injection；只有局部 budget tests | allocator shim/test service 支持 site/domain/count/bytes/failure phase 注入，并验证 rollback、retry、shutdown |
| MEM-P1-015 | Partial | raw payload、foreign output、RHI texture/buffer、UI node、inline arena 有局部上限 | 生成式 large-allocation audit 覆盖所有 `with_capacity`, `resize`, `collect`, decode、GPU upload、native bridge，并绑定 owner disposition |

### 5.3 Pool、Arena、Cache 与 Residency

| ID | 当前状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MEM-P1-016 | Open | pool/arena 类型各自管理 clear/retain/trim，未共享 lifetime trait/receipt | `MemoryRetainer`/`ReclaimableOwner` 合同：live/pinned/inflight、high-water、last-use、trim result、drop epoch |
| MEM-P1-017 | Closed | `InlineCommandArena::trim_idle_storage` 仅空 arena 释放 backing；World/worker facade 和 regression 已存在 | 保留该局部实现；后续只需接入统一 pressure coordinator，不再把“无 trim”作为独立 finding |
| MEM-P1-018 | Closed | `UiSurfaceNodePool` 有 256 bucket、4 per bucket、1,024 total bound、capacity rejection 和 `trim_retained_node_pool` | 保留局部 bound；后续补 byte accounting/priority，不再声称 pool 无界 |
| MEM-P1-019 | Partial | RHI/render graph、asset chunk、text rich/shaped/layout、UI node 各有独立 byte/entry limits | central budget resolver + cross-cache arbitration，缓存不能只在本地超限时互相挤压 |
| MEM-P1-020 | Partial | asset chunk trim 会把 cache-owned bytes 归零并追踪 caller-held `Arc` leases；tracker cap 溢出时报告 lower bound | external lease 必须有 owner/generation receipt、hard/soft charge 和 eviction blocked reason；下界不能进入 hard budget truth |
| MEM-P1-021 | Partial | asset payload identity 用 `Arc::as_ptr` 去重；其他 Arc caches 没有共享 identity ledger | shared payload registry 以 allocation identity/owner shares 计算一次 physical bytes、多个 logical leases，避免 double/under-count |
| MEM-P1-022 | Open | text/cache/asset estimators 使用 `size_of`, glyph/entry lengths 和 index slack，但无 actual allocator calibration receipt | estimator 输出 estimate/error bound/source revision，并在 debug/profile workload 与实际 allocator size 对账 |
| MEM-P1-023 | Partial | asset/transient pools 以 LRU/last-used/byte cost eviction；没有跨 owner importance/pinned/streaming priority | eviction candidate contract 必须同时有 cost、importance、age、pinned/inflight、recreate latency 和 owner veto |
| MEM-P1-024 | Open | trim API 分散在 arena/node/artifact，未发现 cross-cache `trim_epoch` 或统一 completion receipt | pressure epoch 广播、ordered reclamation、每 owner acknowledged/released bytes 和 deadline |

### 5.4 Pressure、Recovery 与 Lifecycle

| ID | 当前状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MEM-P1-025 | Open | `MemoryPressure` 在 Runtime/App/Editor 无命中；render degrade 只读取 frame profile | Runtime MemoryPressure service，采样 OS/device/allocator/product ledger，发布 severity、reason、deadline、epoch |
| MEM-P1-026 | Partial | Render `BudgetDegradeLadder` 有固定 7 步；asset/text/transient 各自有 eviction/trim | 分级 ladder 由 coordinator 排序：drop scratch -> trim caches -> evict recreateable GPU -> degrade render -> reject optional -> fatal |
| MEM-P1-027 | Partial | asset/transient 插入前会 eviction，RHI create 失败直接返回；没有统一“回收后再 admission” receipt | `admit -> reclaim -> retry once/typed fail` 闭环，记录 reclaim candidates、released bytes、retry outcome |
| MEM-P1-028 | Partial | RHI pending/retired 与 asset external leases 有局部保护；缺 shared dirty/pinned/inflight state | 所有 reclaimable owner 声明 dirty/pinned/inflight/readback/fence/lease，coordinator 不得回收未满足保护的资源 |
| MEM-P1-029 | Open | singleton/thread-local/worker arenas、allocator page retention 没有 pressure callback；仅少数显式 trim | process-lifetime owners 注册 maintenance hook，shutdown/pressure 能证明 released vs allocator-retained |
| MEM-P1-030 | Partial | VM plugin policy 有 soft/hard config；dynamic registry 有 session kind；无 allocation generation/domain | native/VM allocation receipt 绑定 plugin generation、module epoch、quota、quarantine/revoke 和 unload barrier |
| MEM-P1-031 | Partial | session census 与 foreign-output per-kind counters 可读；没有 child/shared/GPU/third-party process correlation | process-wide census 合并 RuntimeSession、foreign/native、GPU snapshot、child process、shared mapping，并标注 overlap |
| MEM-P1-032 | Partial | session close 会阻止 outstanding runtime allocations；RHI device epoch/retired tracking 存在；没有 memory-quiescence receipt | shutdown barrier 必须等待 allocation leases、GPU fences、worker arenas、foreign buffers、allocator callbacks 全部 terminal，并报告残留 owner |

### 5.5 Observability、测试与资格

| ID | 当前状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MEM-P1-033 | Open | 仅有局部 counters/diagnostic lines；无统一 low-overhead tag/stack allocation trace | sampling/flight-recorder trace，site/domain/tag/stack hash、alloc/free/resize、pressure epoch，release 可关闭 |
| MEM-P1-034 | Open | 只有 cache/RHI/session snapshots；无 heap snapshot/diff artifact schema | owned snapshot artifact：logical/accounted/resident/committed、leases、pools、slack、RSS/GPU correlation 与 diff |
| MEM-P1-035 | Open | Windows RSS 诊断与 foreign metrics 分散；无 child/shared/GPU closure | platform collector 统一 process/child/shared/mapped/GPU/allocator-retained 字段，并声明不可相加项 |
| MEM-P1-036 | Partial | RHI resource lifecycle、arena trim、node pool、text/asset budget 有 focused tests；不是统一 required gate | `HotPathAllocationGate` 以 fixed workload、steady-state allocation count/bytes、failure path 和 product owner list 强制执行 |
| MEM-P1-037 | Open | 未发现 fragmentation 或 long-soak qualification artifact；局部 benchmark 多为 ignored/release-only | allocator/pool/GPU/asset/text/scene workload 做 30-60 分钟 soak、fragmentation、peak-to-live、reclaim latency 与 p95 |
| MEM-P1-038 | Partial | `GpuMemoryClass` 和 fixed reference profile 存在；没有 platform memory class 进入 admission/quality | platform tier/mobile/console/desktop/UMA/discrete profile 进入 resolver、test matrix、release gate |
| MEM-P1-039 | Open | 代码有常量和单元测试，没有固定产品 workload、sample count、variance、p50/p95/p99 memory report | reproducible workload manifest、warmup/sample/statistics、peak/resident/commit/reclaim/CPU/GPU result artifact |
| MEM-P1-040 | Open | 当前没有 Runtime/App/Editor product qualification consuming a memory truth artifact | release qualification 必须阻止 budget/soak/fragmentation/oom/shutdown evidence 缺失的 product build |

## 6. P2 改进项重判

| ID | 当前状态 | 复核结论 |
|---|---|---|
| MEM-P2-001 | Open | 需要从 build/source inventory 生成 resolved AllocationSiteInventory；当前只能人工 grep |
| MEM-P2-002 | Open | `MemoryScope/MemoryTag` 尚不存在，`GpuMemoryClass` 不能替代全域 tag hierarchy |
| MEM-P2-003 | Open | ProductMemoryBudget resolver 尚不存在；RHI reference profile 不是产品 resolver |
| MEM-P2-004 | Partial | WGPU/RHI、text completion、少数 asset/plugin 路径已有 fallible reserve；无统一 bulk `ExpansionPlan` |
| MEM-P2-005 | Open | 局部 trim/degrade 存在，MemoryPressure Coordinator 不存在 |
| MEM-P2-006 | Partial | inline arena、UI node pool、artifact chunk 已有显式 trim；缺 high-water policy、bytes、priority 和 acknowledgement |
| MEM-P2-007 | Partial | artifact external lease tracker 已有 identity/overflow 统计；不是 shared payload lease accounting service |
| MEM-P2-008 | Open | allocator/platform 选择没有 benchmark-driven BuildSet evidence |
| MEM-P2-009 | Open | 没有低开销 Memory Trace/Snapshot service 或稳定 artifact schema |
| MEM-P2-010 | Partial | focused allocation-free tests 和 RHI fallible paths 存在；没有统一 required gate |
| MEM-P2-011 | Open | 没有 OOM/pressure/failure injection suite；局部 budget tests 不能模拟 allocator failure |
| MEM-P2-012 | Open | 没有 fragmentation/scalability qualification 与 long-soak evidence |

## 7. 资格门

| Gate | 判定 | 证据/缺口 |
|---|---|---|
| G1 canonical allocation inventory | Fail | 无 `AllocationSiteInventory` 生成物 |
| G2 domain/tag registry | Fail | 无 `MemoryDomain`/`MemoryTag` |
| G3 product budget resolver | Fail | 只有 RHI/local budgets |
| G4 CPU/GPU/mapped/foreign taxonomy | Partial | RHI/foreign/asset local classes exist, no unified ledger |
| G5 reservation/commit/release receipts | Fail | no common receipt |
| G6 allocator BuildSet/platform evidence | Fail | fixed constants, no workload artifact |
| G7 fallible bulk admission | Partial | several `try_reserve` boundaries, broad infallible heap remains |
| G8 ECS growth pre-admission | Fail | no typed expansion receipt |
| G9 untrusted/decompression expansion | Partial | artifact/foreign limits exist, not universal |
| G10 stable OOM identity | Partial | RHI typed errors, cross-domain flattening remains |
| G11 failure injection | Fail | no controllable allocator fault lane |
| G12 large allocation audit | Partial | local caps, no generated global audit |
| G13 pool/arena lifetime contract | Fail | owners implement unrelated local semantics |
| G14 inline arena idle trim | Pass | `trim_idle_storage` + focused regression |
| G15 UI node pool bounded/trimmed | Pass | bucket/per-bucket/total bound + trim |
| G16 cross-cache budget arbitration | Fail | independent cache budgets |
| G17 external lease accounting | Partial | artifact lower-bound tracker only |
| G18 estimator calibration | Fail | no allocator-size/error receipt |
| G19 weighted eviction | Partial | byte/LRU local policies, no importance registry |
| G20 cross-cache trim epoch | Fail | no coordinator/epoch |
| G21 MemoryPressure service | Fail | no service/signal |
| G22 reclamation ladder | Partial | render ladder/local eviction only |
| G23 eviction retry loop | Partial | some local pre-insert eviction, no common retry |
| G24 dirty/pinned/inflight protection | Partial | RHI/asset local protection |
| G25 plugin generation isolation | Fail | policy config without allocation generation |
| G26 process/child/shared/GPU census | Partial | session/foreign/RHI local snapshots |
| G27 shutdown memory quiescence | Partial | outstanding session allocation guard, no all-owner proof |
| G28 trace/snapshot artifact | Fail | no common service |
| G29 RSS/child/shared/GPU closure | Fail | fragmented diagnostics |
| G30 allocation-free required gate | Partial | focused tests only |
| G31 fragmentation/soak | Fail | no qualification artifact |
| G32 release/product qualification | Fail | no product memory truth gate |

## 8. 参考引擎约束

本轮重读仓内参考切片，使用它们的结构性约束，不把实现语言直接照搬到 Rust：

- **Unreal**：`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/UnrealMemory.h` 与 `MemoryBase.h` 将 `FMemory`/`FMalloc` 作为统一入口，明确 `Malloc/Realloc/Free/TryMalloc/TryRealloc/GetAllocSize/QuantizeSize`、external allocation 和 low-level tracking；`LowLevelMemTracker.h` 提供 tag/scope 维度。Zircon 当前没有对应的统一 entry、try path、quantized size 或 scope/tag stack。
- **Godot**：`dev/godot/core/os/memory.h`、`core/templates/paged_allocator.h` 将 allocator、page/block 生命周期和 pool ownership 分开，强调可回收 page 与容器增长契约。Zircon 的 arena/pool 可回收动作仍是 owner-specific，缺 page/high-water/fragmentation receipt。
- **Bevy**：`dev/bevy/crates/bevy_ecs/src/storage/blob_array.rs`、`storage/table/mod.rs` 以连续列存储、明确容量/增长和实体表生命周期为核心；这支持 Zircon 保留 ECS contiguous storage，但要求 growth 在 commit 前可失败，而不是由默认 Vec OOM 终止进程。
- **Fyrox**：`dev/Fyrox/fyrox-core/src/pool/mod.rs` 的 generation-qualified pool handle、free-list 与 reuse 说明 slot lifetime/retirement 必须是显式 contract。Zircon 应将 generation、lease、reclaim reason 纳入内存 receipt，而不是只记录 entry count。
- **Unity Graphics**：`dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphObjectPool.cs` 与 `RenderGraphResourcePool.cs` 同时维护 pool reuse、frame allocation tracking、stale lifetime、resource size、exception cleanup 和 release validation。Zircon transient pool 已有其中一部分，但缺跨 owner resource-pool arbitration 和 product memory artifact。

## 9. 目标 Runtime 架构

```text
Platform/Device/Allocator probes
              |
              v
     MemoryBudgetResolver -----> ProductMemoryBudget
              |                           |
              v                           v
 AllocationSiteInventory ----> MemoryDomain/Tag Registry
              |                           |
              +---- AllocationReceipt ---+
                               |
                               v
                    MemoryPressureCoordinator
                               |
          +--------------------+--------------------+
          v                    v                    v
  ReclaimableOwner      OOMDisposition        Snapshot/Trace
 (cache/pool/GPU/ECS) (retry/degrade/reject) (RSS/GPU/lease)
          |
          v
   App/Editor Product Qualification and Release Gate
```

Runtime ownership must remain inside `zircon_runtime`/RHI; App consumes health and product disposition, Editor projects diagnostics and authoring warnings. Tooling can later migrate to Rust and consume the same receipts, but is explicitly outside this review's implementation scope.

## 10. 分阶段重构顺序

### M0 · 事实与命名冻结

- 生成 source/build `AllocationSiteInventory`，建立 domain/tag vocabulary 和 overlap rules。
- 统一 requested/accounted/resident/committed/released 定义；所有现有局部 reports 标注 owner 与 lower-bound caveat。
- 把 `GpuMemoryClass`、foreign output kind、asset lease kind 映射到 registry，不复制常量。

### M1 · Receipt 与 fallible boundary

- 引入 reservation/commit/rollback/release receipt 和 `ExpansionPlan`。
- 先覆盖 ECS growth、asset decode/decompression、GPU upload、plugin/native/foreign output，再收紧大 `Vec/Box/Arc` 入口。
- 稳定 `AllocationFailure` envelope；保留 RHI/Asset/Scene/Text domain source，禁止跨层只传 String。

### M2 · Reclaimable owners 与真实驻留

- 将 RenderGraph、ArtifactChunk、Text、UI node、command arena、shader prewarm、VM/plugin cache 适配 `ReclaimableOwner`。
- external/shared `Arc` 进入 physical payload ledger；estimator 记录 calibration/error bounds。
- 以 Unity/Fyrox 的 release validation/generation 约束补齐 dirty/pinned/inflight 与 exception cleanup。

### M3 · Pressure、OOM 与 lifecycle

- 建立 platform/device sampled pressure coordinator，按 severity/epoch 驱动 ordered trim。
- 统一 `admit -> reclaim -> retry -> disposition`，支持 optional rejection、render degrade、plugin quarantine 和 fatal crash receipt。
- shutdown 必须产生 all-owner memory-quiescence receipt，说明 retained allocator pages 与外部 lease。

### M4 · Trace、fault 与 qualification

- 低开销 sampled trace、heap snapshot/diff、RSS/child/shared/GPU collector。
- failure injection 覆盖 allocator failure、budget exceeded、device loss、plugin unload、decode expansion、shutdown timeout。
- 固定 workload、warmup/sample/statistics，建立 fragmentation/long-soak/p95 gate，并让 App/Editor product qualification 消费 artifact。

## 11. 验收标准

1. 每个 production allocation boundary 都能解析到 stable site、owner、domain/tag、generation 和 reservation receipt。
2. CPU/GPU/mapped/staging/foreign/plugin/cache/pool bytes 的 overlap 规则可机器校验；cache-owner bytes 不被误报为 RSS。
3. 低压、软压、硬压和 OOM 各有确定的 reclaim/retry/degrade/reject/fatal disposition；注入失败时 rollback 与 shutdown 仍可验证。
4. RenderGraph、asset、text、UI、ECS、command arena、plugin/native 至少各有一个真实 product consumer 读取统一 pressure/receipt，而不是只跑 isolated unit test。
5. `InlineCommandArena` 与 `UiSurfaceNodePool` 的局部 trim/bound 继续通过回归；其余 owner 也必须报告 high-water、released bytes、pinned/inflight blockers。
6. Windows desktop、UMA/mobile-like、discrete GPU 至少各有固定 workload 的 peak/resident/committed/reclaim/fragmentation/soak artifact。
7. App release gate 和 Editor diagnostic projection 在缺失 memory truth、quiescence 或 qualification artifact 时 fail closed。

## 12. 本轮验证与限制

- 已完成当前工作树静态扫描、代表 owner 源码阅读、生产消费者搜索、局部 focused test 文件复核及 Unreal/Bevy/Fyrox/Godot/Unity Graphics 参考切片复核。
- `MemoryDomain`、`MemoryTag`、`MemoryPressure`、`ProductMemoryBudget`、`AllocationSiteInventory`、`AllocationReceipt` 在 Runtime/App/Editor 当前树无统一实现；局部 `GpuMemoryBudget` 不改变该结论。
- 本轮没有修改 production Rust、tests、Cargo、ABI、ZUI 或 tooling；只新增本报告并更新索引/coverage。
- 未运行 Cargo（当前仓库存在既有动态验证/工作树阻断，且本轮为 review-only），未运行真实 WGPU/GPU、OS memory pressure、DLL/native allocator、fault injection、Miri/loom/sanitizer、fragmentation/long-soak 或 benchmark。
- 结论不能由 `Vec`/`Arc`/`clone` 词法次数推导 allocation 次数或性能；下一阶段必须以 receipt、snapshot 和固定 workload 数据验收。
