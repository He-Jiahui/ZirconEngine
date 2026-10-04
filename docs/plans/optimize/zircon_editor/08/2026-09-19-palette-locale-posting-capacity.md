---
title: Editor08 palette locale posting capacity
category: zircon_editor
report_id: Editor812-palette-locale-posting-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor812 · palette locale posting capacity

## Scope

`EditorCommandPaletteLocaleProjection::build` populated 256 byte-posting
vectors while projecting each localized seed. Every posting bucket started
empty and grew geometrically as entries were discovered, so a locale rebuild
could perform repeated reallocations in all frequently occurring byte buckets.

## Implementation

- Count each deduplicated document byte during the existing projection pass.
- Reserve every posting bucket with its exact count before the ordered fill pass.
- Re-scan the retained documents only to append the same entry indices in the
  same source order; preserve byte deduplication, rarest-posting selection,
  locale cache identity, and boxed posting output.
- Add a lower Rust source regression and the ignored
  `EDITOR812_PALETTE_LOCALE_POSTING_CAPACITY_BENCH_V1` Release marker.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old
`array::from_fn(|_| Vec::new())` collector and GREEN after the count-then-fill
shape was added. For a dense 4,096-seed/256-byte-bucket model, geometric
posting growth accounts for `2,816` bucket growth events; exact count
reservation models zero posting growth. The extra bounded document scan is
performed only when a locale projection is built and does not alter query-time
work. This is allocation-shape evidence only, not allocator, RSS, CPU, or
product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_palette_locale_posting_capacity_performance_contract.py`
  (`4/4`).
- Combined command/palette static batch (Editor812 plus existing palette,
  command, toolkit, settings-wiring, and product-interaction contracts) passes
  `25/25` with zero failures, errors, or skips.
- The strict non-tooling performance/pressure batch (tooling, export, and
  coordinator paths excluded) loads `681` files and passes `2639/2639` tests
  in `47.359s`, with zero failures, errors, or skips. The merged batch includes
  Runtime804/807/808 and Editor805-812.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- Lower Rust source regression and ignored Release marker are wired in
  `zircon_editor/src/core/commands/palette/locale_projection.rs`.
- Managed Cargo/Release and palette allocation/product percentile evidence
  remain pending; tooling production remains deferred for the later Rust
  migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/commands/palette/locale_projection.rs` | `A38B781C84EE9368E830D59AC309EA4CE4882A64B8A7940F78A9D11C019FB801` |
| `tools/tests/test_editor_palette_locale_posting_capacity_performance_contract.py` | `5D57E20955FA885F05504CD4941E48327469AA1ADAE87388E5B0B85E3733AE42` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies palette
allocation and product p50/p95/p99 evidence. No coordinator status is polled
by this session.
