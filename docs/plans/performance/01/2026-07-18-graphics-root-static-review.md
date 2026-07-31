---
related_code:
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/graphics/resource_limits.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
tests:
  - current graphics root Rust source census 5 of 5 files reviewed, 681 lines
  - runtime prepare execution call graph traced through scene renderer and plugin registrations
  - existing PERF-MVP-379, PERF-MVP-399 and PERF-MVP-406 responsibilities confirmed
  - current-source Cargo, F2 plugin scale and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics root静态审查（2026-07-18）

## 当前源覆盖

`zircon_runtime/src/graphics`根级当前5/5个Rust文件、681行已逐文件静态阅读：`debug_markers.rs`71行、`mod.rs`122行、`prelude.rs`16行、`resource_limits.rs`12行、`runtime_prepare_collector.rs`460行。facade/prelude只做owner导出，resource limits为常量合同，没有独立每帧资源或算法热点。

## 性能责任

`RuntimePrepareCollectorContext`把完整frame extract/snapshot、prepared sidebands、`Device`、`Queue`与可变`CommandEncoder`直接交给插件collector；执行调用图在scene renderer submission路径按注册顺序串行调用全部collector，每帧新建external-binding Vec并merge owned outputs。该根因已由`PERF-MVP-379`定义CPU prepare artifact与render-thread record/apply两阶段、有界plugin/compute lane和持久binding/output owner，本页不重复编号。三个context合同测试又分别调用`RenderBackend::new_offscreen()`，测试初始化复用归`PERF-MVP-406`。

`marker_for_render_graph_pass`为每个执行pass动态`format!`前缀；调用点同时线性按pass name查找、clone profile name与executor id。它属于`PERF-MVP-399`的compiled dense pass/executor/diagnostic handle，不应在marker helper内用第二套cache掩盖。最终compiled artifact持有稳定marker/profile/executor identity，执行期不得分配String或回退name lookup。

`register_external_buffer_binding`的logical/backing String和WGPU handle clone发生在每帧collector回调中，但也必须由`PERF-MVP-379`的持久generation binding identity解决；局部改借用字符串无法解决binding Vec和GPU handle owner生命周期。因而本切片没有修改生产代码。

## 验收状态

当前只完成逐文件静态审查与调用图核对。Cargo协调器的JSON解析阻塞尚未修复，collector 0/1/16/64规模callback wall、payload/binding bytes、queue age/drop、F2 plugin parity及RenderDoc marker/capture均未得到current-source动态证据；继续留在`pending.md`，不进入`review.md`。
