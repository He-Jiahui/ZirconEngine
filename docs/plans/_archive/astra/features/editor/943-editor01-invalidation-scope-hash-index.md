---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-invalidation-scope-hash-index.md
related_records:
  - docs/plans/astra/features/editor/942-editor01-pane-option-hash-membership.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/invalidation/root/transaction.rs
tests:
  - zircon_editor/src/ui/retained_host/app/invalidation/root/transaction/hash_index_tests.rs
---

# Editor943 Editor01 Invalidation-Scope Hash Index


| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Invalidation coalescing | pending invalidation scopes use `HashMap<HostInvalidationScope, HostInvalidationMask>` for expected constant-time coalescing and lookup; mask union, consume semantics and deterministic presentation view sorting remain unchanged. | Behavior/source contracts cover scope coalescing, explicit snapshot ordering, and hash ownership. |
| 性能门禁 | 4,096 long shared-prefix view scopes and stable hits switch from ordered lookup to hash lookup without extra hit allocations. | ignored marker `EDITOR01_INVALIDATION_SCOPE_HASH_INDEX_BENCH_V1` requires hash P95 at least 30% below the legacy ordered path; managed Editor01 Release receipt remains pending. |


- `transaction.rs` and `hash_index_tests.rs` source checks pass.
- No tooling changes; this record registers the existing implementation and its pending managed gate.

### Grouped validation submission (2026-09-25)

The Editor01 marker is included in the grouped Editor Release lane PTY
`51515`, alongside Runtime development PTY `8644`, Editor development PTY
`21683`, and Runtime02 Release PTY `11980`. The wrappers remain
intentionally unpolled; managed compiler and P95 evidence is pending.

### Replacement grouped validation submission (2026-09-25)

The narrow `editor01` filter was superseded because this marker keeps its
historical batch name. The replacement broad-prefix Editor Release lane uses
PTY `35753`, with Runtime development PTY `76539`, Editor development PTY
`36565`, and Runtime02 Release PTY `51493`. These wrappers remain intentionally
unpolled; compiler, functional-test, and P95 receipts are still pending.
