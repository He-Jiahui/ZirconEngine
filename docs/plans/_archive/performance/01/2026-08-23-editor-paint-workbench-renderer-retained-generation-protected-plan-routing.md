---
title: Editor paint workbench renderer retained-generation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-paint-workbench-renderer-retained-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/paint_workbench_renderer{.rs,/**}`
- 104/104 Rust files current-source reviewed. Root damage still enters the immediate scene fan-out and
  stable command ranges are not retained. M1 routes a typed Welcome layout index through the existing
  generation traversal and eliminates up to twelve O(N) paint lookups (`12N -> 0`; focused GREEN 4/4;
  broad 115/120 with five unchanged failures). M0/M2-M6 owner-range, retained command/text, WPR/power
  and RenderDoc acceptance remain pending.

Do not add this module to `review.md` before M0-M6 pass on one current-source executable fingerprint.

## Performance plan ownership

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md` owns M0-M6 counters, scale,
managed behavior, WPR/power and RenderDoc/pixel acceptance. `docs/plans/performance/02-unreal-aligned-
engine-system-hard-cutover.md` owns removal of immediate whole-scene dispatch, paint-time string
control lookup and duplicate CPU/recording policy.

## Editor and Runtime UI ownership

`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md` must consume the typed Welcome
layout artifact and publish root/scene/pane owner ranges. `docs/plans/zircon_runtime/runtime/09-ui-
subsystem-architecture.md` owns shared generation, invalidation and prepared range contracts. The
project/recent workflow plan
`docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-
session-guard-focus-recent-recovery-product-integration-review.md` remains the authority for project
discovery, recent source refresh and async I/O; paint optimization must not duplicate that work.

## Acceptance handoff

The handoff requires post-change 104-file and focused M1 fingerprints, focused contract evidence,
managed Rust behavior/scale tests, owner/range and allocation counters, same-executable WPR/power
artifacts on D/E/F, RenderDoc/pixel/text parity, milestone commit and quantified WeCom notification.
Protected ledgers remain unchanged until then.
