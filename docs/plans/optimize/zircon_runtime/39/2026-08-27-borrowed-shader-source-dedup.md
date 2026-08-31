---
title: Runtime39 Borrowed Shader Source Deduplication
category: zircon_runtime
report_id: Runtime39-borrowed-shader-source-dedup-2026-08-27
date: 2026-08-27
session_id: root-runtime39-borrowed-shader-source-dedup-20260827
implementation_status: implementation_complete
validation_status: local_contract_passed_managed_validation_pending
---

# Runtime39 Borrowed Shader Source Deduplication

## Scope

This slice removes eager ownership from plugin shader-module source deduplication. Extension input
assembly now borrows each source and its identity fields while checking uniqueness, then clones only
the first source for each `(owner_id, import_path, content_hash)` key.

## Change

- Iterate over borrowed shader-module sources during duplicate admission.
- Store borrowed `&str` identity tuples in the private `HashSet`.
- Move `.cloned()` after the uniqueness filter so duplicate payloads are never cloned.
- Preserve registry order, first-seen ownership, the identity tuple, and the owned output type.

## Deterministic Performance Evidence

The standalone Rust model uses 16,384 candidate bindings, 256 unique bindings, four owned string
fields plus a shared source `Arc<str>`, and 15 alternating legacy/optimized samples.

| Shader source collection | Before | After | Reduction |
|---|---:|---:|---:|
| String clones | 114,688 | 1,024 | 99.107% |
| Cloned string bytes | 6,193,152 | 55,808 | 99.099% |
| P50 | 20,804,500 ns | 6,346,300 ns | 69.496% |
| P95 | 45,330,100 ns | 14,978,400 ns | 66.957% |

The model checks exact output equality and first-seen order. Its final checksum is `3,840`.

## Validation

- `python -m unittest tools.tests.test_runtime39_borrowed_shader_source_dedup_performance_contract`
  passes all three source contracts.
- Exact-file `rustfmt --edition 2021` passes.
- The standalone optimized Rust model compiles with `rustc --edition 2021 -C opt-level=3` and
  enforces at least 99% clone/byte reduction and at least 60% P50/P95 reduction.
- Existing Rust coverage
  `identical_feature_extension_shader_modules_are_collected_once` verifies duplicate behavior.
  Cargo execution remains pending through the session coordinator batch.

## Remaining Parent-plan Work

Runtime39 still owns the larger prefab compiler, instantiation, provenance, update/rebase, lifecycle,
streaming, network, and save integration work. This slice only removes duplicate shader source
ownership in runtime extension input assembly.
