---
title: Editor05 Borrowed Field Type Normalization
category: zircon_editor
report_id: Editor05-borrowed-field-type-normalization-2026-08-26
date: 2026-08-26
session_id: root-editor05-borrowed-field-type-normalization-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor05 Borrowed Field Type Normalization

## Scope

This slice removes transient lowercase strings from inspector field-editor lookup and registration.
It preserves exact contributed-type precedence, built-in alias families, ASCII case-insensitive
matching, suffix/substring inference, qualified plugin-type isolation, registration errors, editor
factories, public APIs, and the parent plan's fail-closed ownership boundary.

## Change

- Numeric and exact built-in aliases now use borrowed `eq_ignore_ascii_case` candidate scans.
- Color, enum, resource, asset, and curve inference now compares borrowed ASCII suffixes or windows.
- The lookup path still rejects fallback inference for qualified `.` and `::` type identities.
- A Rust regression covers mixed-case aliases, inferred suffixes, unknown types, and qualified misses.
  A Python source contract prevents lowercase allocations from returning to either hot path.

## Deterministic Performance Evidence

The independent release model performs 65,536 mixed field-type records. Each record executes eight
lookup normalizations and four registration alias checks; each run uses 21 paired samples with
alternating baseline/optimized order.

| Evidence | Lowercase baseline | Borrowed matching | Result |
| --- | ---: | ---: | ---: |
| Classification checksum | 2,424,832 | 2,424,832 | identical |
| Total allocations | 786,432 | 0 | 786,432 fewer; 100% reduction |
| Run 1 P50 | 76.9904 ms | 21.6306 ms | 71.905% faster |
| Run 1 P95 | 127.8585 ms | 34.1694 ms | 73.276% faster |
| Run 2 P50 | 86.3493 ms | 21.9984 ms | 74.524% faster |
| Run 2 P95 | 214.7910 ms | 84.7888 ms | 60.525% faster |
| Run 3 P50 | 110.0145 ms | 24.1406 ms | 78.057% faster |
| Run 3 P95 | 159.1083 ms | 65.3699 ms | 58.915% faster |

The managed gate requires the exact checksum and allocation counts, 100% modeled allocation
reduction, at least 60% P50 improvement, and at least 50% P95 improvement.

## Acceptance

- TDD RED observed exact 3/3 failures for both allocating functions and the missing Rust semantic
  regression.
- `tools.tests.test_editor05_borrowed_field_type_normalization_performance_contract` passes 3/3
  locally.
- Exact production/model formatting, model compilation, three paired model runs, Python compilation,
  PowerShell parsing, and scoped diff checks pass before the candidate is frozen.
- Existing field-editor behavior tests and the new regression are submitted together in one
  coordinator Cargo batch alongside source contracts, formatting, the model, and scoped diff checks.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Editor05 still owns Inspector data-correctness hard cuts, structured property paths, mixed-value
multi-selection, typed editors, executable customization, transaction semantics, virtualization,
failure isolation, and product-scale qualification. This record only accepts field-type matching.
