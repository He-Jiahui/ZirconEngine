# Runtime83 Borrowed Localization Diagnostic Identity Dedup

- Date: 2026-08-31
- Parent plan: `docs/plans/optimize/zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md`
- Related record: `docs/plans/optimize/zircon_runtime/83/2026-08-25-localization-path-buffer.md`
- Status: implementation_complete; managed_validation_pending
- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Scope: `UiLocalizationTableCatalog` report validation only

## Problem

`validate_localization_report_against_catalog` already borrowed the selected locale table map,
but repeated dependencies still constructed an owned diagnostic for every missing reference before
sorting and deduplicating the result. Large documents can report the same path/key repeatedly, so
the diagnostic payload allocations scaled with dependency count rather than unique missing
references.

## Optimization

- Preflight keeps a borrowed identity tuple of `(path, key, table, fallback-present)` for each
  missing table or missing key.
- A duplicate identity returns `None` before cloning the path or formatting the diagnostic message.
- Valid keys never enter the identity set.
- Existing final sort/dedup remains in place, preserving deterministic ordering and diagnostic
  equality semantics.
- No catalog ownership, locale fallback policy, or public API changed.

## TDD And Verification

- RED: the new source contract failed because no emitted-diagnostic identity set existed.
- GREEN: the new contract and the existing Runtime83 path-buffer contract pass `5/5`.
- Rust regression `duplicate_missing_localization_references_emit_one_diagnostic` is active and
  verifies 50,000 repeated missing references produce one `missing_locale_key` diagnostic.
- `python -m compileall` and scoped `git diff --check` pass.
- The existing release benchmark now uses the same missing-key workload and emits
  `legacy_diagnostic_constructions=50000 optimized_diagnostic_constructions=1`.

## Deterministic Performance Evidence

At `DEPENDENCY_COUNT=50,000` repeated missing references:

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| diagnostic constructions | 50,000 | 1 | -99.998% |
| selected locale-map lookups | 50,000 | 1 | -99.998% |
| final diagnostics | 1 | 1 | behavior preserved |

The parent Runtime83 record retains the previously measured path-buffer and hoisted-lookup timing
evidence. Managed Windows Cargo validation is submitted in the next coordinated batch and is
required before integration or commit.

## Remaining Scope

This slice does not implement catalog artifacts, culture negotiation, fallback graphs, message
formatting, or product localization ownership. Those remain open in the Runtime202 review.
