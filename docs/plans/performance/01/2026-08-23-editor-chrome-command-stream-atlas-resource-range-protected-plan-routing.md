---
title: Editor chrome command-stream atlas, resource and retained-range protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-chrome-command-stream-atlas-resource-range-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/chrome_command_stream/**`
- 42/42 Rust files source-reviewed. Invalidated frames traverse three command vectors; stable icons are
  scanned three times and clone about three resource keys per icon under the global atlas mutex; cached
  Runtime conversion still rebuilds owned commands/identifiers. Existing damage rejection, monotonic
  order, Arc pixels, generation-qualified bounded atlas and residency suppression are retained. M1-M6
  borrowed lookup/ranges/ownership/handles/CPU replay/profile/power acceptance remain pending. M1 has
  reduced stable atlas passes `3 -> 2` and stable key clones `3I -> 0` (focused GREEN 3/3; owned
  contracts GREEN 77/77), while M0/M2-M6 and dynamic acceptance remain open.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to MVP retained-host invalidated-frame and GPU submission work. Record command scans/moves,
Vec growth, rebuilt/reused ranges, icon key-clone bytes, atlas lock/admission/upload/eviction, resource
residency, CPU replay allocations, CPU/RSS/latency/context switches, WPR power and RenderDoc parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of triple transient command-vector ownership, flat full-stream rebuilds, duplicated command
String identifiers and parallel geometry implementations after retained ranges/handles are authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own editor scene generation to stable root/chrome/dock/overlay command ranges, damage-trigger policy,
window atlas mutation ownership and backend prepared-list reuse.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable prepared render-list arenas, canonical text/image handles, borrowed atlas/resource views
and one geometry/invalidation contract consumed by editor and runtime backends.

## `docs/plans/zircon_runtime/render/17-performance-and-profiling.md`

Own command/range/resource counters, wgpu upload/batch/timestamp markers and same-workload RenderDoc GPU
evidence. CPU/WPR evidence remains owned by the MVP performance plan.

## Acceptance handoff

The handoff requires 42/42 post-change fingerprints, focused and managed Rust behavior tests, the
command/icon/text/image/damage/backend matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc
draw/GPU/pixel/text parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.
