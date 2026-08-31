---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-hit-query-scratch-and-quadratic-dedupe
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/query_scratch.rs
  - zircon_runtime/src/ui/surface/arranged.rs
tests:
  - exact-query zero-intermediate-candidate-Vec guard
  - dense-radius near-linear dedupe counter
  - 120-Hz pointer hit-query allocation and CPU trace
---

# Runtime UI hit query临时分配与二次去重

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`hit_grid.rs`与`tree/hit_test.rs`命中调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：EditorUI02提供dense arranged/input-policy index。
- 交接原因：hit query scratch、cursor-radius策略和pointer frame budget属于input dispatch；arranged结构索引不能在input层私建。

## 失败现象与复现证据

PERF-MVP-316：每query原先分配五个Vec；radius跨cell以`Vec::contains`去重，密集候选最坏O(K²)。本轮已TDD让radius=0借用单cell，删除四个中间candidate Vec；radius query、linear arranged get和ancestor input-policy仍未解决。

## 最低共享层根因

hit grid没有generation-owned query scratch/dense visit mark，arranged tree也没有O(1) node index与继承input-policy；每次高频pointer move只能重新收集、去重、排序和走祖先。

## 架构修复验收

- radius query用generation-stamped dense mark/scratch或等价近线性去重，不以全grid clear替代。
- frame-local scratch受entry/byte预算且不跨并发query泄漏；exact path保持零中间candidate Vec。
- EditorUI02 index使arranged get近O(1)、input policy不逐candidate走祖先。
- 1/100/1k/10k overlapping entries、radius 0/8/64、120 Hz记录probes/sort/alloc/CPU p95；route/z/paint/scope/world parity与Cargo/F4 trace通过。

## 禁止临时方案

- 不得用每query新建HashSet替代Vec而不记录allocation/byte预算。
- 不得关闭cursor-radius fallback或改变exact-hit优先级来降低成本。

## 修复结果与回传

Open state: `radius scratch 与 retention budget 代码完成，受管 Rust/产品规模证据待恢复`。

- `UiHitTestIndex` 现持有 serde-skipped、clone-reset 的 mutex scratch；radius query 用
  generation-stamped `Vec<u32>` marks 对 cell entry references 做一次探测去重，候选排序后
  复用容量，poisoned lock 可恢复。radius 0 仍走 borrowed single-cell 路径。
- scratch retention 以 `max(1024, current_entry_count * 4)` 为 entry budget，并按
  `u32 mark + usize candidate` 的实际 capacity 计算 byte budget。历史容量超过预算时，
  两个 buffer 都重建到当前规模；16,384→32 entries 回收和随后小规模容量复用已有 focused
  regression，dense 256-entry/4-cell test 同时约束 1,024 probes、256 unique 与 byte budget。
- immutable snapshot 2090 / ticket `fa6dccf51e19454d9aa5a45a271c3b8d`
  于 2026-08-24 在 Cargo 前终态 failed。copy
  `6708798578ef40f8bcec59336dc13673` 的 durable error 为
  `validation_copy_compile_time_resource_missing`：
  `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` 引用已不存在的
  `zircon_editor/assets/ui/editor/animation_editor.zui`。该 foreign tracked-deleted
  resource 不属于本 failure scope，不能以重试、伪造 asset 或扩大 owner 绕过。
- 仍需在资源 closure 恢复后取得 focused Rust terminal，并补齐
  1/100/1k/10k、radius 0/8/64、120 Hz allocation/CPU p95 与产品 parity；因此不得执行
  fixed return。
