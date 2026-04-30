---
related_code:
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
plan_sources:
  - user: 2026-04-18 剩余更值得继续推进的主链是更完整的 screen-probe hierarchy / RT hybrid lighting
  - user: 2026-04-18 列出后续所有 tasks，把它们作为 todo，然后继续深入
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-hierarchy-gap-resolve-and-rt-inheritance.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-primary-lineage-gather-and-lineage-budgeting.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_inherits_farther_resident_ancestor_irradiance_beyond_nearest_resident_parent
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo check -p zircon_graphics --lib --offline --locked
  - cargo check -p zircon_asset --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Farther-Ancestor Resolve Irradiance Continuation

**Goal:** 把 `Hybrid GI` 的 hierarchy continuation 再往 resolve 侧推一层，让 resident child probe 在跨过 nearest resident parent 之后，仍能把 farther resident ancestor 的 irradiance continuation 带进最终 post-process GI resolve，而不是只在 GPU completion 或 RT tint 继承层体现层级信息。

## Delivered Slice

- `GpuHybridGiProbe` 新增了 `hierarchy_irradiance_rgb_and_weight` payload。
- post-process probe 编码阶段新增 `hybrid_gi_hierarchy_irradiance(...)`：
  - 沿 `parent_probe_id` chain 向上遍历 resident ancestors
  - 保留 nearest resident parent 现有 resolve 语义
  - 只把 “beyond-nearest” 的 resident ancestor 作为 farther-lineage irradiance continuation 编进 GPU payload
- `post_process.wgsl` 现在会先把 farther-ancestor irradiance continuation 混进 probe 本地 irradiance，再和已有的 hierarchy RT-lighting continuation 合并。
- 这样做之后，deeper screen-probe hierarchy 的 radiance continuation 不再只停在 GPU completion readback；resident probe 自身的最终屏幕 resolve 现在也会对更远 resident ancestor 的 radiance history 有真实响应。

## Why This Slice Exists

- 之前的 hierarchy 链已经覆盖：
  - nonresident hierarchy gap resolve
  - ancestor-derived RT tint inheritance
  - pending probe 的 multi-ancestor GPU completion continuation
  - primary-lineage-only gather 与 lineage-fair request budgeting
- 但 resolve 侧仍然有一个空洞：
  - resident child probe 的最终 GI color 只认本地 `irradiance_rgb`
  - 外加 scalar hierarchy weight
  - 外加 hierarchy RT-lighting tint
- 这意味着更远 resident ancestor 的 radiance 虽然已经能进入 pending update/readback，但对 resident probe 的最终 resolve 仍然没有独立的数据路径。
- 本切片补的就是这条缺口：让 deeper hierarchy 对最终屏幕 GI color 产生真实、可测量的变化。

## Validation Summary

- `hybrid_gi_resolve_inherits_farther_resident_ancestor_irradiance_beyond_nearest_resident_parent`
  - 证明 resident child probe 在跨过 nearest resident parent 后，会从 farther resident ancestor 继承更暖的 irradiance continuation，而不是只剩本地 neutral irradiance。
- `hybrid_gi_resolve_render`
  - 证明新的 farther-ancestor irradiance continuation 没有破坏现有 parent/child hierarchy resolve、nonresident gap resolve 与 RT tint inheritance。
- `hybrid_gi`
  - 证明 request -> runtime host -> GPU completion -> resolve 的主链仍然闭环。
- `visibility`
  - 证明这条 resolve-side 变更没有反向污染 visibility 层。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 post-process probe payload 扩展后的编译闭环仍然成立。

## Remaining Route

- `Hybrid GI`
  - 继续把 hierarchy-aware resolve weighting 做得更 scene-driven，而不只是新增 farther-lineage color continuation。
  - 继续补更完整的 screen-probe hierarchy RT hybrid lighting continuation，让 deeper lineage 对 trace tint / gather 组合产生更完整的影响。
  - 继续把 hierarchy-aware gather / resolve 的部分启发式进一步下沉到 runtime / GPU path，减少单帧 encode 侧特殊规则。
- `Virtual Geometry`
  - 继续推进 unified indirect ownership 下沉、deeper cluster raster consumption 与 residency-manager cascade。
