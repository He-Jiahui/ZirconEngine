---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74/2026-09-21-prototype-resource-alias-single-buffer.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/prototype_file_cache.rs
tests:
  - zircon_runtime/src/ui/template/asset/prototype_file_cache/resource_alias_single_buffer_tests.rs
  - tools/tests/test_runtime872_prototype_resource_alias_single_buffer_performance_contract.py
---

# Runtime872 Prototype Resource Alias Single Buffer

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime74 UI prototype-file resource aliasing | Append valid UTF-8 relative components directly into one capacity-bounded `res://` alias, eliminating the component vector and joined child while preserving root selection, path ordering, normalized separators, Unicode, and empty/no-root behavior. | Combined intentional RED `2/10` → GREEN `10/10`; lower absolute/relative/Unicode/nested-root parity and ignored `RUNTIME872_PROTOTYPE_RESOURCE_ALIAS_SINGLE_BUFFER_BENCH_V1` are wired. The 4,096-alias model changes reference slots/join outputs `262144/4096→0/0`; final adjacent combined static coverage passes `31/31`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/prototype_file_cache.rs` | `86E0B3F8F3C7F32EAB5437F8C5F76FAE8F4C2D732D1E0D84B8B67839FF17DFB5` |
| `zircon_runtime/src/ui/template/asset/prototype_file_cache/resource_alias_single_buffer_tests.rs` | `59D2AC49547BE357BD198B3F1F9FEC546055A0F85FD87794FFE3FDA289682540` |
| `tools/tests/test_runtime872_prototype_resource_alias_single_buffer_performance_contract.py` | `D30F486B3BD2272C7280D68C34570DC3B889FBD832DE47208B2E2D7C243F8FFC` |

## Managed gate

Runtime872 was submitted with Editor891 in asynchronous v18 (PID `35604`) at
`2026-09-21T22:02:33.0422146+08:00` rather than receiving a per-task Cargo run.
Keep it pending until that combined Windows lane supplies current-source
Runtime compilation, lower/ignored Release execution, allocator evidence, and
prototype-cache resource-resolution product p50/p95/p99 evidence.
