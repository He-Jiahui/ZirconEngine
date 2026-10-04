---
title: Runtime82 Metadata Batch Borrowed Unchanged String
category: zircon_runtime
report_id: Runtime82-metadata-batch-borrowed-unchanged-string-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: local_release_gate_pending
---

# Runtime82 metadata batch borrowed unchanged String

## Hot path and bounded change

The editable-text property commit sends ten owned candidate values through
`mutate_tree_metadata_properties`, including the proposed body for a caret-only
edit. The old batch called `UiValue::to_toml` before checking whether each
candidate changed. For an unchanged million-character String, this allocated
and copied the complete body into a TOML String, compared it with the retained
TOML String, then discarded it without a body change, component projection, or
binding update.

The batch now compares an `UiValue::String` against an existing
`toml::Value::String` by borrowed `&str` first. Equality skips conversion. A
changed String still takes the old mutation, previous-value, dirty, and
binding path; the known-unequal String pair does not need a second full-body
comparison after `to_toml`. Other `UiValue` variants retain the old conversion
and comparison, including Color/Enum string-like values, Array/Map, Null, and
TOML Datetime. A TOML Datetime and an `UiValue::String` remain different TOML
variants even if their display text matches.

The change removes one transient full-body allocation and copy from each
unchanged String candidate. It still compares the complete body and still
needs one owned candidate from the calling transaction. It does not remove
Surface state materialization, document String editing, changed-value
projection, renderer extraction, or full product O(N) behavior.

## Behavioral and local Release evidence

The focused regression module verifies a caret-only Surface commit preserves
the body, text-document epoch, and text-layout revision while emitting only
small-field binding updates. Changed text still advances the epoch and emits
its previous body in the reflected binding receipt. Direct batch tests cover
unchanged/changed strings, missing nodes/metadata, repeated properties,
Datetime and other typed variants, dirty flags, and equality with a test-local
copy of the original HEAD batch function.

The ignored `runtime82_metadata_batch_borrowed_unchanged_string_release_profile`
compares the original batch and new batch on separate retained trees with the
same ten editable properties. Both receive independently owned proposed
values constructed **outside** the measured interval. Each of five warmups
and 31 measured pairs changes the small caret offset and leaves the 1K, 10K,
or 1M ASCII body unchanged. The call order alternates old/new by pair, and
every pair checks equal batch receipts, one caret change, and equal tree state.
The marker `RUNTIME82_METADATA_BATCH_UNCHANGED_STRING_V1` prints raw
nanosecond samples, nearest-rank p50/p95/p99, and environment identity. The
local 1M p95 gate requires the new batch to take at most 80% of the old batch.
This measures only metadata batch mutation, not the construction of owned
candidate values or a complete input-to-present edit.

Static source checks do not establish behavior or latency acceptance. Managed
Windows Runtime lib tests and ignored Release comparison remain pending under
the grouped batch. Runtime82 `RTE-GATE-016` and `RTE-GATE-047` remain open
pending frozen product budgets, allocation/RSS and input-to-present samples,
and matched Unreal evidence.
