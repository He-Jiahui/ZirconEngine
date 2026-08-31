# Runtime83 Single-Allocation Missing-Key Diagnostic Message

- Date: 2026-08-31
- Parent plan: `docs/plans/optimize/zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md`
- Status: implementation_complete; managed_validation_pending
- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`

## Problem

Every emitted `missing_locale_key` diagnostic with a source URI first allocated an intermediate
`" in {source_uri}"` string and then allocated the final message containing that temporary value.
The intermediate string was immediately dropped.

## Optimization

The missing-key branch now selects one final format string based on whether `source_uri` exists.
Both branches allocate exactly the owned final diagnostic message; neither builds an intermediate
suffix. Diagnostic text and optional-source behavior are unchanged.

The release evidence keeps a self-contained legacy helper so it compiles independently of the new
production helper signature and continues measuring the old two-allocation kernel.

## TDD And Verification

- RED: both new source/evidence contracts failed against the intermediate suffix allocation.
- GREEN: the message-allocation, borrowed diagnostic dedup, and prior path-buffer contracts pass
  `7/7` together.
- Python bytecode compilation and scoped `git diff --check` pass.
- Managed Windows Cargo and release benchmark evidence remain pending in the coordinated batch.

## Deterministic Performance Evidence

For 50,000 unique missing keys with registered source URIs:

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| message-related `String` allocations | 100,000 | 50,000 | -50.000% |
| final diagnostic messages | 50,000 | 50,000 | behavior preserved |

The model counts only the intermediate suffix and final message allocations. It does not claim to
measure all diagnostic allocations or end-to-end catalog latency.

## Remaining Scope

Catalog artifacts, fallback graphs, culture switching, message formatting, and product authority
remain open in the Runtime202 current-source review.
