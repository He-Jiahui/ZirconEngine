---
related_code:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-17 continue M5
  - user: 2026-04-17 Hybrid GI still needs scene-driven radiance cache / probe gather / RT hybrid lighting
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-driven-probe-gather.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-cache-entry-residency-cascade.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_holds_newly_resident_hybrid_gi_probe_out_of_evictable_list_for_one_frame
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Newly-Resident Probe Hysteresis

**Goal:** 给 `Hybrid GI` 的 scene-driven radiance-cache residency 再补一层稳定机制，让“上一帧刚被请求、本帧刚成为 resident”的 probe 不会立刻又被标进 `evictable_probe_ids`，避免 radiance cache 进入一帧热插拔抖动。

**Non-Goal:** 本轮仍然不实现完整的 screen-probe hierarchy、probe relocation、surface cache 或 RT hybrid lighting。

## Delivered Slice

- `build_hybrid_gi_plan(...)` 现在会先恢复 `previous_requested_probe_ids`，把上一帧被请求过的 probe 当作短期保护集合。
- `dirty_requested_probe_ids` 继续只包含真正的新请求 probe，但 `evictable_probe_ids` 现在还会额外排除 `previous_requested_probe_ids`。
- 这意味着：
  - probe 在上一帧进入 request 集，本帧成为 resident 后，不会被立刻视为可驱逐
  - 到下一帧如果它仍然不 active，且也不再出现在 previous-request history 中，才会重新进入 evictable 集

## Why This Slice Exists

- scene-driven probe gather 与 cache-entry residency cascade 已经让 request/update/resident 主链开始跟随真实场景与 GPU cache truth。
- 但如果 newly resident probe 在成为 resident 的同一帧就被列入 `evictable_probe_ids`，runtime host 会出现“刚装入就准备驱逐”的短周期震荡。
- 这种抖动会让后续 radiance-cache gather、screen-probe hierarchy 和 RT hybrid lighting 都建立在不稳定的 resident 集上。
- 本轮先补一帧 hysteresis，确保 probe residency 至少能稳定跨过一个 update->resolve 周期。

## Validation Summary

- `visibility_context_holds_newly_resident_hybrid_gi_probe_out_of_evictable_list_for_one_frame`
  - 证明上一帧请求、本帧 resident 的 probe 不会立刻出现在 `evictable_probe_ids`，但下一帧失活后会正常回到 evictable 集
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility`
  - 证明 visibility planning 的 probe request / trace schedule / evictable policy 仍然一致
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明这条 hysteresis 不会破坏 Hybrid GI runtime、GPU completion 与 resolve 主链
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 visibility planning contract 扩展没有留下编译缺口

## Remaining Route

- 把短期 hysteresis 继续推进到更完整的 scene-driven radiance-cache placement / relocation policy
- 把 stable resident probe 集继续接进 screen-probe hierarchy 与更高阶 probe gather
- 继续朝 RT hybrid lighting / Lumen-like lighting integration 推进
