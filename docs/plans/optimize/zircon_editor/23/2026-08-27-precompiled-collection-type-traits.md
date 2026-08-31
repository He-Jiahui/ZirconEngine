---
title: Editor23 Precompiled Collection Type Traits
category: zircon_editor
report_id: Editor23-precompiled-collection-type-traits-2026-08-27
date: 2026-08-27
session_id: root-editor23-precompiled-collection-type-traits-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Precompiled Collection Type Traits

## Scope

This slice removes repeated declared-type normalization and classification from retained UI
`ArrayField` and `MapField` row projection. It preserves component-role precedence, generic,
numeric, reference, boolean, color and vector validation, warning/error text, row order, action
payloads, value conversion, public APIs, and empty-collection behavior.

## Change

- `CollectionTypeTraits` classifies one borrowed declared type with ASCII case-insensitive byte
  matching and stores the private role/validation predicates in a `Copy` value.
- Array projection constructs one element trait set before row iteration.
- Map projection constructs one key and one value trait set before row iteration.
- Role and validation functions consume the precompiled traits instead of allocating and scanning a
  lowercase string for every row.
- A Rust regression covers mixed-case string keys, asset-reference roles and missing-reference
  warnings. A Python contract prevents classification from returning to either row closure.

## Deterministic Performance Evidence

The independent release model projects 32,768 representative map rows. Each baseline row performs
four declared-type classifications for key validation, value validation, key role and value role.
The optimized model compiles the key/value traits once and retains the same four row-level dispatch
decisions. Each run uses 21 paired samples with alternating baseline/optimized order.

| Evidence | Per-row normalization | Precompiled traits | Result |
| --- | ---: | ---: | ---: |
| Classification checksum | 1,437,110 | 1,437,110 | identical |
| Total allocations | 131,072 | 0 | 131,072 fewer; 100% reduction |
| Run 1 P50 | 132.9314 ms | 0.3395 ms | 99.745% faster |
| Run 1 P95 | 206.8299 ms | 0.4099 ms | 99.802% faster |
| Run 2 P50 | 170.9041 ms | 0.3492 ms | 99.796% faster |
| Run 2 P95 | 245.1306 ms | 0.6214 ms | 99.747% faster |
| Run 3 P50 | 126.5068 ms | 0.3395 ms | 99.732% faster |
| Run 3 P95 | 216.2414 ms | 0.4861 ms | 99.775% faster |

This model isolates declared-type classification and dispatch; it does not represent the remaining
row DTO, display text, action payload, or full pane rendering cost. The managed gate requires the
exact checksum and allocation counts, 100% modeled allocation reduction, and at least 95% P50 and
P95 improvement for this isolated path.

## Acceptance

- TDD RED observed exact 4/4 failures for missing traits, per-array compilation, per-map compilation
  and the mixed-case Rust regression.
- `tools.tests.test_editor23_precompiled_collection_type_traits_performance_contract` passes 4/4
  locally.
- Exact production/model formatting, Python compilation, PowerShell parsing, three calibrated model
  runs, and scoped diff checks pass before the candidate is frozen.
- Existing collection-field tests and the new regression are submitted in one coordinator Cargo
  batch alongside source contracts, formatting, the model and scoped diff checks.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Editor23 still owns the authoritative UI document/session hard cuts, runtime/editor convergence,
binding and flow semantics, accessibility, theme/font atlas lifecycle, failure isolation, scalable
authoring and product qualification. This record only accepts collection type classification.
