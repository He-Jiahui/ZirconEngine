---
related_code:
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/build_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/construct.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/build_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/construct.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry unified indirect ownership downshift or wider split-merge policy, or Hybrid GI more complete scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - user: 2026-04-18 continue M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-newly-resident-probe-hysteresis.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-rt-lighting-screen-probe-hierarchy.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_holds_resident_hybrid_gi_child_probe_one_frame_when_frontier_merges_back_to_parent
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
doc_type: milestone-detail
---

# M5 Hybrid GI Merge-Back Child Probe Hysteresis

**Goal:** 把 `Hybrid GI` 的 screen-probe hierarchy 再补一层 merge-back 稳定性，让 frontier 从 resident child probes 回退到 parent probe 的当帧里，仍然 resident 的 child probe 不会立刻进入 `evictable_probe_ids`。

**Non-Goal:** 本轮仍然不实现完整的 probe relocation、多层 hierarchy resolve、surface cache 或硬件 RT gather。

## Delivered Slice

- `VisibilityHistorySnapshot` 现在新增 `hybrid_gi_active_probe_ids`，把上一帧真正处在 active frontier 的 probe 集显式保留下来，而不再只记 `hybrid_gi_requested_probes`。
- `VisibilityContext::from_extract_with_history(...)` 会把当前帧 active probe frontier 写回这份 history，供下一帧 planning 做 merge-back 判断。
- `build_hybrid_gi_plan(...)` 新增了 `previous_active_probe_ids` 和 `merge_back_child_hold_protected_probe_ids`：
  - probe 当前仍 resident
  - probe 上一帧确实处于 active frontier
  - 当前帧这个 child probe 已不在 active frontier
  - 但它的 `parent_probe_id` 已重新回到当前帧 active frontier
- 满足这些条件时，这个 child probe 会在 merge-back 当帧被排除在 `evictable_probe_ids` 外；下一帧若 frontier 继续稳定停在 parent，则它会正常回到 evictable 集。

## Why This Slice Exists

- 之前的 `Hybrid GI` 已经有：
  - newly-resident probe hysteresis
  - `parent_probe_id` 驱动的 parent/child frontier refine
- 但它缺少反向切换的稳定层：
  - 当前帧如果因为 child 不再 resident、预算变化或 request 退化而重新落回 parent
  - 上一帧仍在 child frontier 的 resident child probe 会在同一帧被直接判成 evictable
- 这会让 hierarchy 只在“向 child refine”方向稳定，在“回退到 parent”方向仍然抖动。
- 补上这层 merge-back hold 后，`Hybrid GI` 的 parent/child frontier 两侧都具备最小一帧缓冲，为后续把 hierarchy 继续接进 GPU radiance gather / resolve 提供更稳定的 resident probe 集。

## Validation Summary

- `visibility_context_holds_resident_hybrid_gi_child_probe_one_frame_when_frontier_merges_back_to_parent`
  - 证明 frontier 从 resident children 回退到 parent 的第一帧里，仍 resident 的 child probe 不会立刻进入 `evictable_probe_ids`
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility`
  - 证明 active/request/evictable 的 hierarchy 规划没有破坏既有 Hybrid GI visibility 行为
- `cargo test -p zircon_graphics --offline --locked visibility`
  - 证明这层 Hybrid GI history 扩展没有破坏同一 visibility context 下的其它前处理行为
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 runtime host、GPU completion、resolve 与新的 hierarchy hysteresis 仍然一致

## Remaining Route

- 把 `hybrid_gi_active_probe_ids` 继续接进更深的 screen-probe hierarchy policy，而不是只停在一帧 merge-back hold
- 把 stable hierarchy frontier 继续下沉到 GPU radiance gather 与 resolve，而不是只停在 visibility/eviction 边界
- 继续朝 scene-driven screen-probe hierarchy / RT hybrid lighting 路线推进
