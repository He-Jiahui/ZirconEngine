---
title: Editor retained shell pointer single release receipt protected routing
date: 2026-08-23
status: routing_requested_m0_static_validated
owner_record: 2026-08-23-editor-retained-shell-pointer-single-release-receipt-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/shell_pointer (8/8 reviewed): retained multi-window routing preserved;
single release receipt/committed snapshot and direct resize-move M0 statically validated;
projection generation, scale WPR/allocator/power and managed Rust tests pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own release dispatch/build counts, resize move storm, floating-window scale matrix, WPR/power and
final quantitative evidence.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one typed drag release receipt and one resize capture/release session authority.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own borrowing committed `WorkbenchLayout`/`WorkbenchViewModel` for pointer release and reject mixed
generation model/geometry resolution.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own changed-only typed drag target publication, resize move without discarded generic dispatch and
future evidence-driven spatial-index selection.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own committed pointer-layout/floating-projection generation and O(1) unchanged projection gate.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own pointer capture semantics and receipt reuse; generic dispatch must not force repeated target
resolution or model reconstruction in feature adapters.

## Acceptance handoff

Require focused contracts, managed Rust tests, `F = 0/1/8/64/256` scale evidence, D/E/F WPR,
allocator and power artifacts, drag/drop/resize/capture behavior parity, any required RenderDoc
pixel/draw parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.

Current static evidence: owner pre-M0 8/8 files, 1,355 lines, 49,354 bytes, manifest
`29255d2c78e9e8469ee4918c188f2118ab3c48930253c89a12748cdc7af989b4`; post-M0 8/8 files, 1,359
lines and 49,469 bytes. Drag Up route dispatches are `2 -> 1`; release layout/chrome/context/
command-lock/model builds each `1 -> 0`; resize Move dispatch is `1 -> 0`; resize Up is `2 -> 1`;
invalid setup capture is cancelled immediately. Focused RED 0/6 to GREEN 6/6, retained-host
contracts 76/76, broad performance contracts 255/255, profile Pester 3/3 plus 45/45, Rustfmt and
scoped diff check passed. Managed Cargo did not enter compilation because the coordinator reported
`unmanaged_artifacts_detected` for `F:\cargo-targets\zircon-engine`. No executable, timing, power,
WPR or RenderDoc acceptance is claimed.
