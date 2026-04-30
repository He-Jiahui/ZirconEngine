---
related_code:
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
plan_sources:
  - user: 2026-04-18 Hybrid GI 的 hierarchy-aware resolve / deeper screen-probe hierarchy / RT hybrid lighting continuation
  - user: 2026-04-18 Hybrid GI 的 hierarchy-aware resolve / deeper screen-probe hierarchy / RT hybrid lighting continuation，或者 Virtual Geometry 的 deeper unified indirect / cluster raster / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-rt-lighting-screen-probe-hierarchy.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-hierarchy-aware-radiance-gather.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_changes_when_resident_ancestor_is_reached_through_nonresident_hierarchy_gap
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_inherits_rt_lighting_tint_through_nonresident_hierarchy_gap
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Hierarchy-Gap Resolve And RT Inheritance

**Goal:** 继续把 `Hybrid GI` 的 resolve 侧往更完整的 screen-probe hierarchy / RT hybrid lighting 路线推进一层，让 post-process 不再只看 direct resident parent-child，也不再只消费 probe 自己命中的 trace region。

**Non-Goal:** 本轮仍然不实现完整的 screen-probe relocation、probe hierarchy page-table、surface cache、hardware RT scene query 或真正的 Lumen-like scene representation。

## Delivered Slice

- `encode_hybrid_gi_probes(...)` 现在除了编码：
  - `screen_uv_and_radius`
  - `irradiance_and_intensity`
  之外，还会为每个 resident probe 额外编码一条 `hierarchy_rt_lighting_rgb_and_weight`。
- `hybrid_gi_hierarchy_resolve_weight(...)` 不再只看 direct resident parent/child：
  - resident ancestor depth 现在会沿完整 `parent_probe_id` chain 继续向上遍历，即使中间存在 nonresident hierarchy gap 也不会丢掉 lineage
  - resident descendant attenuation 现在也会递归统计所有 descendant，而不是只看 direct child
- 新增 `hybrid_gi_hierarchy_rt_lighting/mod.rs`：
  - 它会沿 target probe 的 ancestor chain 收集 scheduled trace region 的 `rt_lighting_rgb`
  - ancestor 与 trace region 的 world-space overlap 会被转成 lineage-weighted inherited RT tint
  - lineage 越深，继承权重越小，但不会因为中间 probe 当前 nonresident 就直接中断
- `GpuHybridGiProbe` 与 `post_process.wgsl` 现在会把这条 inherited RT tint 作为 probe-local RT-lighting baseline，再和像素级 scheduled trace-region support 合并。

## Why This Slice Exists

- 之前 resolve 侧已经有两条基础能力：
  - direct resident parent/child hierarchy 会改变 overlapping probe 的最终权重
  - trace region 显式 `rt_lighting_rgb` 会在 resolve 时改变 probe irradiance tint
- 但这两条能力都还停在“单层关系”：
  - hierarchy 一旦跨过 nonresident 中间层，resolve 就退化回 flat probes
  - RT tint 只会作用于 probe 自己直接吃到的 trace region，不会顺着 hierarchy 继续向 child probe 传播
- 对 M5 的目标来说，这个断层太明显：
  - visibility / GPU gather 已经开始知道 hierarchy
  - resolve 却还只能做 direct link
  - RT tint 也还只是 local trace-region override，而不是 hierarchy-aware hybrid lighting continuation
- 把这一层补上后，当前架构已经具备：
  - hierarchy-aware visibility frontier
  - hierarchy-aware GPU radiance gather
  - hierarchy-gap-aware resolve weighting
  - ancestor-derived RT tint inheritance

## Validation Summary

- `hybrid_gi_resolve_changes_when_resident_ancestor_is_reached_through_nonresident_hierarchy_gap`
  - 证明 resident child probe 即使通过 nonresident hierarchy gap 才连到 resident ancestor，resolve 结果也会真实变化，而不再退化成 flat mixing
- `hybrid_gi_resolve_inherits_rt_lighting_tint_through_nonresident_hierarchy_gap`
  - 证明当 scheduled trace region 只直接覆盖 ancestor probe 时，child probe 现在也会沿 hierarchy chain 继承更暖的 RT tint
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render`
  - 证明 inherited RT tint、新的 probe buffer layout 与 hierarchy-gap resolve 没有破坏既有 GI resolve 行为
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu`
  - 证明 resolve 侧扩展没有反向破坏 GPU completion / radiance gather 主链
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 visibility、runtime host、GPU completion 与 resolve 仍然闭合
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明新的 probe buffer layout 与 shader/resource contract 没有留下 crate 编译缺口

## Remaining Route

- 把当前 ancestor-derived inherited RT tint 继续推进到更完整的 scene-driven screen-probe hierarchy，而不是只停在 ancestor chain 的 resolve 预编码
- 把 RT hybrid lighting 从 trace-region tint inheritance 继续推进到更真实的 screen-probe gather / scene representation / RT hybrid lighting continuation
- 如果切回 `Virtual Geometry`，下一条主链仍然是 visibility-owned unified indirect / deeper cluster raster / residency-manager cascade
