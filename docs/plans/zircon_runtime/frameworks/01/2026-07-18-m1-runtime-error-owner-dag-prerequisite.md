---
related_code:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
implementation_files:
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - tools.tests.test_frameworks_01_runtime_error_owner_boundary
  - tools.tests.test_frameworks_02_core_error_single_source
doc_type: milestone-detail
---

# Frameworks01 M1 Runtime Error Owner DAG Prerequisite

Plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
Milestone: M1 owner-DAG prerequisite
Status: failure_repair_current_source_exact23_review_correction_pending
Date: 2026-07-18

## Delivered

| Slice | Status | Evidence |
|---|---|---|
| Error owner | implemented | `CoreError/CoreResult` have one physical owner in `core/runtime/error.rs`. |
| Framework reverse edge | removed | `core/framework/error.rs` and the framework module mount are deleted. |
| Runtime-local imports | implemented | lifecycle and module-order code resolve the kernel-owned error contract without importing framework. |
| Public facade | preserved | callers continue through curated `core::{CoreError, CoreResult}`. |
| Compatibility removal | implemented | no old module, alias, forwarding export, duplicate enum, or conversion shim survives. |
| Static guards | passed | focused Frameworks01 and Frameworks02 guards are 2/2 GREEN after an observed RED. |
| Independent review | correction pending | Snapshot 1364 code/architecture review reported P0/P1/P2 = 0/0/0, but found one Important stale coordination statement in this record. This row and the failure-repair state are being corrected before a fresh exact23 review. |
| Managed Cargo | pending | focused lifecycle/error behavior and Runtime lib compilation remain FIFO pending. |
| Failure repair | exact23 current source leased / lower dependency pending | Current successor `frameworks01-m1-runtime-error-owner-dag-prerequisite-r4-20260731` owns the 23 dirty paths plus two future return records. Snapshot 1364 includes 21 existing inputs and the two intentional framework-error tombstones. The superseded r6 session is cancelled and r7 is archived; neither holds current leases. The earlier `ResourceRegistryError` hard cut and foreign Runtime mirror inputs remain outside this exact23 acceptance boundary. Frameworks01/02 owner guards are 2/2 GREEN and scoped diff-check is clean; fresh source-bound Cargo and post-correction exact23 review remain pending. |

## Architecture Decision

The former framework error owner depended on kernel `ServiceKind`, while kernel lifecycle and module
ordering imported the framework error. That was a real extraction cycle, not a naming issue.
`CoreError` describes runtime registry, service resolution, module activation, channel/thread, and
configuration failures, so its lowest coherent owner is the runtime kernel.

The public `core::{CoreError, CoreResult}` surface remains the curated `zircon_runtime` facade. It is
not a compatibility path between internal crates: the retired `core::framework::error` owner is
physically absent and all direct consumers have moved.

The current checkout also contains the dependent Runtime11 diagnostics hard cut. Its exports share
three Frameworks01-owned facade/mirror paths, so the two slices must close from one current-source
manifest. The lower `ResourceRegistryError` owner must commit first; restoring the removed resource
variants on `CoreError` would be a forbidden rollback rather than a dependency repair. This record
does not treat the three dependency-owned architecture mirrors as accepted, does not claim a Cargo
result, and does not claim a commit SHA before both dependencies land and the manifest is recomputed
from the new `HEAD`.

## Remaining M1 DAG Work

This slice removes the error-owner edge only. Kernel runtime still consumes framework-owned
event/state/time/task contracts, `engine_module` ownership is not yet extracted, and the five M1
internal crates do not yet exist. Those dependencies and the full managed acceptance matrix remain
open before physical extraction can start.
