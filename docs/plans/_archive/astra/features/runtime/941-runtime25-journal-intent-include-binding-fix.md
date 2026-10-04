---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime25-journal-intent-include-binding-fix.md
related_records:
  - docs/plans/astra/features/runtime/939-runtime02-direct-rayon-execution-authority.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
read_only_dependencies:
  - zircon_runtime/crates/zr_resource/src/io/transaction/journal/intent.rs
tests:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
---

# Runtime941 Runtime25 Journal Intent Include Binding Fix

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Hard-cut owner binding | The durability contract now includes the `zr_resource` journal-intent owner directly after Runtime25 moved it out of the monolithic Runtime path. | the stale `core/resource` include is absent and the current owner path exists; no compatibility file or fallback was added. |
| Durability assertions | The crash-window contract checks replacement/flush/directory ordering, retired-sidecar persistence, Windows writable-handle flushing, and write-through promotion. | focused source contract is present; it is covered by the grouped Runtime25 local batch. |
| Ownership | Only the Runtime test owner changed; the `zr_resource` intent implementation remains a read-only dependency owned by Frameworks01. | current SHA-256 and scoped diff evidence are recorded in the source optimize plan. |
| Validation gate | The historical batch failed during compile-time resource materialization before Cargo. | a current grouped managed batch must prove path materialization and compile/test success; no wrapper or historical failure is treated as a pass. |

## Validation boundary

The combined local static batch passed `18/18`, including the Runtime25 pressure contracts. Managed
Cargo validation is still grouped with the Runtime/Editor lanes and remains pending; tooling is
intentionally deferred.
