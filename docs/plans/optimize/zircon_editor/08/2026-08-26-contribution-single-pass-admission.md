# Editor08 Contribution Single-Pass Admission Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plans: Editor08 and Editor130 command registry reviews, scalable admission budget under E-CMD-P1-06
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Both contribution registration paths checked `command_ids.contains(id)` and
then called `command_ids.insert(id)`. Every successful unique command therefore
traversed the same `BTreeSet` twice before publishing its pending descriptor.
Large builtin/plugin contribution batches paid the duplicate membership lookup
on the common success path.

## Change

- Introduce one `claim_command_id` admission authority shared by descriptor-only
  and descriptor-plus-factory registration.
- Use the boolean result of a single `BTreeSet::insert` to detect duplicates.
- Preserve descriptor validation, duplicate-command error precedence, retained
  seen IDs after `take_pending`, pending descriptor/factory publication order,
  and rollback if an inconsistent factory-only collision is observed.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 32,768 unique parsed command IDs | 65,536 membership tree traversals | 32,768 membership tree traversals | 50% |
| Successful ID string clones | two owned IDs for retained set and pending map | two owned IDs for retained set and pending map | unchanged |
| Duplicate semantics | typed `DuplicateCommand` | typed `DuplicateCommand` | unchanged |

The ignored release gate runs 17 alternating contains-plus-insert/single-insert
sample pairs over 32,768 already parsed IDs. Acceptance requires single-pass
nearest-rank P95 to be at most 80% of legacy P95, a minimum 20% reduction.
Exact Windows timing values remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826c_editor08_contribution_admission_uses_one_tree_traversal`
  locks the shared insert-return-value authority.
- `optimization_batch_20260826c_editor08_contribution_admission_keeps_seen_ids_after_take`
  locks duplicate rejection after pending descriptors are consumed.
- `optimization_batch_20260826c_editor08_contribution_admission_performance_evidence`
  emits `EDITOR08_CONTRIBUTION_SINGLE_PASS_ADMISSION_BENCH_V1`, raw samples,
  command count, tree-traversal counts, and the 20% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt, source contracts, and scoped diff checks must
  pass before managed validation submission.

## Remaining Plan Work

This slice does not close Editor08 or Editor130. Atomic owner-scoped registration
leases, complete secondary indexes, command ID unification, remote deny-by-default,
typed invocation outcomes, menu graph convergence, and full catalog scale gates
remain open.
