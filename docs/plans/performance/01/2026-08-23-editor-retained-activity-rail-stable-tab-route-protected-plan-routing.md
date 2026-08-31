---
title: Editor retained activity rail stable tab route protected routing
date: 2026-08-23
status: routing_requested_m0_static_evidence_attached
owner_record: 2026-08-23-editor-retained-activity-rail-stable-tab-route-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/activity_rail_pointer (23/23 current Rust files): compact route M0;
shared workbench generation, native hit receipt, incremental topology, WPR/power pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own tab projection visits/allocations, pointer dispatches, route clones, topology rebuilds and
scale/storm/WPR/power matrices.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the generation-owned activity projection shared by layout, paint and input.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own explicit coordinate-space/native hit receipts and removal of the mirror dispatch.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own workbench topology generations and changed-projection publication.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own stable tab node identity and one authoritative hit path.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own atomic incremental node/handler/route topology updates.

## `docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`

Own compact typed drawer/tab command receipts.

## Acceptance handoff

Require post-change fingerprints, focused and managed Rust tests, D/E/F WPR artifacts, interaction
and pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.

## M0 evidence attached

- owner remains 23/23 Rust files; post-M0 527 lines, 21,232 bytes and manifest
  `cdd714a5ee1760bf64ce701009702c111380c653b461d3cc0f4d3e209052f814`;
- product hit dispatches per click: maximum 2 -> exactly 1 by focused static contract;
- route String payload clones: 2 per surface button and 2 per click -> 0;
- focused RED/GREEN: 0/4 -> 4/4; adjacent retained contracts: 17/17;
- broad performance contracts: 184/185, with only the unrelated missing `available_slots` owner;
- Rustfmt and diff checks pass; managed Cargo and dynamic acceptance remain blocked/pending.

## Drawer-command follow-on

The current owner is 22/22 files, 564 lines and 20,753 bytes with manifest
`180badbdaf06e333ef1702ce6e99fff87856e21caf731530b10ebf847553db75`. Its target accessor now
returns typed slot/view identities; the shared drawer command no longer scans template bindings,
parses the known slot, or clones the active drawer map. Broad static performance contracts are
199/199; the complete layout snapshot and dynamic acceptance remain pending.
