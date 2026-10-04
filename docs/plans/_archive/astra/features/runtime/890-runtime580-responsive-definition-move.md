---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime580-responsive-definition-move.md
related_records:
  - docs/plans/astra/features/runtime/891-runtime580-cookie-context-preflight.md
  - docs/plans/astra/features/editor/947-editor580-persistent-bucket-dense-sort.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/layout/pass/responsive_mui/candidates.rs
tests:
  - zircon_runtime/src/ui/layout/pass/responsive_mui/candidates.rs
---

# Runtime890 Runtime580 Responsive Definition Move

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Responsive candidate patching | Compare borrowed definitions for invalidation, then move the owned definition into the index instead of cloning its component string and attribute map. | Behavior/source contracts preserve membership, thresholds, invalidation, and definition contents. |
| 性能门禁 | 4,096 definitions with 24 attributes avoid the legacy deep-copy path. | ignored marker `RUNTIME580_RESPONSIVE_DEFINITION_MOVE_BENCH_V1` requires optimized P95 ≤70% of legacy; managed Runtime Release receipt remains pending. |

- `candidates.rs` contains the Runtime580 behavior/source contracts and marker.
- No tooling changes; the combined Runtime580/Editor580 release gate remains pending.

### Grouped validation submission (2026-09-25)

Runtime580 was included in the related `optimization_batch_gz` submission:
Runtime development PTY `81972`, Editor development PTY `21257`, Runtime02
Release PTY `20043`, and Editor Release PTY `50976`. All four wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.

The exact `optimization_batch_gy` replacement wave uses Runtime development PTY
`72314`, Editor development PTY `76797`, Runtime02 Release PTY `83368`, and
Editor Release PTY `99044`; these wrappers remain intentionally unpolled.
