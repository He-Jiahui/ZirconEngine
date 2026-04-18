---
related_code:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
- zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/frontier/refine_visible_probe_frontier.rs
- zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/ordering/hybrid_gi_probe_request_sort_key.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-18 剩余更值得继续推进的主链是更完整的 scene-driven screen-probe hierarchy / probe gather / RT hybrid lighting
  - user: 2026-04-18 continue M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-driven-probe-gather.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-gpu-hierarchy-completion-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_requests_nonresident_hybrid_gi_descendant_supported_by_trace_regions
  - cargo test -p zircon_graphics --offline --locked visibility_context_prefers_deeper_nonresident_hybrid_gi_descendant_when_trace_support_ties
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
doc_type: milestone-detail
---

# M5 Hybrid GI Descendant Request Frontier

**Goal:** 把 `Hybrid GI` 的 scene-driven screen-probe hierarchy 从“resident parent 只会请求 direct child”推进到“resident frontier 可以继续追 visible nonresident descendants”，让更深层 probe 真正进入 request/update 主链。

**Non-Goal:** 本轮仍然不实现完整的 hierarchy page table、probe relocation、surface cache，或多层 ancestor chain 的完整 GPU lineage 编码。

## Delivered Slice

- `build_hybrid_gi_plan(...)` 现在不再只从 active resident probe 收集 direct nonresident child。
- 对于 active resident probe，visibility planning 现在会沿 `children_by_parent` 继续向下遍历，收集整棵可见 descendant 子树里的 nonresident probe 候选。
- 这些 descendant 候选仍然走现有的 scene-driven 排序与 budget 裁剪：
  - scheduled trace-region local support
  - ancestor trace-lineage support
  - hierarchy depth specificity
  - ray budget
  - stable probe id tie-break
- 当 trace-region support 打平时，请求排序现在会继续偏向更深 descendant，而不是退回较浅的 child / sibling。
- 当 descendant 自己的 local support 较弱、但 parent/ancestor chain 明显落在 scheduled trace work 上时，请求排序现在也会把这条 ancestor lineage 继续记入 descendant score，而不再只按 probe 自己的 local overlap 截断。
- 这意味着：
  - active frontier 仍然可以保持 coarse resident parent
  - request frontier 却不再被 direct child 边界卡住
  - 更深层、对当前 trace work 更有价值的 descendant probe 现在可以直接进入 `requested_probe_ids`

## Why This Slice Exists

- 在前几轮里，`Hybrid GI` 已经具备：
  - hierarchy-aware active frontier
  - hierarchy-gap-aware GPU completion
  - hierarchy-gap-aware resolve
- 但 visibility request 侧还留着一个很明显的浅层限制：
  - resident parent 只能把 direct child 放进 request 集
  - 更深 descendant 即使更靠近 scheduled trace region，也必须等 direct child 先被请求
- 这会让 “scene-driven screen-probe hierarchy” 在最前置的 planning 层仍然不完整：
  - GPU completion 与 resolve 已经能跨 nonresident gap 继续工作
  - request frontier 却还只能在 direct child 处停住
- 把 descendant request frontier 接上之后，当前主链就变成：
  - visibility 能继续追 visible nonresident descendants
  - GPU completion 能跨 hierarchy gap 延续 ancestor gather / RT continuation
  - resolve 能保留 ancestor/descendant lineage 与 inherited RT tint

## Validation Summary

- `visibility_context_requests_nonresident_hybrid_gi_descendant_supported_by_trace_regions`
  - 证明当 direct child 与更深 descendant 同时可见时，request frontier 不再固定卡在 direct child，而会把 trace-support 更强的 descendant 直接选进 `requested_probe_ids`
- `visibility_context_prefers_deeper_nonresident_hybrid_gi_descendant_when_trace_support_ties`
  - 证明当 trace support 打平时，请求排序会继续偏向更深 descendant，而不是只按 ray budget 或稳定 id 退回较浅层 probe
- `visibility_context_prefers_nonresident_hybrid_gi_descendant_supported_by_ancestor_trace_lineage`
  - 证明当 descendant 自己的 local support 较弱、但其 ancestor chain 明显覆盖当前 scheduled trace region 时，请求排序会继续偏向这条 descendant lineage，而不是停在较浅 child
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility`
  - 证明 descendant request frontier、depth specificity 与 ancestor-trace-lineage scoring 没有破坏现有的 direct child request、merge-back child hysteresis、newly resident hysteresis 与 requested-probe history 规则
- `cargo test -p zircon_graphics --offline --locked visibility`
  - 证明更深 descendant request frontier 没有破坏整套 visibility 主链，也没有影响 Virtual Geometry 的 hierarchy tests
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 visibility request、runtime host、GPU completion、resolve 仍然保持闭环

## Remaining Route

- 把当前 descendant request frontier 继续推进到更完整的 scene-driven screen-probe hierarchy，例如 descendant-vs-ancestor lineage budgeting、跨多层 ancestor 的 request specificity，或更完整的 hierarchy-aware probe gather policy。
- 把 `Hybrid GI` 的 GPU completion 从“最近 resident ancestor continuation”继续推进到更完整的 multi-ancestor / RT hybrid lighting continuation。
- 如果切回 `Virtual Geometry`，下一条主链仍然是 visibility-owned unified indirect / deeper cluster raster / residency-manager cascade。
