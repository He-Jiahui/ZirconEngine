---
title: Editor retained-host apply and native-presenter shared-artifact protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-retained-host-apply-native-presenter-shared-artifact-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/{apply_presentation.rs,scoped_presentation.rs,shell_content_presentation.rs,workbench_window_projection.rs}`
- 4/4 root Rust files source-reviewed; native windows repeat full projection with fresh caches,
workbench and host own duplicate scene trees, fallback shell targeting happens after full shell build,
and scoped floating lookup rescans rows; M1 patches only changed Welcome dispatch rows, while child
module review and M0/M2-M6 dynamic/profile/power/visual acceptance remain pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to retained editor/native-window MVP work. Record source/per-window projection and
conversion builds, fresh caches, row visits/clones/bytes, duplicate scene bytes, allocations, damage,
main-thread CPU, latency, RSS and energy across window and node scale.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of fresh-cache presenter entry points, duplicate scene transfer trees, post-build scope
guessing and repeated floating presence scans after consumers use shared artifacts and exact indices.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own `HostPresentationArtifactSet`, shared main/native source segments and one target overlay per window.
Changed source work must be independent of native window count.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own persistent hit/damage models per window and row-patch replacement without rebuilding shared source
presentation artifacts.

## `docs/plans/zircon_editor/editor_layout/06-floating-windows-and-design-parity.md`

Own native window target overlays and committed pane/presenter row indices. Scoped updates must be
O(matches), not all-window scans per presenter.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Preserve exact scope and dirty generations through coalescing so shell-content target selection occurs
before presentation construction and does not fall back to a post-build pane-ID guess.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared immutable UI artifacts and per-window invalidation roots/overlays across runtime/editor
boundaries, aligned with Unreal persistent-window invalidation.

## Acceptance handoff

The owner handoff requires 4/4 root and all child-module post-change fingerprints, managed focused and
behavior tests, full W/window/node scale matrix, current-source WPR/power artifacts on D/E/F,
interaction/screenshots, multi-window RenderDoc parity, milestone commit and quantified WeCom
notification. Shared ledgers remain protected until then.
