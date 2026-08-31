---
title: Runtime Interface 02 Canonical Decode Performance Batch
category: zircon_runtime_interface
report_id: RuntimeInterface02-canonical-decode-batch-2026-08-24
date: 2026-08-24
session_id: optimize-runtime-interface02-canonical-decode-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Runtime Interface 02 Canonical Decode Performance Batch

## Scope

This batch advances the Runtime Interface 02 canonical serialization and bounded decode path. It
preserves canonical key order, recursive value normalization, duplicate-key rejection, wire data,
and public errors. It does not close the parent plan's spool budgets, schema registry, identity,
resource, reflection, or paged world-sync work.

## Task 1: Reuse Canonical JSON Collections

`canonicalize_value` previously consumed every JSON object into a temporary `Vec`, sorted it, and
built a second `Map`. It now canonicalizes child values in place and calls `Map::sort_keys()` before
returning the original map. Arrays are likewise canonicalized in their original vector. The
sorting call remains correct if Serde JSON's `preserve_order` feature is enabled and is a no-op for
the default ordered map.

The recursive walk now descends through `&mut Value` rather than replacing every child with null
through `mem::take` and then writing the whole value back. Transient aggregate collections created
per non-empty object are `2 -> 0`, and whole-value slot rewrites per child are `2 -> 0`. The release
gate uses a 1,024-key nested object over 21 alternating sample pairs and requires optimized P95 to
be at most 60% of the legacy rebuild path.

## Task 2: Single-lookup Binary Object Decode

Binary object materialization previously called `contains_key` and then `insert`, traversing the
object index twice for every accepted entry. It now uses the Serde JSON `Entry` API, preserving the
duplicate-key error while inserting unique values with one lookup. Duplicate-key error copy is
confined to the rejected path.

Index lookups per accepted entry are `2 -> 1`. The release gate builds a 10,000-key object over 21
alternating sample pairs and requires optimized P95 to be at most 80% of the legacy double-lookup
path.

## Validation

- TDD red state: the new source contract failed 3/3 before implementation.
- Follow-up true in-place recursion red state: 2/3 failed while the independent binary entry
  contract remained green.
- Source contract after implementation: 3/3 passed.
- `rustfmt`, Python bytecode compilation, and scoped whitespace validation: passed.
- Managed ticket `9a74b2c5b94a44a19b71c252d5dbe585` passed 62 serialization behavior tests,
  with one unrelated ignored test, plus both ignored release gates.
- Canonical object reuse measured `legacy_p95_ns=271,400 -> optimized_p95_ns=126,600`, a
  `53.353%` reduction; transient object collections remain `2 -> 0` and child slot rewrites
  remain `2 -> 0`.
- Binary object materialization measured `legacy_p95_ns=5,737,100 ->
  optimized_p95_ns=3,607,700`, a `37.116%` reduction; accepted-entry index lookups remain
  `2 -> 1`.
- No local Cargo lane or cargo dry-run was started, polled, or terminated.

## Remaining Parent-plan Work

Runtime Interface 02 still requires bounded canonical spool ownership, durable compatibility
artifacts, validated schema and identity catalogs, canonical resource locators, reflection
admission, and producer-bounded paged world synchronization.
