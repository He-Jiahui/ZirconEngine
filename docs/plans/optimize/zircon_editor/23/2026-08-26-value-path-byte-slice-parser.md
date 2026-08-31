---
title: Editor23 Value Path Byte-slice Parser
category: zircon_editor
report_id: Editor23-value-path-byte-slice-parser-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Value Path Byte-slice Parser

## Scope

This slice removes whole-input character-vector and index-substring allocations from the shared UI
asset TOML value-path parser used by binding payloads and preview mock data. Leading, trailing, and
repeated dot handling; bracket index parsing and whitespace; Unicode keys; malformed input results;
owned key output; lookup; and mutation semantics remain unchanged. It does not claim the parent
plan's schema-backed binding or preview data milestones are complete.

## Change

- Scan the borrowed UTF-8 input bytes because all path delimiters are ASCII.
- Parse indices directly from borrowed source slices.
- Allocate only the key strings required by the returned segment model.
- Preserve the existing parser's permissive dot and post-index key behavior.

## Deterministic Performance Evidence

| 1,024 key/index pairs, 64 parses per sample | Before | After |
|---|---:|---:|
| Temporary whole-input character vectors per sample | 64 | 0 |
| Temporary index substring strings per sample | 65,536 | 0 |
| Total eliminated parser-only temporary allocations | 65,600 | 0 |
| Final owned key strings per sample | 65,536 | 65,536 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_VALUE_PATH_BYTE_SLICE_BENCH_V1`. Acceptance requires byte-slice parsing P95 to be at
least 30% below character-vector parsing. Exact Windows timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826ba_value_path_byte_slices_preserve_parser_semantics` compares the
  new parser with the prior algorithm across Unicode, whitespace, permissive-dot, and malformed
  inputs.
- `optimization_batch_20260826ba_value_path_parser_uses_utf8_byte_slices` requires direct borrowed
  slicing and rejects whole-input character and index-substring materialization.
- `optimization_batch_20260826ba_value_path_byte_slice_p95` reports paired release P50/P95 samples
  and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns lossless V2 editing, typed binding and preview schemas, transaction-safe save,
generation-qualified previews, input fidelity, and large-asset gates. This slice only converges the
shared value-path parser.
