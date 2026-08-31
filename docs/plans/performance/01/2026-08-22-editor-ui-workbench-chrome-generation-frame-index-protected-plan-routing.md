---
title: Editor workbench chrome generation-owned frame index protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-chrome-generation-frame-index-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection{.rs,/**}` -
8/8 Rust files source-reviewed; page/dock/activity frame derivation repeatedly scans and clones all
projected nodes per control/tab, while node models and derived geometry lack one exact generation; M1
now uses one borrowed index per multi-control group and removes production read-only row clones, but
M0/M2-M5 managed dynamic/profile/power/visual acceptance remains pending.

Do not add this module to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to `PERF-MVP-106`, `PERF-MVP-113`, `PERF-MVP-147` and `PERF-MVP-626`. Record node visits,
frame-index/projection/image/text builds, model-row and node clones, segment receipts, scoped/full
applies, main-thread CPU, allocations, latency, RSS and package energy.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of raw-node repeated frame queries, incomplete thread-local cache keys and duplicate
private chrome cache protocols after all consumers use generation-owned projection artifacts. Do not
retain a compatibility entry that reconstructs an index or scans nodes per control.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own immutable chrome segment artifacts and exact page/dock/menu/activity/status generations. Scene
assembly must clone shared artifact owners and must not derive geometry from flattened nodes.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own the committed control-frame/hit-test index consumed by pointer, close, drag and overflow routes.
Input dispatch must use the same geometry generation that paint accepted.

## `docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md`

Own persistent live tab collections, one linear arrangement pass and narrow selection/close/drag
patches, following Unreal `SDockingTabWell`; do not rebuild or rediscover all tab frames by string ID.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Coalesce exact chrome segment generations once per frame. Status-only, pointer-only, render-only and
single-dock changes must not escalate to unrelated page, menu, activity or dock projection.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Own the typed control-frame index and common `ChromeProjectionArtifact` contract. The index is created
with projection and invalidated by template/resource/text/model/geometry receipt, not paint.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the runtime-side immutable projection/frame artifact and resource-generation image handles so the
Editor does not maintain a second geometry or resource authority.

## Acceptance handoff

The owner handoff requires the 8/8 post-change fingerprint, managed focused and behavior tests, the
full T/N scale matrix, current-source WPR/power artifacts on D/E/F, interaction and screenshot checks,
RenderDoc parity for GPU content, milestone commit and quantified WeCom notification. Shared ledgers
remain protected until then.
