---
title: Editor08 Palette Fixtures and Real Posting Capacity Record
category: zircon_editor
report_id: Editor08-palette-fixtures-real-posting-capacity-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: managed_validation_pending
performance_status: existing_gates_preserved_actual_build_pending
---

# Editor08 Palette Fixtures and Real Posting Capacity Record

## Scope and findings

This slice repairs the palette allocation source contract and benchmark fixtures, and adds test-only observation of the real locale posting builder. The focused diff is four tracked Rust files plus an existing ignored Python contract file. The Python file is a preexisting gitignored working-tree file, not a new tracked source: its verified before-bytes have SHA-256 `5d57e20955fa885f05504cd4941e48327469aa1adae87388e5b0b85e3733ae42` and its reviewed bytes have SHA-256 `6c7632e8cdcc62c33343e44957bf32ea0bfed08e060dfc76b8de5464e4ac7f59`.

The catalog source regression now scopes its inspection from `from_descriptors` through the following `locale_projection` method, retaining the positive reservation assertions and both negative `Vec::new()` guards. MRU fixtures now use canonical three-segment operation IDs, so the tests exercise missing-command omission and membership rather than failing ID parsing. The Python source contract follows the split Rust producer, keeps all four existing cases, preserves the historical model assertion, and binds the new actual-build producer and spy.

The Rust production diff is test-only capacity instrumentation behind `cfg(test)`; the read-only `palette.rs` module and catalog implementation are unchanged. The actual V2 producer builds the locale projection from 4,096 canonical descriptors carrying valid UTF-8 keywords (`ÉCOLE`, `ΟΣ`, and Chinese text). It feeds the projection's normalized search documents, in the same order, to a legacy `Vec::new()` collector with the same per-document byte deduplication. The test compares every optimized boxed posting's ordered indices to that baseline, verifies sorted indices, and observes each bucket's initial and final `Vec` capacity and each push capacity transition. The final vector capacities are captured immediately before boxing; this is not a post-box capacity or total-allocation measurement.

The normal 4,096-entry regression also runs the real catalog query for `zanzibar`, checks the sole returned ID `fixture.palette.command_4095`, and asserts one visited entry and one enablement evaluation. Existing Unicode query regressions remain in the wired palette test tree; this slice does not change query normalization or presentation behavior.

## Preserved measurements and gates

The historical `EDITOR812_PALETTE_LOCALE_POSTING_CAPACITY_BENCH_V1` remains unchanged in name and its 4,096-by-256 model still asserts 2,816 geometric growth events. Its output now labels it `dense_bucket_geometric_model`; this is a synthetic historical model, not a UTF-8 locale build or product observation.

The independent `EDITOR812_PALETTE_LOCALE_POSTING_ACTUAL_BUILD_BENCH_V2` is a deterministic capacity producer, not a timer. It reports the actual valid-UTF-8 build's populated buckets, posting indices, initial and final pre-box capacities, and observed legacy/optimized push-growth transitions. The assertions require exact per-bucket posting equivalence, zero optimized push growth, and more observed legacy transitions. It makes no CPU, P95, allocator-call, RSS, whole-query, or whole-lifecycle claim.

The existing `EDITOR629_PREALLOCATED_PALETTE_CATALOG_BENCH_V1` contract remains 32,768 descriptors over 17 alternating sample pairs with optimized nearest-rank P95 at or below 85% of the legacy helper P95. It measures two detached `Vec<String>` projection helpers, not the catalog constructor or locale/query lifecycle. The MRU release benchmark remains 100,000 catalog IDs, capacity 32, and 21 alternating pairs, with indexed membership nearest-rank P95 at or below 50% of the legacy scan; its P95 sample index is 19. Both release gates are preserved and remain unmeasured here.

## Source identity and review artifacts

The final cumulative patch is `.codex/outbox/reports/optimize-20261013-01a13451/editor-palette-contract.patch` (SHA-256 `528d29d2e42c3f699f5877baced4c81cf5ddfe0ef781ee1d4a28b9f900b44194`). Evidence is `.codex/outbox/reports/optimize-20261013-01a13451/editor-palette-contract-evidence.json` (SHA-256 `3ce82f031dabfadf63bcbc991fab5d295c829973a3c04551bad243d090e14798`). The root-owned exact guard is `.codex/outbox/reports/optimize-20261013-01a13451/editor-palette-contract-guard.json` (SHA-256 `72a4c13e0a8fe91edcabfcea2f0651b7a5597686d755cd9bdb4b119baf38bd3e`; passed with 0 errors and 1 warning for the newly ignored V2 case at line 75).

