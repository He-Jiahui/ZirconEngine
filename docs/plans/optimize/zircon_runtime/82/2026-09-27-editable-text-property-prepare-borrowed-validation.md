---
title: Runtime82 Editable Text Property Prepare Borrowed Validation
category: zircon_runtime
report_id: Runtime82-editable-text-property-prepare-borrowed-validation-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: local_release_gate_pending
---

# Runtime82 editable text property prepare borrowed validation

## Current-source problem

An InputField uses the same canonical value and text property. Its pre-edit
prepare constructed two full `UiValue::String` copies of the proposed body,
then dropped one because no supplemental property was needed. Admission also
converted the old TOML value to an owned `UiValue` twice to compare only the
variant, and `display_text()` copied the proposed String to compare it with
the state. For one million ASCII bytes, these are four avoidable million-byte
copies around the one retained proposed property. The existing
`ui_text.edit.property_value_clone_bytes` counter describes only the proposed
property projection; it is not a complete allocation counter.

## Implementation and semantics

The shared-property prepare owns one proposed value. Distinct canonical/text
properties, including NumberField's numeric `value` and string `value_text`,
retain independent values and validation. Admission compares TOML's borrowed
outer variant with the proposed `UiValue` variant. It preserves the old
`UiValue::from_toml` mapping, including Datetime to String, Array to Array,
and Table to Map. String-like display values compare by borrowed `&str`;
other typed values retain the old formatting path.

All kind, display, grapheme-state, and edit-intent validation happens before
building a prepared transaction or mutating Surface metadata, runtime style,
component state, dirty flags, document epoch, or binding reports. Explicit
editable component aliases still update only their selected property. The
ComboBox catalog is a typed selection control; this change does not alter its
selection model or treat its `value` and editable `value_text` as one value.

The focused regression module covers a real InputField Backspace dispatch and
typed edit receipt, prepare/discard atomicity, alternate editable aliases,
distinct NumberField value/text projection, kind mismatch, reserved property,
invalid grapheme boundary, and the old/new TOML/display kind contract.

## Local Release comparison and acceptance

The ignored `runtime82_property_prepare_legacy_vs_borrowed_release_profile`
test contains a test-local copy of HEAD's former InputField prepare,
validation, and property construction. It first checks prepared property and
supplemental arrays for equality. The Release-only comparison uses 1K, 10K,
and 1M ASCII body sizes, five warmups and 31 alternating old/new samples per
size. It prints all raw nanosecond samples and nearest-rank p50/p95/p99 under
`RUNTIME82_PROPERTY_PREPARE_BORROWED_V1`. The local 1M p95 gate requires the
new prepare to be at most 80% of the old p95. This measures prepare/discard
only; dispatch, document edit, Surface commit, render extraction, WGPU
presentation, and allocator/RSS are outside this comparator.

Source edits, static Rustfmt, and the test fixture do not establish a pass.
Managed Windows Runtime lib behavior and ignored Release runs remain pending;
the parent batch retains their receipts. The Runtime82 product gates
`RTE-GATE-016` and `RTE-GATE-047` remain open: they need frozen numeric budgets,
full input-to-present allocation/RSS evidence, and matched Unreal samples.

Remaining full-source costs include state materialization from Surface TOML,
String edit shifts, optional component event payloads, render text extraction,
and non-ASCII/cold document index rebuilds. This slice removes redundant
work in property prepare without changing the document authority model.
