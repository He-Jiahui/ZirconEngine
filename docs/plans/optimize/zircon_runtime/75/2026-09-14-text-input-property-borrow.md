---
title: Runtime75 Text Input Property Borrow
category: zircon_runtime
report_id: Runtime75-text-input-property-borrow-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Text Input Property Borrow

## Scope

TextInput validation runs on change/commit/blur paths. It formerly allocated a candidate-property
vector on every validation, cloned the selected text before validation, and allocated a mirror
property vector on every value event. The candidate and mirror rules are finite descriptor-derived
sets, so this Runtime75 slice represents them as static ordered slices and borrows text until
validation completes.

## Implementation

- Replaced dynamic candidate de-duplication with static slices for the exact `query`,
  `value_text`, `text`, and `value` priority permutations.
- `validation_text` now borrows the selected state/default string and returns a static empty fallback
  when no textual value exists.
- Text-input mirror targets now use static empty/single-property slices while retaining the existing
  query, `value_text → value`, and `value → value_text` rules.
- Added a lower candidate/mirror-order regression and ignored
  `RUNTIME767_TEXT_INPUT_PROPERTY_BORROW_BENCH_V1` Release marker.

## Deterministic work boundary

For each validation event, candidate list allocation and selected-text cloning are removed; mirror
target allocation is also removed. Validation timing, required/min/max grapheme checks, error text,
property priority, and mirror publication are unchanged. This is allocation-shape evidence only,
not allocator, CPU/RSS, or product input p50/p95/p99 evidence.

## Validation

- TDD source contract was RED before implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_text_input_property_borrow_performance_contract.py`.
- The combined Runtime/Editor command-palette, keyboard, menu, TreeView, TextInput, and adjacent
  capacity-contract invocation passed `69/69` in `0.577s` through `python -B -m unittest`.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
