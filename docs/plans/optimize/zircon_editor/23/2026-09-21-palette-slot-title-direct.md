---
title: Editor Palette Slot Title Direct
category: zircon_editor
report_id: Editor897-palette-slot-title-direct-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor897 Palette Slot Title Direct

## Finding and optimization

Asset Editor palette-drop slot labels previously formatted every ASCII word
into a separate `String`, collected those strings into a vector, and then
joined them into the final title. The title-case owner now lives in a narrow
`resolution/title_case.rs` module instead of adding more behavior to the
near-1,000-line drag-resolution owner. It appends ASCII case changes and
single-space delimiters to one input-bounded output. If the input contains no
ASCII word, the original string remains the exact fallback. Empty, repeated
or Unicode delimiters, mixed case, and embedded NULs preserve legacy text.
The higher-level drop-choice logic and UI ownership have not moved.

## TDD and deterministic evidence

The combined Editor897/Runtime878 contracts were RED `2/7` with three missing
lower-test/module errors, then GREEN `7/7`. Eleven distinct labels in a lower
regression compare the retired word-join behavior; a 4,096-label/16-word
deterministic model eliminates `65536` temporary word strings and their
reference slots while preserving the one required title. Ignored
`EDITOR897_PALETTE_SLOT_TITLE_DIRECT_BENCH_V1` records 101 alternating Release
p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired path; it
has not been run.

The pair plus preceding animation/export/asset-palette contracts pass `31/31`.
Exact Rustfmt and scoped diff checks pass. No per-task Cargo run was performed.
The current Runtime/Editor/App source was submitted together in async v25 as
PID `28484` at `2026-09-21T23:28:01.8534392+08:00`, without monitoring.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs` | `97C5B82A1A70C079191483FF59FEB80FEE3BD671537EE5D4D2BB96F959ED7AEE` |
| `zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution/title_case.rs` | `605ED0417826AEB898A899567A8EF2A834D0814AA66D6148B606CE921B6FFD4A` |
| `zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution/title_case/tests.rs` | `3C3A5DE858E9714B18607CDF809C18B09A46E1CF5111EA5125A8EF08CE403E13` |
| `tools/tests/test_editor897_palette_slot_title_direct_performance_contract.py` | `FF897A508710604695DB8050E8A8387CA9491B8DC9855F74379801098439DF27` |

## Acceptance boundary

The deterministic intermediate-elimination target is met. A later bounded
v25 reconciliation confirms the managed Editor development build exited `0`
on the source submitted before Editor898. Runtime's separate package failed
foreign `compile_input_changed` and App had not ended in the snapshot. Rust
lower tests, ignored Release percentiles, allocator measurement, and Asset
Editor palette product p50/p95/p99 remain pending. The `-SkipTest` build
cannot close them.
