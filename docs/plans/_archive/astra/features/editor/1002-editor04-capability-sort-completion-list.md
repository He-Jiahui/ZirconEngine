---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-08-26-unstable-context-command-capability-sort.md
  - docs/plans/optimize/zircon_editor/04/2026-08-26-unstable-creation-template-capability-sort.md
  - docs/plans/optimize/zircon_editor/04/2026-08-26-unstable-toolkit-capability-sort.md
related_records:
  - docs/plans/astra/features/editor/987-editor06-plugin-manager-optimization-completion-list.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/asset/type_registry/context_command.rs
  - zircon_editor/src/core/asset/type_registry/context_command/optimization_tests.rs
  - zircon_editor/src/core/asset/type_registry/toolkit.rs
  - zircon_editor/src/core/asset/type_registry/toolkit/optimization_tests.rs
  - zircon_editor/src/core/asset/type_registry/creation_template.rs
  - zircon_editor/src/core/asset/type_registry/creation_template/optimization_tests.rs
tests:
  - zircon_editor/src/core/asset/type_registry/toolkit/optimization_tests.rs
  - zircon_editor/src/core/asset/type_registry/creation_template/optimization_tests.rs
---

# Editor04 · capability canonicalization completion list

The three implementation-complete Editor04 descriptor slices are recorded as
one validation unit. They preserve capability membership, sorting, and
deduplication while reserving iterator lower bounds and using `sort_unstable`;
no stable-order contract is required by these descriptors.

| Plan slice | Optimization boundary | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Context command capabilities | Reserve and extend the required-capability vector, then unstable-sort/deduplicate. | `EDITOR04_TYPE_REGISTRY_CONTEXT_COMMAND_CAPABILITY_BENCH_V1`; release P95 ≤95% of legacy. | implemented_pending_validation |
| Creation template capabilities | Apply the same lower-bound reservation and unstable canonicalization to creation descriptors. | `EDITOR04_TYPE_REGISTRY_CREATION_TEMPLATE_CAPABILITY_BENCH_V1`; release P95 ≤95% of legacy. | implemented_pending_validation |
| Toolkit capabilities | Remove stable-sort overhead while keeping the public required-capability projection unchanged. | `EDITOR04_TYPE_REGISTRY_TOOLKIT_CAPABILITY_BENCH_V1`; release P95 ≤95% of legacy. | implemented_pending_validation |

## Source snapshots

| Owner | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/asset/type_registry/context_command.rs` | `C37ED2463787B4F67049BF410E416E05263CCE23B554249EEDCC2AA949E399B6` |
| `zircon_editor/src/core/asset/type_registry/context_command/optimization_tests.rs` | `599C76FAB5211214D375F06BF5603E65A96CFA1C7A6A38CA2480F63627DE018E` |
| `zircon_editor/src/core/asset/type_registry/toolkit.rs` | `7E88BD4BA496CF3BEB3ADBA4FECAF4324AE417E3403BE3B9917979609E1389EB` |
| `zircon_editor/src/core/asset/type_registry/toolkit/optimization_tests.rs` | `F172994ECF6CFDE10CC7B0320C5151E277F2A310BD8D1F772A1C8B349D250B8D` |
| `zircon_editor/src/core/asset/type_registry/creation_template.rs` | `F024DF0AFF424B417828643CE9EA32B58F38E2CCED982B8D7FDAE83DA573DCDD` |
| `zircon_editor/src/core/asset/type_registry/creation_template/optimization_tests.rs` | `29619E29E35962F9C03825E187411DF60D2CF7335091EA1F7F90F2EB3C98A032` |

Focused descriptor-equivalence/source-contract tests and the three ignored
release markers are present. The grouped managed Editor Cargo/Release receipt
is still required for the p95 thresholds; no timing acceptance is inferred.
