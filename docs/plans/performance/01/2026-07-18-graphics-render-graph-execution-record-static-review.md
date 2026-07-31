---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphTrace.cpp
tests:
  - render graph execution record four of four Rust files reviewed, 1818 current lines
  - compute workload audit temporary partition source guard changed from one to zero
  - matching then unexpected dispatch ordering regression added
  - fixed RenderPassStage storage source guard retained
  - rustfmt and scoped git diff check passed
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics RenderGraph execution record逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`render_graph_execution_record.rs`、`compute_workload.rs`及两层tests当前4/4个Rust文件、1,818行，并追踪`execute_graph_stage`产品写入和`update_stats/base_stats.rs`消费。该记录同时承担per-pass详细诊断、compute workload审计和常驻frame stats权威，导致compiled graph中已有的pass/executor/dependency/resource元数据在每帧执行时多次深clone，随后又被多轮统计查询扫描。根因继续归属PERF-MVP-343；本轮直接消除compute workload审计的临时partition Vec，不创建重复编号。

## 已直接止损：compute audit不再partition临时Vec

旧`audit_compute_workload`对每个compute pass执行`dispatches.iter().partition(...)`，无论slice大小都构造matching/unexpected两份临时Vec。产品`execute_graph_stage`传入的是该pass执行器刚`take_compute_dispatches()`得到的局部slice，通常0或1项，堆分配没有提供算法收益。

现在先借用slice查找第一个matching index，再以借用filter按原合同顺序记录“其余matching，随后non-matching”unexpected dispatch；planned-none路径继续线性借用全部dispatch。源码合同把`.partition(`从1降为0，并新增混合顺序回归，锁定primary matched、duplicate matching、foreign unexpected的原有分组顺序。当前Cargo仍受共享FIFO约束，所以只记录source RED→GREEN、rustfmt与diff-check，不把该修复写入`review.md`。

## PERF-MVP-343补充：per-pass元数据多owner与stats重复扫描

产品执行一个pass时，当前路径先为profile scope clone pass name，再clone executor id；构造`RenderPassExecutionContext`时又clone pass name、dependencies和resource accesses。执行后profile record再次复制pass/executor String，execution record再复制同一pass/executor/dependencies/resources并保存debug-marker String。compute dispatch和workload audit还分别保存pass/executor/pipeline/resource String，导致同一compiled graph的稳定元数据在graph、context、execution record、profile和audit多owner存在。

frame stats写回不是消费一次summary：workload audit按planned/matched/missing/mismatch/unexpected扫描5次，volumetric dispatch按count/group volume/uploaded bytes扫描3次，resource access按shadow read/write扫描2次，stage/queue又各自重扫。`profile_report()`clone全部profile rows，alias report也在stats边界深clone；此前固定17 stage的唯一性统计已从`BTreeSet`改为bool表，只解决了其中一处小分配。

正确方向是compiled graph继续作为immutable metadata authority，execution context借用pass handle/切片，不持有owned metadata副本；always-on路径在pass完成时一次增量累计dense `RenderGraphExecutionSummary`，stats按Copy/borrow读取。完整pass names、resource rows、alias Strings和workload detail只在capture/profile/diagnostic请求打开时记录，且以interned/Cow/static label或compiled handle引用；同generation详细report用Arc共享，不在stats再clone。

## 参考与验收

Bevy的render diagnostics由显式`RenderDiagnosticsPlugin`启用；`FrameData`通过`clear()`复用path/span Vec，并在submitted/finished frame池间交换整套workspace，稳定帧不重建capacity，名称接受`Cow<'static, str>`。Unreal RDG trace同时受`RDG_ENABLE_TRACE`和`IsEnabled()`保护，关闭时在详细resource/pass消息物化前返回；compiled RDG对象继续持有元数据。Zircon应保留always-on确定性计数，但不应因此强制每帧复制完整调试DTO。

按passes 16/64/256/1024、resources/accesses 1/8/64 per pass、compute dispatches 0/1/8、diagnostics off/on、stable/1% graph changed记录pass/executor/dependency/resource clone bytes、record owners、audit temporary allocations、summary scans、profile/alias clone bytes、workspace growth与CPU p95。diagnostics-off稳定帧要求compiled metadata clone bytes=0、audit temporary Vec=0、summary全量重扫=0、详细String row build=0；diagnostics-on每pass详细row至多1份、workspace warm growth=0、stats额外deep clone=0。pass/stage/queue/dependency/resource/workload顺序、诊断内容与Cargo/F2/RenderDoc对拍完成前保留在`pending.md`。
