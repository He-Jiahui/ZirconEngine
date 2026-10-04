---
status: in_progress
review_date: 2026-09-07
plan_sources:
  - docs/plans/optimize/00-ui-architecture-performance-reassessment-2026-09-02.md
  - docs/plans/astra/optimize/01-review-and-repair.md
---

# 有界热路径与性能验证计划

## 当前问题与边界

optimize 已指出布局全扫描、几何驱动的无界 hit-test backing、重复 projection 和 allocation；这些属于待复核线索。当前工作树有许多 cache、allocation test 和持久化结构的在途实现，不能只按旧报告重做优化。

首先审查 Interface UI hit-test 与 Runtime UI 几何/布局，分别确认生产 caller、异常输入边界、热路径频次、已有缓存失效合同和回归测试。具体发现补充在本计划后再授权相应实施。

## 验收门槛

| 维度 | 必须满足的门槛 |
|---|---|
| 正确性 | 有效输入输出与顺序语义不变；边界输入不能 panic、越界或绕过预算 |
| 资源有界性 | NaN/Inf/超大有限几何不得按坐标范围分配或枚举；工作量由已接收元素数和明确预算限制 |
| 增量性 | 有 no-op 合同的重复请求不得重建与变更无关的全量产品；generation 变化必须正确失效 |
| 分配 | 被优化的重复请求在预热后达到对应 finding 声明的分配上限；不能只测 helper 而漏掉实际 caller |
| 时间 | 同一进程、同一 release workload 的 before/after 或现有 baseline；报告样本、预热、p50/p95/p99，正常路径 p95 回退不超过 5%，除非有明确安全成本依据 |
| 规模 | 覆盖空、小规模和 1k/10k 元素；病态输入另设预算断言，不能让测试机实际进行无界工作 |
| 证据 | 记录 source fingerprint、profile、硬件/OS、workload、实际结果；未取得 release/产品数据的维度保留 pending |

5% 是本轮回退门槛，不是已有测量结论。保留有效既有专项目标；新增热点以正确性等价基线的 release p95 改善至少 20% 为目标。对低于计时噪声的微操作，优先使用确定性的分配和访问计数，并保留产品性能待测状态。

固定本机 Ryzen 7 5800H、RTX 3060 Laptop GPU，记录 adapter、驱动、分辨率、画质及 workload；排除虚拟显示适配器。计时由验证 lane 在重编译竞争结束后进行，预热并交错 before/after，附 RSS/VRAM 与产品帧、交互证据。多个任务共享同一次 compatible package/feature 编译，性能采样与正确性回归分开，禁止逐修复重复构建。

当前续接先核对 RG-A3/A4 与 UI-A3 的已有实现及完整回归集合，再执行配对测量；没有新增性能通过证据。RT213-P1-009/010 的局部包围盒变换与生产 `prepared_local_bounds` collector 接线已存在；由 `05-cpu-visibility-bounds.md` 继续记录同代 bounds 的 fail-open、历史失效测试与产品 CPU/GPU 一致性验证，后者仍待受管证据。

## 当前源码确认项

### UI-A1：非有限根尺寸使增量布局永久失效（P1）

- 状态：`implemented_pending_validation`。发现时 `zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs` 用原始 `UiSize` 比较和缓存，使 NaN 输入持续失效；当前 rebuild/incremental/authored_geometry 已在比较与保存前统一规范化。已有 1k/10k no-op 回归和 release paired workload，仍须当前封存快照实测。
- 影响：异常 native/ABI 或直接 API 尺寸可使正常 no-op 帧持续布局、文本测量和安排；后续 `.max(0)` 不能修复比较键。
- 修复边界：在现有 surface 根尺寸入口统一规范化为 finite 非负值，比较、缓存键和 layout pass 消费同一值；不另建全局布局 owner。必须覆盖 `compute_layout`、`rebuild_dirty` 和 authored-frame 入口的一致性。
- 验收：NaN/Inf/负尺寸首轮产生确定性有限布局；相同规范化输入第二轮 `layout_recomputed=false` 且 measure/arrange probes 为 0。1k/10k 节点重复 100 次无脏帧保持同一 no-op 计数；正常 resize 和 font generation 变化仍正确重算。

### UI-A2：直接虚拟列表配置绕过实体化预算（P1）

