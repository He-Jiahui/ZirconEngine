---
related_code:
  - zircon_runtime_interface/src/tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
tests:
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs
  - zircon_runtime_interface/src/tests/window_input_contracts.rs
  - zircon_runtime_interface/src/tests/window_runtime_event_adapter_contracts.rs
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
doc_type: implementation-evidence
status: partial_static_complete_dynamic_pending
---

# Runtime interface clean contract tests 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/tests/**`本批新增完成 **22** 个受跟踪且clean的 Rust测试文件、**6,262** 行逐文件阅读；另1个clean `reflect_contracts.rs`已计入reflect批次，不重复计数。目录当前共31个Rust文件，其中6个dirty tracked与2个foreign untracked文件不吸收、继续pending。本轮未修改Rust源码。

## 性能结论

- 现有合同对serde/default/ABI shape、顺序、错误和兼容语义覆盖较强，但多数fixture只有1–3个节点、row或event；除`pipeline_contracts.rs`的100次pointer move零layout断言外，没有heap allocation、retained bytes、queue age、p50/p95或100k规模门禁，不能作为动态性能验收。
- `ui_ecs_projection_contracts.rs`验证interaction-only/render-only分类、stage/domain派生与snapshot/delta重算语义，但只使用1–3节点；补强 **PERF-MVP-278**：增加1/1k/10k/100k nodes的passes、BTree/Vec alloc、published bytes与stable-generation zero-work计数。
- layout/report、v2与surface合同只覆盖少量selection、空arena或单entry；继续归 **PERF-MVP-263/274/312**，需要large tree、slots/tracks及generation复用门禁，不能用小fixture证明增量复杂度。
- dispatch reply/error合同序列化完整event/reply/effect/diagnostics，支持 **PERF-MVP-293/294** 的wide result重复所有权风险；需要1MiB payload与100k events的owners/clone/retained bytes预算。
- window pump只验证相邻redraw合并、事件映射与批次顺序；runtime adapter测试只使用3-event batch，未覆盖move/raw-motion/wheel/axis coalesce、entries+bytes+age硬限或late-invalid partial work。继续归 **PERF-MVP-297/314/426**，由Runtime12补125/500/1000 Hz和100k mixed-event门禁。
- `world_sync_contracts.rs`的NotModified语义测试在调用前仍先构造完整rows，直接保留 **PERF-MVP-563** 的“先投影再决定不发布”验证缺口；应断言unchanged generation row build/visit/bytes为0。
- `boundary.rs`递归扫描源码只发生在合同测试，不进入产品热路径；本批没有发现需要新增编号的独立产品瓶颈，也没有足够小且已获特定设计批准的Rust行为修复。

## 动态验收

1. interface package当前源码合同测试通过，按Runtime09/12的规模矩阵补确定性counter；计时仅作辅助，不替代allocation/visit/queue预算。
2. ECS/layout/v2/surface使用1/1k/10k/100k nodes与stable 300 frames，验证stable generation build/clone/heap=0，delta成本随changed rows而非总树规模增长。
3. dispatch/window使用100k mixed events与125/500/1000 Hz输入，记录event/result clone bytes、coalesce、edges、queue entries/bytes/age、drops与p95；press/release/cancel/key/text/IME严格保序。
4. world sync unchanged generation记录row projection visits/alloc/published bytes=0，并保留revision/NotModified/serialization合同。
5. F0/F4产品输入、UI、world-sync trace通过；相关dirty/untracked测试由原owner在当前源独立验收。

current-source Cargo、规模counter与F0/F4产品trace未完成，因此本批继续保留在 `pending.md`，不进入 `review.md`。
