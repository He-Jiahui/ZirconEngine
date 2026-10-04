---
related_code:
implementation_files:
plan_sources:
  - user: 2026-09-29 improve Coordinator storage policy, Failure claim handling, validation scheduling and single-port remote workers
tests:
  - tools/session_coordinator/tests/test_storage_ledger.py
  - tools/session_coordinator/tests/test_storage_admission.py
  - tools/session_coordinator/tests/test_validation_priority.py
  - tools/session_coordinator/tests/test_validation_groups.py
  - tools/session_coordinator/tests/test_durable_tasks.py
  - tools/session_coordinator/tests/test_remote_paths.py
  - tools/session_coordinator/tests/test_control_storage.py
  - tools/session_coordinator/tests/test_optimization_cli.py
  - tools/session_coordinator/tests/test_optimization_actions.py
doc_type: workflow-detail
---

# Coordinator Storage and Task Policy

> Retired on 2026-10-02. The policies below describe the archived local coordinator and worker implementation. Follow [retirement](coordinator-retirement.md) and the current [storage policy](../../.codex/skills/zircon-dev/references/cargo-target-disk-policy.md). Do not restore the old scheduler, cleanup service, or remote storage exceptions; every new compilation product and compiler cache must physically remain below drive-root D/E/F `cargo-targets`.

## Local Coordinator storage

Every local Cargo target, compiler cache, and managed artifact must physically stay below one of the drive-root directories `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`. This is the allow-list used by local Coordinator admission and local validators. It rejects `C:`, repository-local output, `D/E/F:\targets`, `D/E/F:\ZirconBuilds`, nested lookalike roots, and path aliases.

The local storage ledger defaults to a 35 GiB free-space floor, a 24 GiB idle managed-cache budget, and a 16 GiB incremental-data sub-budget. Under pressure, reclamation targets 16 GiB of idle usage. At most two compatible hot pools are retained. Reservations count against available capacity so concurrent admissions cannot spend the same observed free space.

Inventory refresh is bounded to seven minutes and publishes a cached snapshot. The control-plane summary reads that snapshot; it does not start a filesystem walk on a request path. Reclamation is queued for background execution and defaults to at most eight items, one GiB, and seven minutes per run. Live jobs, validation copies, leases, retained pools, rollback paths, and other recorded references remain protected and are rechecked before deletion. Actual freed bytes are recorded separately from logical reclaim estimates.

## Remote worker storage boundary

A remote worker is a separate node contract. Its operator explicitly selects one dedicated absolute `storage_root`; the worker confines source inputs, target output, Cargo home, sccache, logs, temporary files, and artifacts beneath it. The worker rejects UNC/device paths, parent traversal, linked ancestors, and replacement or identity changes. `storage_class="local"` separately enforces the same D/E/F drive-root Cargo rule for a worker intended to share local storage.

An explicitly configured remote `storage_root` is the remote-node contract exception. It does not add another root to the local Coordinator allow-list, local `-TargetDir`/`CARGO_TARGET_DIR` checks, local build wrappers, or CI, and it does not authorize `C:` for local work. The local storage ledger and local reclaim summary do not account for or delete a remote node's files. Remote root admission, accounting, and reclamation must be owned by the worker node contract rather than inferred from local disk state.

## Scheduling, input reuse, and attempt evidence

The shared local validation queues use weighted classes with a 4:2:1 urgent/fix/ordinary schedule and promote a waiting task one class every ten complete minutes. Dependencies, compatibility, and resources still determine eligibility; priority does not admit an ineligible task.

Remote worker execution uses one heavy and two light execution slots. Separately, the source materializer permits at most two stable input slots; reuse requires matching input identity and slot generation. These are independent limits.

Durable task attempts bind task, generation, immutable input digest, worker/node, lease, provider, command, and execution metadata. Result and receipt references remain attempt evidence. They do not replace validation tickets, Cargo receipts, route-source validation, or runtime-instance activation evidence. Root-owned integration is required before remote attempt results can advance the local validation/acceptance lifecycle.

## Related operator guides

See [the Cargo target disk policy](../../.codex/skills/zircon-dev/references/cargo-target-disk-policy.md), the [validation skill](../../.codex/skills/zircon-dev/validation/SKILL.md), and the [worker protocol guide](coordinator-worker-protocol.md).