- 状态：`implemented_pending_validation`。发现时公开 `UiVirtualListConfig::overscan` 与极端 viewport/extent 绕过解析器预算；当前 materialization 在创建 state/槽/回调前复用 `MAX_UI_LAYOUT_DISCRETE_VALUE=4096` 并返回 `SlotCapacityExceeded`。已有 exact-limit/+1、最大整数及非有限输入测试；不得将源码检查等同于运行通过。
- 触发：logical_count=1,000,000、overscan=usize::MAX，即可把逻辑列表实体化为百万槽。极小 item_extent 与巨大有限 viewport 的比值也必须纳入容量检查，不能只限制 overscan。
- 修复边界：在实际 runtime admission、任何 owner/state mutation 前复用已有离散值限制或明确物理槽预算，按现有错误合同拒绝非法配置；保留合法可见范围与 backfill 语义。低层公开 slot map 的预算责任需明确，不能默默截断后伪称完整物化。
- 验收：最大整数、最小正 extent、非有限尺寸和预算边界用例均有界；拒绝后 tree/state/changes 不发布半状态。正常 1k/1M logical items 的同一 viewport 保持相同槽位上限；滚动只更新必要映射，重复窗口无新增分配。

### 已过时的旧描述

当前 `TaffyParentProductCache` 已保留父级产品，hit-test 也已有 persistent grid 与 backing bounds；不能重复宣称所有布局仍每次创建临时 Taffy 树或所有 hit grid 都无界。仍需分别实测失效、候选枚举和实际调用层成本，局部底座不等同于完整产品性能达标。

### UI-A3：密集几何更新反复扫描同一 hit-grid cell（P1）

- `zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs` 与 projected frame hit-test 按更新项分别 retain/insert，同 cell 的 K 项更新可重复扫描 M 项 backing，形成 K*M 成本。
- 按 cell 汇总整批增删，再一次重建该 cell；预检所有预算后再发布，保留 painter order、完整命中集合、COW 快照和失败原子性。
- owner 细化于 `04-hit-grid-cell-batching.md`；密集更新、跨 cell 移动、删除/插入、共享快照和 fallback 用例共同验收。真实 cell visit 计数应与受影响 cell/成员线性相关；release paired p95 目标改善至少 20%。

公开正半径 hit-query 仍有另一项待深审预算缺口：巨大有限 radius 可遍历大量 cell/candidate 并排序。不得静默转换为零半径而丢失完整结果；后续先确定 typed budget failure 或 continuation 合同及全部消费者，另立子计划，不混入 UI-A3 性能结论。

`UiHitQueryScratch` now also retains the nearby-fallback sort buffer, so repeated admissible
radius queries no longer allocate that temporary vector. This leaves candidate enumeration,
fallback ordering, and the unresolved large-radius admission contract unchanged.

## 当前源码确认项：RenderGraph

### RG-A1：新增 state plan 的 const 比较不能编译（P0）

- 当前状态：`implemented_pending_validation`。`crosses_queue` 已移除 const，以下保留原编译缺陷和验收条件。

- `zircon_runtime/src/render_graph/graph/resource_state_plan.rs:63-65` 的 `pub const fn crosses_queue` 调用 QueueLane 派生 PartialEq 的 `!=`；当前稳定编译器独立最小复现已返回 E0015。
- 修复：该方法只由运行期使用，移除无需求的 const；不改变 enum/wire 合同。
- 验收：受管 Runtime lib 编译与现有跨队列 true/false state-plan 测试通过。最小编译复现不是 workspace 编译通过的证据。

### RG-A2：最后一次相交查询丢失部分子资源前驱且可平方退化（P1）

- 当前状态：`implemented_pending_validation`。state plan 已消费 AccessScopeTracker 生成精确前驱；其 tracker 全表合并与纹理枚举的剩余复杂度另归 RG-A4，不能由已有 work_count 低值宣称达标。

- `resource_state_plan.rs:98-145` 每次访问追加历史 Vec，查询只取逆序第一个 overlap。mip0 ColorAttachment、mip1 StorageTextureWrite 后全范围 SampledTexture 消费会只生成一个前驱状态，并错误扩张到目标整范围。
- 相同实现对 N 个互不相交范围扫描 N(N-1)/2 个历史条目；10k 范围即 49,995,000 次检查。
- 修复：复用 graph 已有 AccessScopeTracker 的 texture cell/buffer interval 机制维护当前非重叠状态；对本次范围各实际前驱生成精确 transition，覆盖被写入状态，不保留无效全历史。不得在本切片扩展物理 GPU barrier/queue 后端。
- 验收：不同 mip/layer/aspect、相邻/重叠 buffer range -> 大范围消费者的所有前驱与范围正确；重复同状态无冗余 transition；10k disjoint 工作计数接近线性或 NlogN，明确断言上限并做 release paired workload。

