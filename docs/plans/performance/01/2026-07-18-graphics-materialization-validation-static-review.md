---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphValidation.cpp
tests:
  - materialization validation one of one Rust files reviewed, 424 current lines
  - per-frame BTreeSet references changed from two to zero
  - compiled graph lifetime index lookup changed from zero to two production call sites
  - compiled lifetime index source guard added
  - rustfmt and scoped git diff check passed
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics materialization validation逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`materialization_validation.rs`当前1/1个Rust文件、424行及其内8个行为测试，并追踪产品帧在所有资源binder完成后、graph执行前的always-on调用。验证会统计typed/external/sparse覆盖并拒绝stale绑定，正确性价值明确；但旧实现每帧把全部live lifetime names重新收集进`BTreeSet`，随后仅用于stale texture/buffer lookup，而compiled graph当前已有name→lifetime索引。本轮直接复用现有索引，剩余dense binding/validation收敛继续归入PERF-MVP-366。

## 已直接止损：复用compiled lifetime索引

旧成功路径即使没有stale或missing资源，也为每个live lifetime分配一个tree node，再对每个bound texture/buffer执行O(log N)查找。现在删除该临时`BTreeSet`，stale-binding过滤直接调用`graph.resource_lifetime(name).is_none()`，复用compiled graph的O(1)索引；错误分类、排序与文本没有变化。源码RED→GREEN记录为`BTreeSet`引用2→0、production indexed lookup 0→2，并新增同文件源码合同。

## PERF-MVP-366补充：always-on验证仍按名字扫描多张资源表

修复后成功路径仍依次遍历全部lifetimes、bound texture names与bound buffer names；每项查询进入String-keyed执行资源map，随后另行构造resource和alias reports。只要执行资源仍以多张`BTreeMap<String, ...>`为权威，这些扫描就是每帧提交成本，也难以证明stale绑定在类型层不可能发生。

RG-M2验收后的compiled dense materialization计划应同时生成expected typed/external/sparse counts与dense required-binding mask。执行资源按handle/slot写入bound bitset和增量summary，验证只做word-wise required/bound/stale比较；只有错误时才通过compiled handle反查名称并格式化完整诊断。required external仍是per-frame动态输入，不能简单删除release产品校验；目标是把字符串树扫描变成紧凑位集/计数，而不是降低正确性。

## 参考与验收

Unreal RDG的重型user validation位于`RDG_ENABLE_DEBUG`边界，且多处受`GRDGValidation`早退控制；Zircon仍需要产品帧required-external失败语义，但可借鉴“热路径只保留必要状态、详细诊断按开关/失败展开”的分层。当前compiled name index复用是低风险第一步，最终以dense binding contract消除名字表全扫。

按lifetimes/bound resources 16/64/256/1024、required external 0/1/64、stable/1% changed记录validation passes、name-index builds/lookups、tree/String/Vec allocations、bitset words、error formatting与CPU p95。当前目标为per-frame lifetime-name index build/tree alloc=0；最终成功路径String lookup/format=0、required/bound比较O(words)，错误路径仍输出确定排序与完整resource type/name。现有missing typed、required/report-only external、stale texture/buffer测试、受管`materialization` focused gate、F2与RenderDoc完成前保留在`pending.md`。
