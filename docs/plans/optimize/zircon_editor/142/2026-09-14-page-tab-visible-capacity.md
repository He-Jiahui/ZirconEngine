---
title: Editor142 page-tab visible-prefix capacity
category: zircon_editor
report_id: Editor762-page-tab-visible-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor142 page-tab visible-prefix capacity

## Scope

`visible_page_tab_indices` already computes a width-bounded visible prefix and
then restores an active tab when the prefix does not contain it. The scratch
`Vec<usize>` nevertheless started empty, so a wide page-tab strip paid
geometric growth as rows were admitted. This is a bounded follow-up to the
Editor142 workspace/document-tab capacity plan and the retained page-chrome
projection path; it does not change page-tab authority, overflow behavior, or
the broader layout-restore work.

## Implementation

- Added `page_tab_visible_capacity`, which clamps the width-derived cap both to
  the tab count and to the finite tab-lane minimum-slot count, while keeping one
  slot for the active-tab fallback. A very large page count can therefore no
  longer turn a wide but finite chrome lane into a full-count allocation.
- `visible_page_tab_indices` now reserves that upper bound before scanning.
  Width clipping, missing-row tolerance, tab order, active-tab replacement,
  deduplication, and overflow detection remain unchanged.
- Added a lower capacity/order regression and an ignored Release marker
  `EDITOR762_PAGE_TAB_CAPACITY_BENCH_V1` for the next owner-attributed batch.

## Deterministic pressure model

For a 4,096-row visible prefix, the legacy empty vector incurs geometric growth
while the reserved model admits the bounded prefix and reports zero growth
events. The marker records this allocation-shape comparison only; it is not a
CPU, RSS, or product p50/p95/p99 measurement.
The owner-attributed Release run should additionally apply the parent plan's
relative timing gate (`optimized_p95_ns <= legacy_p95_ns * 0.70`) before this
slice can be accepted as performance-complete.

## TDD and local evidence

- The source contract was intentionally RED before the helper and reservation
  existed, then GREEN at `4/4` after the implementation and test-module wiring.
- The lower Rust regression covers the bounded-capacity helper, finite-lane
  upper bound, active fallback, and ordered visible-prefix behavior. Scoped
  Rustfmt, Python compilation, and diff checks pass in the shared batch with
  the adjacent Editor slices.
- The current one-process merged Runtime/Editor performance-contract loader now
  includes the tightened finite-lane path, loads `517` modules, and passes
  `1852/1852` tests in `159.897s`; the focused seven-contract capacity batch
  passes `29/29` in `0.513s`. Wiki validation also passes `272/272` Markdown/navigation pages
  with one pre-existing metadata warning. These are local source/model
  receipts, not managed Cargo or product-latency evidence.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs` | `218AE54079AD49363080DD476294DC3421CA8F16003C367CF90FA0B2A71649E0` |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests/mod.rs` | `F8197862112F040BFD47654E7E1A1C9C70C343E81C1CFA7A55406AB65E8CE11F` |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests/capacity.rs` | `15D170F0723221DB558421E2D2A29E61B90E18E9E16613D2D26C84F1223F8E7E` |
| `tools/tests/test_editor_page_tab_capacity_performance_contract.py` | `1A66B5BCEC95BD8B8794D4EFF6426BA5FC8E8AD28357DD2D8626B5DD3F4C9804` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, visible-tab
parity, allocation behavior, and page-chrome p50/p95/p99. No standalone Cargo
command or coordinator status query is used here; tooling production work
remains deferred by request.
