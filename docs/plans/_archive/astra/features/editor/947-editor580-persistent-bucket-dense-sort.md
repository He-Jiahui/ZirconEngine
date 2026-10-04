---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-editor580-persistent-bucket-dense-sort.md
related_records:
  - docs/plans/astra/features/editor/948-editor580-command-row-deferred-text-clone.md
  - docs/plans/astra/features/runtime/890-runtime580-responsive-definition-move.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/index/persistent_buckets.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/index/persistent_buckets.rs
---

# Editor947 Editor580 Persistent-Bucket Dense Sort

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Persistent hit-test buckets | Collect cell entries directly into one vector and sort by packed cell key before balanced-tree construction, removing the intermediate `BTreeMap` allocation/traversal while preserving tree order and update semantics. | Behavior/source contracts cover empty buckets, lookup, balanced shape, and persistent updates. |
| 性能门禁 | 8,192 cells avoid the legacy map-then-vector path. | ignored marker `EDITOR580_PERSISTENT_BUCKET_DENSE_SORT_BENCH_V1` requires optimized P95 ≤70% of legacy; managed Editor Release receipt remains pending. |

- `persistent_buckets.rs` contains the Editor580 behavior contract and marker.
- No tooling changes; the combined Runtime580/Editor580 release gate remains pending.

### Grouped validation submission (2026-09-25)

Editor580 was included in the related `optimization_batch_gz` submission:
Runtime development PTY `81972`, Editor development PTY `21257`, Runtime02
Release PTY `20043`, and Editor Release PTY `50976`. All four wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.

The exact `optimization_batch_gy` replacement wave uses Runtime development PTY
`72314`, Editor development PTY `76797`, Runtime02 Release PTY `83368`, and
Editor Release PTY `99044`; these wrappers remain intentionally unpolled.