### RG-A3：资产变化时 component-journal replay 可能保留旧几何（P1，下一切片）

- `graphics/scene/resources/resource_streamer/resource_streamer_residency.rs:160-166` 在 geometry resolution 前以 world+journal generation 返回 Replayed；prepared mesh/model revision 更新不必伴随 ECS journal 变化，因此 RenderScene 的 bounds/revision 可保留旧值。
- 计划：使 replay admission 同时检查已引用资产 geometry generation 的变化，利用现有资源事件游标与 resource-to-primitive 反向索引更新受影响 primitive；游标缺口执行有界全量恢复。不能以每帧全部解析资源替换旧快速路径。实施归 `03-render-scene-resource-replay.md`。
- 验收：同 component artifact + mesh revision/bounds 改变只发布一次 geometry dirty；未变化帧仍零 geometry resolution。先复核其确切源事件与 owner 再授权实现。

### RG-A4：区间合并漏计平方扫描，纹理范围过早膨胀（P1）

- `render_graph/builder/access_scope_tracker.rs::merge_adjacent_buffer_segments` 每次 mutation 收集并扫描整张区间表；10k disjoint 插入约 49,995,000 次合并检查未进入原查询计数。texture_cells 先按 mip*layer*aspect 枚举，shape 验证晚于这次分配。
- 只检查 mutation 边界和受影响相邻区间，真实计数覆盖合并与展开；纹理形状先验证，范围保持紧凑表示，不能通过拒绝合法大纹理伪造改进。
- 实施归 `02-render-graph-state-tracking.md`；buffer disjoint/overlap/adjacent、全部 texture aspect/mip/layer 前驱语义、非法形状在分配前拒绝与同负载 release p95 共同验收。

### 渲染旧报告修正

Runtime223/300 关于“完全没有 resource-state plan”的描述已过时，当前有中立 compiler plan/dump/tests，但尚无完整初终态、外部所有权或 native lowering。`scene_renderer_render_scene.rs` 也已在两个提交路径调用 GpuSceneJournalConsumer，旧“无产品 caller”描述已过时；pending draw 全扫描与 semantic executor 无真实 consumer 的缺口仍应分别保留。

## 实施与批量测试

M0：定位具体调用链和可复现缺陷；确认语义与预算。

M1：补回归并修复最低共享 owner，保持已有模块职责；允许多个独立修复一起进入 Interface/Runtime package 验证批次。

