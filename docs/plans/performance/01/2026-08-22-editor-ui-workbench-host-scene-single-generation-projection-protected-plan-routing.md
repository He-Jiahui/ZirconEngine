---
title: Editor workbench host scene single-generation projection protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-host-scene-single-generation-projection-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection{.rs,/**}` - 2/2 Rust
files source-reviewed; Full apply projects all floating windows twice for main/native, scene segments
lack exact generations, and related chrome frame derivation repeatedly scans all nodes per tab;
M1/dynamic/profile/power/visual acceptance remains pending.

Do not add the module to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M1-M5 to `PERF-MVP-106`, `PERF-MVP-113`, `PERF-MVP-147` and `PERF-MVP-626`. Record
scene/segment generations, floating projection and conversion passes, wide row/refcount work,
control-frame node visits, scoped/full/native applies and OS calls.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of the uncached scene entry, duplicated main/native floating conversion and flat scene
assembly after all consumers use exact generation-owned artifacts. Do not retain a compatibility
entry that creates a fresh projection cache per call.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the host scene domain DAG, immutable segment receipts, main/native shared artifacts and scoped
apply cursors. Full fallback requires a recorded reason; unchanged or render-only work must not
project scene data.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own exact floating/dock/pane route identity and damage so pointer, drag, resize and focus changes can
patch narrow window state without scanning or cloning every floating row.

## `docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md`

Own the persistent dock/floating window model, main/native window identity, per-window geometry/DPI
state and live tab reuse. Shared content artifacts must not collapse independent native apply state.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Coalesce exact scene segment generations once per frame. Status, render, pointer and single-pane
changes must not escalate to an unrelated Full scene transaction.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Own a typed control-ID/frame index in the chrome projection owner and replace repeated formatted-ID
full-node scans. The index is invalidated by template/resource/text/geometry generation, not paint.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared immutable template node/frame artifacts and per-window invalidation/apply generations so
Editor does not maintain a second full-tree authority.

## Acceptance handoff

The owner handoff requires the 2/2 post-change fingerprint, managed focused tests, main/native shared
identity and behavior tests, the full scale matrix, current-source WPR/power artifacts on D/E/F,
F0/F4 interaction, screenshots, RenderDoc parity for GPU content, milestone commit and quantified
WeCom notification. Shared ledgers remain protected until then.
