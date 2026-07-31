---
related_code:
  - zircon_runtime/src/core/framework/render/frame_phase_queue_summary.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/core/framework/render/prepared_runtime_sidebands.rs
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/core/framework/render/renderer_common.rs
  - zircon_runtime/src/core/framework/render/shadow.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/12-effects-particles.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/buffer_vec.rs
tests:
  - sideband relevance renderer-common shadow root six of six Rust files reviewed
  - focused Hybrid GI Virtual Geometry and particle production output sources traced
  - source-guard RED to GREEN for direct diagnostic-name construction and single-owner VG sideband feedback
  - rustfmt and scoped git diff check passed
  - current-source Cargo, scale counters, F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render sideband/relevance/renderer-common逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`frame_phase_queue_summary.rs`、`plugin_renderer_outputs.rs`、`prepared_runtime_sidebands.rs`、`relevance.rs`、`renderer_common.rs`与`shadow.rs`当前6/6个Rust文件、1,501行；并聚焦追踪Hybrid GI、Virtual Geometry、particles的生产output/collector与feedback merge。prepared sideband主链已使用owned move/`mem::take`，relevance为compact bitset，shadow为Copy report，material override只在配置/资产变更时构建，均无新的稳态独立热点。frame summary诊断名原先额外分配临时`Vec<&str>`，本轮已改精确capacity的单String构建并继续归入PERF-MVP-339。

## PERF-MVP-347：Virtual Geometry frame sideband被深clone回renderer output

VG runtime-prepare collector原对`prepared_virtual_geometry_readback_outputs()`做完整clone，只为注册page-request external buffer；renderer随后保存这份clone，frame末又把renderer output与原sideband owned merge。大规模page table、cluster、visbuffer、traversal/worklist Vec因此被复制，且同一feedback可能被合并两次。

本轮TDD让collector只借用page requests注册GPU buffer并返回empty renderer output；原frame sideband继续作为唯一feedback owner，在collect阶段被`take`一次。公开neutral DTO不变。Render03/17仍须把GPU feedback buffer与page-request capacity做generation持久化，避免每帧重建buffer；所有VG feedback必须有单一producer/owner标识和duplicate counter。

## PERF-MVP-348：particle neutral fallback每帧创建七个GPU buffer

当persistent particle runtime owner没有instance时，neutral collector每帧clone `gpu_frame`，创建particles A/B、emitter params、counters、alive indices、indirect args、debug readback共7个buffer，构造多份临时`Vec<u32>/Vec<u8>`并为每binding格式化String；即使empty frame也创建最小非零buffer。真实backend路径已持有`ParticleGpuRuntimeOwnerHandle`和active bindings，neutral fallback反而是高频资源抖动源。

Render12/17与Plugin01应把neutral fallback也纳入per-device/viewport persistent owner，按capacity增长复用7类buffer，dirty range写入并共享static binding IDs；empty/no-particle frame不创建/写入资源。Bevy `RawBufferVec`/`BufferVec`只在capacity不足或label变化时重建GPU buffer，普通帧复用buffer并仅queue write，Zircon应采用同类契约。该改造涉及GPU lifetime与collector ABI，本轮不做局部兼容双路径。

## 验收要求

PERF-MVP-347按page/cluster/traversal/visbuffer records 0/1/1k/100k记录sideband clone bytes、feedback item counts、merge duplicates和external buffer creates：prepared readback deep clone=0、每item feedback次数=1、stable page-request buffer create=0。PERF-MVP-348按particles/emitters 0/1/1k/100k、stable/1% changed记录buffer create/destroy、CPU temp alloc、write bytes、binding String alloc和CPU/GPU p95：empty creates=0、stable creates=0、buffer count固定≤7、只写dirty range。phase summary另要求temporary Vec alloc=0并最终由static diagnostic metadata消除固定String。current-source Cargo、provider/collector/feedback parity、F2 trace与RenderDoc通过前，本批留在`pending.md`。
