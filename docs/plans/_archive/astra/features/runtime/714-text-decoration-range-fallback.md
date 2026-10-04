---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-28-editable-text-decoration-lazy-source-map.md
related_records:
  - docs/plans/astra/features/runtime/661-text-image-owner-and-contract-repair.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
implementation_files:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/decoration_performance_tests.rs
tests:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/decoration_performance_tests.rs
  - tools/tests/test_runtime_interface03_text_decoration_line_range_performance_contract.py
  - tools/tests/test_runtime_ui_text_decoration_source_map_pressure.py
---

# Runtime text-decoration range fast path and safe DTO fallback

## Scope

The editable-text decoration path now retains the lazy source-map optimization for the
normal resolved-layout order while explicitly handling interface DTOs whose line source
ranges are not monotonic. A one-time order check selects the binary line-range probe for
normal shaped output; malformed or foreign DTOs use a linear intersection pass so a
decoration cannot be silently dropped. Declaration order, source order, bidi mapping,
IME clause output, and caret behavior remain unchanged.

## Implementation

- `TextDecorationLineSourceMaps` records whether line source ranges have monotonic starts
  and ends when the transient cache is created.
- Ordered layouts use `partition_point` to visit only the candidate line interval and
  reuse one `UiTextLineSourceMap` per touched line across selection and IME decorations.
- Unordered layouts scan the retained line slice for intersections and call the same
  source-map insertion helper; no duplicate map or alternate geometry authority is added.
- The lower regression swaps two line records and verifies that the matching decoration is
  still emitted exactly once, while the existing binary-intersection oracle remains intact.
- Editable text now passes the same transient map cache from range-decoration projection into
  caret projection. A focused regression proves that selection, IME underline, and caret on one
  line initialize one `UiTextLineSourceMap`, without changing declaration order or caret geometry.
- Range projection now calls the cache's `for_line` accessor after the ordered interval or
  unordered DTO branch has already established intersection. This removes one redundant
  source-range predicate per touched-line/decoration pair without weakening the explicit
  unordered fallback check.

## Complexity and acceptance boundary

For the normal shaped layout, source-map construction remains proportional to touched
lines and the range probe remains logarithmic in line count after the one-time monotonicity
check. The defensive DTO path is `O(L)` per decoration by design. This is a correctness
fallback, not a product-timing claim; it avoids pretending that an undeclared ordering
contract exists at the interface boundary.

## Local verification

- Runtime text-decoration/source-map contracts: `9/9` passed in one focused invocation,
  including the updated `4/4` source-map pressure contract and the binary/fallback/text-box
  companions.
- The deterministic pressure model records `3 -> 0` redundant touched-line predicates for
  the one-line/three-decoration case; it remains algorithm-work evidence, not product timing.
- The latest combined current-source Runtime/Editor batch loaded `60` modules and passed
  `212/212` tests in `0.144s`, with zero failures, errors, or skips.
- The combined Runtime/Editor performance-plus-pressure loader loaded `536` modules and
  passed `1993/1993` tests in `8.398s` on the current source snapshot (single-invocation
  loader; an earlier equivalent run measured `12.048s`).
- Scoped `rustfmt --edition 2021 --check --config skip_children=true,reorder_imports=false`
  and `git diff --check` pass for the touched text-geometry source and contract.
- Tooling production implementation was not changed; the source-contract test was updated
  only to lock the direct touched-line lookup invariant.

## Follow-up source receipt (2026-09-18)

The append helper's intersection callers now share the same lazy map lookup without a second
range predicate. The updated source contract is RED before the change and GREEN after it; the
focused Python module passes `4/4`. Current source fingerprints are:

| File | SHA-256 |
| --- | --- |
| `zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs` | `72AB17B4BF8C4609509135A509169C25CD38BF1C12A671EA31C4140D11258ABA` |
| `tools/performance/runtime/runtime_ui_text_decoration_source_map_pressure.py` | `0D4A841BAD4ACB46D3A87CB055041488937068419F56C5A867772F0B30B10332` |
| `tools/tests/test_runtime_ui_text_decoration_source_map_pressure.py` | `09DF5B0129E24406A34DE2F4BA065C623D911A7C84C96D20020EB1A4C423DB4D` |

This is a source/model receipt only. Managed Cargo, lower Rust regressions, Release markers,
allocator counts, and Editor text-edit p50/p95/p99/RSS evidence remain pending.

Managed Windows Cargo compilation, Release allocation/time measurements, and text-edit
input-to-present p50/p95/p99 evidence remain pending the owner-attributed asynchronous
validation lane. This record therefore stays `implemented_pending_validation`.
