---
title: Editor retained route intent single binding index protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-retained-route-intent-single-binding-index-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete route-intent portion with one concise `pending.md` entry:

`zircon_editor retained_host/route_intent (2/2 current Rust files): M0 single node binding index;
stable typed route handles, runtime topology ownership, WPR/power and behavior parity pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint. Detailed findings
remain in the owner report, not the protected ledgers.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own route probe/clone/copied-byte/topology counters and pointer storm WPR/power matrices.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own typed stable route handles and interaction replies without String/Vec clones.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own handled/raw-target fallback, generation coherence and one retained route identity.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own changed-node route publication and stale-generation rejection.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own atomic node/handler/route topology transactions and incremental hit-index integration.

## Acceptance handoff

The handoff requires 2/2 post-change fingerprints, focused and managed Rust tests, scale/storm WPR
artifacts on D/E/F, behavior/pixel parity, milestone commit and quantified WeCom notification.
Protected ledgers remain unchanged until then.

Current follow-on evidence after the document-tab native-receipt hard cutover: the dead document-tab
intent variant/accessor is removed; the owner remains 2/2 files, now 184 lines and 6,150 bytes with
manifest `d5f486e273d8976c34a85cef2ca7b3c135ce74a142e470105154765e7a66677b`. Dynamic acceptance is
still pending.

Current follow-on evidence after the drawer-header native-receipt hard cutover: the dead
drawer-header intent variant/accessor is removed; the owner remains 2/2 files, now 173 lines and
5,707 bytes with manifest
`867198883baf866b256b0d2bcb5bc3cf7d670a7485bae18157e2fe0ae2985adb`. Broad static performance
contracts are 199/199; dynamic acceptance remains pending.
