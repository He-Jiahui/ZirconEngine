---
related_code:
  - zircon_runtime/src/core/framework/render/core_pipeline
  - zircon_runtime/src/core/framework/render/frame_extract
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/12-effects-particles.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
tests:
  - core pipeline thirteen of thirteen Rust files reviewed
  - frame extract subdirectory three of three Rust files reviewed
  - source-guard RED to GREEN for phase spans, summary indexing and borrowed static-batch keys
  - one-pass override constructor behavior test added
  - rustfmt and scoped git diff check passed
  - current-source Cargo, scale counters, F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render core-pipeline与frame-extract逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`core/framework/render/core_pipeline/**` 13/13和`frame_extract/**`子目录3/3，共16个Rust文件、2,053行；另聚焦追踪root `frame_extract.rs`的geometry/sprite构造、scene world产品调用和submission particle counters。root文件只做调用点追踪，不在本证据冒充逐文件覆盖。

## PERF-MVP-339：phase queue全表读取与重复artifact

`RenderPhaseQueue`已按`(phase_order, sort_key, entity)`排序，但每个render stage的`items_for_phase`仍从头过滤全部items；sprite/mesh多stage会重复O(P×N)扫描。summary又为每item线性查13个phase rows和10个order spans。本轮TDD把phase读取限制到两个`partition_point`求得的有序phase-order span，并把summary写入改为O(1) phase/order索引；source guards、rustfmt与diff check通过。

剩余主因是每camera/frame同时物化source snapshots、phase inputs和sorted phase items，稳定generation仍全量建Vec/排序；summary还反复创建固定diagnostic Strings/phase Vec。Bevy按retained view持久保存`SortedRenderPhase`，新帧只drain transient items或更新对应entity，再排序实际变化集合。Render09/17应据此建立per-view generation phase owner、phase ranges和复用storage；诊断直接借用phase metadata与已有ranges。

## PERF-MVP-340：static batch每camera/frame重建

`GeometryExtract`原为每个static mesh把`RenderLayerSet`展开成新`Vec<u32>`作为BTreeMap key；scene随后先用空overrides全量建一次static batches，再立刻带真实overrides全量重建。TDD已改为借用layer set key并新增一次性overrides constructor，scene产品路径只build一次。

但稳定场景仍对每camera/frame重建整个BTreeMap、mesh-index Vec和重复entity Vec；同一静态batch在多camera间没有generation共享。Render03/17应让scene/static generation拥有compiled batch membership，camera extract只投影visibility/phase references；material override或layer变化只失效受影响entity/batch，不保留frame-local BTreeMap权威。

## PERF-MVP-341：particle history diagnostics重复索引

submission context同帧依次调用`previous_state_sprite_count`与`anonymous_stream_ambiguity_sprite_count`：前者先由当前sprites建anonymous BTreeMap+BTreeSet，再由previous建identity BTreeMap并扫描当前；后者又从当前sprites重建一份anonymous BTreeMap。particle burst把多个树分配、排序和至少三轮当前扫描放在frame preparation主线程。Render12/17应产出一次`ParticleHistoryMatchReport`，以stable sprite key/dense或reused hash scratch单遍汇总matched/missing/ambiguous并由stats复用。

## 验收要求

phase按items 0/1/1k/100k、phases 1/4/13、cameras 1/8、stable/1% changed记录queue build/sort、item visits、Vec/String alloc、summary probes与CPU p95：phase read visits≤同order span，stable generation queue rebuild/sort=0，summary per-item probes=O(1)。static batches按meshes 1/1k/100k、batches/layers/overrides 1/100/10k记录key projection、BTree probes、indices/entities bytes、builds/camera：layer projection alloc=0，当前产品调用build=1，最终stable generation跨camera rebuild=0。particles按current/previous 1/1k/100k、anonymous 0/10/100%记录passes/tree nodes/alloc：history index build≤1/frame或generation。排序/override/visibility/particle identity行为、Cargo、F2 trace与RenderDoc全部通过前，两目录留在`pending.md`。