M2：运行同批回归、规模和性能测试；所有计数与输出合同通过后再判断实际性能。编译期间继续 features/layouts 中的独立修复。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Hub01/02/04 Node 热路径 | 目录搜索、目录分组、项目搜索、交付/引擎投影 | implemented_pending_validation | 2026-09-07 | Node 22.13.1 Windows 交错采样 4/4 性能门槛通过；原生窗口、GPU、RSS/VRAM 和受管 Cargo 仍待验收。Node 24.19.0 首轮交付投影未达 50% 门槛，未替换 Node 22 基准证据。|
| UI-A3 / RG-A4 | hit-grid cell batching 与 RenderGraph interval/texture tracking | implemented_pending_validation | 2026-09-09 | 当前源码与回归已落源；UI/RG 静态合同 `18/18` 通过，焦点 rustfmt/diff-check 通过。UI 密集 1k/10k、RG 10k disjoint/最大 texture array 的 ignored release p50/p95/p99 及完整 Runtime/Editor managed Cargo 仍待验证；外部 `E:/Git/zr_vm` 最新 `d717af6c8fefb4f0e1a5a69423904327296f1254` 仍有 75 条 dirty status，保持 pending，详见异步日志。|
| RT213-P1-009/010 | prepared local mesh bounds 的 affine CPU visibility 投影与 fail-open | implemented_pending_validation | 2026-09-09 | 生产 collector 已传递 `prepared_local_bounds`，局部 bounds/无效输入/历史 revision 回归与静态卫生通过；CPU/GPU 产品一致性及 release p50/p95/p99 仍待受管验证。|
| Native plugin trust/inventory | inventory preflight、selection-scoped authority 与 Runtime/Editor load/hot-reload admission | implemented_pending_validation | 2026-09-09 | 相关静态合同 `45/45`，public-surface `86/2/risks=[]`；真实 DLL side-effect、Cargo 和 release 性能仍待受管批次。|
| RT213 visibility query | static index overflow/cell candidate normalization | implemented_pending_validation | 2026-09-09 | Query candidate collection now uses one sorted/deduplicated Vec; correctness/static checks passed, managed release comparison remains pending.|
| RT213 VIS213-G08 | disabled directional shadow view admission, single planner owner, and cascade capacity | implemented_pending_validation | 2026-09-09 | Disabled directionals produce zero views; additional enabled directionals no longer create unconsumed views; capacity reserves the effective cascade count for the first planner owner. Owner regressions and static checks passed, enabled cascade parity and release p50/p95/p99 remain pending.|
| Runtime11A/M30 | default nested-scroll candidate scratch reuse | implemented_pending_validation | 2026-09-10 | Runtime UI pointer/scroll/hot-path static batch `73/73`; complete-candidate validation and warm-capacity regressions pass; managed Runtime Cargo and Windows Release allocation/time p50/p95/p99 remain pending.|
| Runtime11A/M31 | retained Taffy child-handle projection reuse during topology updates | implemented_pending_validation | 2026-09-10 | Retained-product capacity/content regression and focused Runtime UI/Taffy/layout contract batch `77/77` pass; managed Runtime Cargo and Windows Release allocation/time p50/p95/p99 remain pending.|
| Runtime11A/M32 | retained arranged-visibility resolver state/path scratch bounded to twice the current node count | implemented_pending_validation | 2026-09-10 | Rust regression is present for warm reuse, high-water shrink, clone scratch omission and visibility (Cargo execution deferred to managed testing); post-change combined Runtime UI/Editor contracts `86/86` and scoped Rustfmt pass; new source/plan files are whitespace-clean; managed Cargo and Windows Release allocation/time p50/p95/p99 remain pending.|
| Runtime11A/M33 | bounded Canvas layer and per-parent child projection capacities | implemented_pending_validation | 2026-09-10 | Existing Canvas behavior coverage plus source capacity guard; combined Runtime/Editor hotpath contracts `135/135`, Python syntax, Rustfmt, and scoped diff checks pass; managed Runtime Cargo and Windows Release allocation/time p50/p95/p99 remain pending.|
| Editor04/M27 | cached Asset Browser search normalization | implemented_pending_validation | 2026-09-10 | Editor asset projection/Watcher static batch `52/52`; mixed-case/idempotent setter regression and no-per-call-normalization guards pass; managed Editor Cargo and Windows Release allocation/time p50/p95/p99 remain pending.|
| Editor04/M28 | borrowed ASCII filter matching and folder-tree capacity reservation | implemented_pending_validation | 2026-09-10 | Combined Runtime UI/Editor asset and evidence-contract batch `111/111`; Python syntax, Rustfmt, and scoped diff checks pass; managed Editor Cargo and Windows Release CPU/allocation/RSS p50/p95/p99 remain pending. |
| Editor04/M29 | borrowed locator parent-folder membership checks | implemented_pending_validation | 2026-09-10 | Combined Runtime/Editor hotpath contract batch `135/135` passed; focused asset/Taffy recheck `53/53` passed with UTF-8 boundary coverage; Python syntax, Rustfmt, source guard, and scoped diff checks pass; managed Editor Cargo and Windows Release CPU/allocation/RSS p50/p95/p99 remain pending. |

## Benchmark Evidence Scope Correction (2026-09-09)

Runtime629's manifest-membership helper previously used 32,768 generated roots, above
`MAX_PROJECT_ASSET_ROOTS` (4,096). Its input is now the admitted root limit. The Runtime629 glTF
dependency, Editor612 reset-seed, and Editor633 widget/export harnesses are explicitly labeled
microbenchmark workloads. They do not constitute a product result because they bypass the real
caller and its surrounding allocation, validation, projection, or executor work.

Those four harnesses now use 31 alternating samples, nearest-rank percentile reporting, and
p50/p95/p99 output. This corrects the former 17-sample layout where p95 could be the maximum
sample; it does not produce a new performance pass. A managed Windows Release real caller
measurement, hardware/profile record, and compatible Cargo validation remain pending. No result
from these helper microbenchmark harnesses may be used as performance acceptance while the external
validation input is dirty.
