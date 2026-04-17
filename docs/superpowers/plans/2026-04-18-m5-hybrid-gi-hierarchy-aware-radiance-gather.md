---
related_code:
  - zircon_scene/src/components.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry unified indirect ownership downshift or wider split-merge policy, or Hybrid GI more complete scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - user: 2026-04-18 continue M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-driven-probe-gather.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-rt-lighting-screen-probe-hierarchy.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_prefers_hierarchy_parent_probe_radiance
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Hierarchy-Aware Radiance Gather

**Goal:** 把 `Hybrid GI` 的 `parent_probe_id` 从 visibility frontier 继续下沉到 GPU radiance-cache update，让 parent/child hierarchy 真正改变 probe irradiance gather，而不是只影响 request/eviction。

**Non-Goal:** 本轮仍然不实现完整的 hierarchy-aware post-process resolve、多层 probe relocation、surface cache 或硬件 RT lighting。

## Delivered Slice

- `GpuResidentProbeInput` 和 `GpuPendingProbeInput` 现在都显式携带 `parent_probe_id`。
- `execute_prepare/probe_quantization.rs` 新增 `probe_parent_probe_id(...)`，把 extract 里的 hierarchy 关系编码到 renderer-local GPU 输入；`None` 会被稳定映射到固定 sentinel，而不是在 shader 里猜测缺省值。
- `resident_probe_inputs(...)` / `pending_probe_inputs(...)` 现在会把这份 parent id 一路写进 compute pass 的 storage buffer。
- `update_completion.wgsl` 的 `gathered_resident_rgb(...)` 新增 hierarchy-aware 权重提升：
  - 如果当前 probe 的 direct parent 正好是某个 resident probe，这个 resident probe 的 gather 权重会被额外提升
  - 如果某个 resident probe 的 `parent_probe_id` 正好指向当前 probe，也会得到较弱的 direct-child gather boost
- 这意味着：
  - pending child probe 的 radiance update 不再只看空间距离和 ray budget
  - hierarchy parent/child 关系会直接改变 GPU-produced `probe_irradiance_rgb`

## Why This Slice Exists

- 之前的 `Hybrid GI` 已经把 `parent_probe_id` 接进 visibility，能做最小的 parent/child frontier 切换。
- 但 GPU completion 仍然只按：
  - trace-region radiance
  - nearby resident probe distance gather
  来决定 probe update。
- 这会造成一条边界断裂：
  - planning 已经知道哪些 probe 是 hierarchy 里的 parent/child
  - 真正的 radiance-cache update 却仍把它们当成无关 probe
- 把 hierarchy 关系继续压到 GPU gather 后，`Hybrid GI` 才真正形成：
  - hierarchy-aware visibility/request
  - hierarchy-aware radiance update
  - runtime host 继续只缓存 GPU 真值

## Validation Summary

- `hybrid_gi_gpu_completion_readback_prefers_hierarchy_parent_probe_radiance`
  - 证明在场景位置、trace work、resident radiance 都相同的情况下，只切换 child probe 的 `parent_probe_id`，GPU readback 就会真实改变
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu`
  - 证明新的 input layout 与 hierarchy-aware gather 没有破坏既有 trace-region、scene-light、resident-history 回归
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 runtime host、visibility、GPU completion、resolve 主链仍然兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明新的 GPU input contract 与 shader layout 没有留下 crate 编译缺口

## Remaining Route

- 把 `parent_probe_id` 继续下沉到 post-process resolve，让 parent/child hierarchy 也能影响最终 probe screen-space 选择
- 把 direct parent/child boost 继续推进到更深层的 multi-level screen-probe hierarchy，而不是只停在 direct relationship
- 继续把 hierarchy-aware radiance gather 与 RT hybrid lighting continuation 合流成同一条 M5 主链