| Path | Before raw SHA-256 | Reviewed raw SHA-256 |
| --- | --- | --- |
| `zircon_editor/src/core/commands/palette/locale_projection.rs` | `d85af4af318c1c9a140fd671d54828d7a12ae2e3e6b7a7ba3e0bcf5a676b205a` | `c8a6a18620fd69c83b163ca491c0fded73e7ea262177683dc707699c08474604` |
| `zircon_editor/src/core/commands/palette/tests/locale_projection.rs` | `8493faf4b16d08fad12cc3362e6903f4ea5f163577269d15ff52e4c218dd13ef` | `b79d489da59ed1b241ae49280da5d78562a836bc6537a39152715ec9a3ca48a3` |
| `zircon_editor/src/core/commands/palette/tests/perf.rs` | `bb072c0603d21556c1a679938623dcc40cb7c318571c2c29d54a1433b76beb41` | `510291290c7bdf5ce9599198202e07e3556e1285eb77670388992423ad30969f` |
| `zircon_editor/src/core/commands/palette/tests/performance_tests.rs` | `c487f6e3185f42a5231d5e205dd9c4ecae32b3180501f66c82f6d186fc80cd4d` | `64d708a8fd897f00daf29e81cf538d70c8ae600aad656f1cdd1ca2e8ce3d32ec` |
| `tools/tests/test_editor_palette_locale_posting_capacity_performance_contract.py` (ignored baseline) | `5d57e20955fa885f05504cd4941e48327469aa1adae87388e5b0b85e3733ae42` | `6c7632e8cdcc62c33343e44957bf32ea0bfed08e060dfc76b8de5464e4ac7f59` |

The unchanged `zircon_editor/src/core/commands/palette.rs` was read-only and has the same before/after raw SHA-256 `96cd50b568a47af6361238fd4a6bf3c3345fc2a446ae6fe0f2edf7600f3007f1`. It wires the production locale projection and the performance test modules; `locale_projection.rs` wires its test module.

## Validation state and exact filters

Rust formatting and scoped `git diff --check` passed. The existing Python contract parsed with `python -B` AST-only validation and still contains four cases. These are static checks, not test execution. The exact guard warning marks the V2 case as ignored; the existing normal regressions remain enabled.

The three normal Rust filters to execute are:

- `core::commands::palette::optimization_batch_ir_editor629_tests::optimization_batch_ir_editor629_palette_catalog_reserves_descriptor_bounds`
- `core::commands::palette::performance_tests::editor08_palette_mru_catalog_indices_ignore_missing_commands_and_preserve_membership`
- `core::commands::palette::locale_projection::tests::optimization_batch_20260919_editor812_palette_posting_capacity_source_contract`

The four ignored Rust evidence filters are:

- `core::commands::palette::optimization_batch_ir_editor629_tests::optimization_batch_ir_editor629_palette_catalog_capacity_performance_evidence`
- `core::commands::palette::performance_tests::editor08_palette_mru_catalog_index_release_benchmark_evidence`
- `core::commands::palette::locale_projection::tests::optimization_batch_20260919_editor812_palette_posting_capacity_bench`
- `core::commands::palette::locale_projection::tests::optimization_batch_20261013_editor08_palette_locale_posting_actual_build_evidence_v2`

Run ignored cases with `--exact --ignored --nocapture --test-threads=1`. Run the four Python cases in `tools/tests/test_editor_palette_locale_posting_capacity_performance_contract.py`, class `EditorPaletteLocalePostingCapacityPerformanceContractTests`. Native Rust tests, the four Python unittests, and all ignored release producers remain pending; no observed P95 or performance pass is claimed.

A separate root-side handoff reconciliation reported the canonical batch09 intake missing. This review did not recheck that location or infer why it is missing or whether a queue accepted the work. The next batch10 handoff remains pending.
