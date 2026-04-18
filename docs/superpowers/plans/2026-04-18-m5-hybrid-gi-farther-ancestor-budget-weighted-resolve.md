---
related_code:
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
plan_sources:
  - user: 2026-04-18 剩余更值得继续推进的主链是更完整的 screen-probe hierarchy / RT hybrid lighting
  - user: 2026-04-18 列出后续所有 tasks，把它们作为 todo，然后继续深入
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-farther-ancestor-resolve-irradiance-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_increases_child_intensity_when_farther_resident_ancestor_has_more_budget
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Farther-Ancestor Budget-Weighted Resolve

**Goal:** 把 `Hybrid GI` 的 hierarchy continuity 从“更远 ancestor 只追加颜色 continuation”再推进一层，让 farther resident ancestor 的 budget/support 也能真实改变 resident child probe 的最终 resolve 强度。

## Delivered Slice

- `hybrid_gi_hierarchy_resolve_weight(...)` 现在除了原有的：
  - resident parent depth specificity
  - resident child attenuation
  之外，还会统计 farther resident ancestors 的 budget support。
- 这条 support 只从 “超过 nearest resident parent 的更远 resident ancestor” 采样，避免把已稳定的 parent/child 语义重新混洗。
- farther ancestor 的 `ray_budget` 会先过 `hybrid_gi_budget_weight(...)`，再按 ancestor depth falloff 做 lineage continuation，最后乘到 resident child probe 的最终 resolve weight 上。
- 结果是：
  - deeper lineage 不再只会改变颜色 continuation
  - 同样的 hierarchy 颜色布局下，更高质量的 farther ancestor 现在也会提高 child probe 的最终 GI resolve 强度

## Why This Slice Exists

- 之前已经有 farther-ancestor irradiance continuation，但它解决的是“颜色从哪里来”。
- resolve 侧仍然缺少一条更完整的 hierarchy-aware weighting 路径：
  - 如果 farther ancestor 质量更高
  - 但本地 probe、nearest resident parent 和 irradiance 颜色都相同
  - 最终屏幕亮度仍然不会变化
- 这意味着 screen-probe hierarchy 还没有把 lineage quality/support 真正推进到最终 resolve strength。
- 本切片补的就是这条 gap：让 farther ancestor 的 budget/support 不再只停在 encode-side color continuation，而会进入最终 probe weight。

## Validation Summary

- `hybrid_gi_resolve_increases_child_intensity_when_farther_resident_ancestor_has_more_budget`
  - 证明在 hierarchy 颜色完全不变时，只提高 farther resident ancestor 的 `ray_budget` 也会让 child probe 覆盖区更亮。
- `hybrid_gi_resolve_render`
  - 证明这条新 weight continuation 没有破坏现有的 hierarchy-gap resolve、farther-ancestor irradiance continuation 和 RT tint 继承路径。

## Remaining Route

- 继续推进 `Hybrid GI` 的 hierarchy-aware resolve weighting / gather/runtime 闭环，减少 encode-side 启发式孤岛。
- 继续推进更完整的 RT hybrid lighting continuation，让 deeper lineage 不只影响 resolve weight，也影响 trace-support / traced-light mixing。
- `Virtual Geometry` 主线仍然保持为 deeper unified indirect / cluster raster / residency-manager cascade。
