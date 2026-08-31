---
title: Editor23 Preview Expression UTF-8 Slice Parser
category: zircon_editor
report_id: Editor23-preview-expression-utf8-slice-parser-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Preview Expression UTF-8 Slice Parser

## Scope

This slice removes full `Vec<char>` projection from preview-mock expression parsing. Dot and
bracket syntax, single- and double-quoted segments, Unicode identifiers and whitespace, invalid
input rejection, and canonical path formatting remain unchanged.

## Change

- Locate ASCII path delimiters with byte offsets while advancing across content by each UTF-8
  character width, keeping every produced string slice on a valid boundary.
- Parse identifiers and bracket contents directly from source slices instead of rebuilding them
  from a projected character buffer.
- Move the first two parsed segments into node/property ownership instead of cloning them before
  moving the remaining segments.

## Deterministic Performance Evidence

| 4,096-character Unicode identifier, 512 references per sample | Before | After |
|---|---:|---:|
| Full character buffers per reference | 1 | 0 |
| Character-buffer entries per reference | full reference character count | 0 |
| Node/property segment clones | 2 | 0 |
| UTF-8 source copies before segment creation | 1 projection | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_PREVIEW_EXPRESSION_UTF8_SLICE_PARSER_BENCH_V1`. Acceptance requires UTF-8 slice parser
P95 to be at least 20% below character-vector parser P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ap_preview_expression_parser_preserves_utf8_segments` covers
  Unicode, whitespace, numeric and quoted brackets, canonical formatting, and invalid inputs.
- `optimization_batch_20260826ap_preview_expression_parser_avoids_char_projection` rejects
  character-vector/slice helpers while requiring byte views and UTF-8-width advancement.
- `optimization_batch_20260826ap_preview_expression_utf8_slice_parser_p95` reports paired P50/P95
  samples and enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed expression diagnostics, incremental
validation, preview fidelity, bindings, transactions, cook artifacts, and large-asset gates. This
slice only converges preview expression path parsing.
