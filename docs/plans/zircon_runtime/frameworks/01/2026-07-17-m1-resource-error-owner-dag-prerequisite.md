---
related_code:
  - zircon_runtime/src/core/resource/error.rs
  - zircon_runtime/src/core/resource/registry.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/runtime/error.rs
implementation_files:
  - zircon_runtime/src/core/resource/error.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/core/resource/registry.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/runtime/error.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - registry_rename_reports_missing_locator_with_resource_error
  - review_f6_core_resource_registry_rename_uses_resource_error
  - scoped rustfmt --edition 2021
  - scoped git diff --check
doc_type: milestone-detail
---

# Frameworks01 M1 Resource Error Owner DAG Prerequisite

Plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
Milestone: M1 owner-DAG prerequisite
Status: current_source_recovered_static_validation_complete
Date: 2026-07-17

## Scope Delivered

| Slice | Status | Evidence |
|---|---|---|
| Registry error owner | implemented | `ResourceRegistryError` and `ResourceResult` live in `core::resource::error`. |
| Resource API hard cut | implemented | registry/manager rename paths return `ResourceResult<ResourceRecord>`. |
| Framework dependency removal | implemented | `CoreError` no longer contains locator/id resource-record variants. |
| Compatibility removal | implemented | no root alias, re-export, conversion shim, or duplicated error variants. |
| Priority review contract | implemented | F6 in `engine-code-review-findings-2026-06.md` now requires the lowest resource owner. |
| Independent review | accepted | exact 16-path re-review reports P0/P1/P2=0 after negative hard-cut guards were strengthened. |
| Current-source recovery | static complete | The archived exact 16-path slice is absorbed into the Frameworks01 + Runtime11 + Resource current-source atomic closure; the three later resource hot-path files required by the current source guard are included rather than split from their test. |
| Managed Cargo acceptance | pending | Focused resource/F6 gates must run only after the Plugins01 mirror dependency is committed and the coordinator FIFO reaches the final source-bound manifest. |

## Architecture Decision

Frameworks01 M1 cannot physically extract `zr_resource` while registry operations return
framework-owned `CoreResult`: that would create a lower resource crate depending upward on
`zr_contracts`. The error variants describe only resource-registry lookup failures, so their
canonical owner is `core::resource`, not the global lifecycle error surface.

The cut is direct. `ResourceRegistryError` is not wrapped into `CoreError`, `ResourceResult` is not
re-exported from `core`, and the removed variants do not survive under aliases. Callers that invoke
rename now observe the resource domain error type.

## Static Evidence

- old resource-record `CoreError` variants in current Rust source: 0;
- registry/manager rename `CoreResult<ResourceRecord>` signatures: 0;
- source and status guards use the resource-owned names and forbid old variants, root aliases/re-exports, and `CoreError` conversion shims;
- independent exact-scope re-review: P0/P1/P2=0;
- scoped rustfmt and diff-check are required before managed acceptance.

The recovered current-source candidate has passed the Framework error-owner guards, the Runtime11
mirror audit, scoped rustfmt, and scoped diff checking. This record does not claim managed Cargo,
final independent review, a failure return, or an independent Resource commit. Those gates remain
bound to the final atomic manifest after the Plugins01 mirror dependency supplies its commit SHA.

## Remaining M1 DAG Work

This prerequisite does not claim M1 physical extraction. Fresh audit still shows
framework↔kernel dependencies, manager-dependent runtime diagnostics, concrete behavior inside
`core/framework`, and `engine_module`↔runtime coupling. Those owners must be cut before the five
`zr_*` crates move as an atomic phase.
