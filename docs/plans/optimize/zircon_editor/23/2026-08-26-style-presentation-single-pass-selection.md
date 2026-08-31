---
title: Editor23 Style Presentation Single Pass Selection
category: zircon_editor
report_id: Editor23-style-presentation-single-pass-selection-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Style Presentation Single Pass Selection

## Scope

This slice removes intermediate token-entry projection and post-build selection scans from the
asset-editor style pane. BTree token order, TOML literal formatting, selected fields, declaration
labels, edit/delete enablement, and missing-selection behavior remain unchanged.

## Change

- Build token labels and selected-token metadata directly from the document token map in one pass,
  avoiding a full `LocalStyleTokenEntry` vector and its per-token name/literal ownership.
- Locate the selected declaration while its presentation labels are generated instead of running
  `position` and `get` before a second full label pass.
- Reserve output capacity from source collection lengths while retaining first-match semantics.

## Deterministic Performance Evidence

| 4,096 tokens, 64 pane builds per sample | Before | After |
|---|---:|---:|
| Intermediate token entries per sample | 262,144 | 0 |
| Token/entry visits per sample | 786,432 | 262,144 |
| Full token presentation passes per build | 3 | 1 |
| Selected-token owned copies per build | 2 | 2 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_STYLE_TOKEN_SINGLE_PASS_BENCH_V1`. Acceptance requires direct token presentation P95 to
be at least 20% below entry projection plus selection. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ar_style_token_presentation_preserves_order_and_selection` covers
  token ordering, literal formatting, selected/missing metadata, and first declaration match.
- `optimization_batch_20260826ar_style_presentation_avoids_token_entry_projection` rejects the
  intermediate token projection and post-build `position` scans.
- `optimization_batch_20260826ar_style_token_single_pass_p95` reports paired P50/P95 samples and
  enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation, preview
fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only converges
style-pane presentation traversal.
