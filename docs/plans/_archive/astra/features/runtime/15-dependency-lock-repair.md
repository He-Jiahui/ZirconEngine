---
status: in_progress
plan_sources:
  - docs/plans/astra/features/runtime/14-test-owner-contracts.md
  - docs/plans/astra/features/editor/05-wizard-plan-failure.md
  - docs/plans/astra/features/editor/06-output-projection.md
---

# Dependency lock repair

## Cause and repair

The M1-M17 combined run stopped before compilation because the current workspace
manifests and Cargo.lock differed. Runtime now declares goblin/ring and Editor
declares uuid; these manifest changes were retained. Cargo metadata reconciled
the existing lock, adding goblin 0.10.7, scroll 0.13.0, scroll_derive 0.13.2 and
syn 3.0.5 plus the current dependency edges. No manifest or tooling change was made.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M20 | Reconcile existing manifest dependencies into Cargo.lock | metadata_validated_tests_pending | `cargo metadata --locked --offline --format-version 1` succeeded: 856 packages/nodes, 45 workspace members |

The preceding run `7d1cd0cdecd5485ea668f587391cdd54` exited 101 at lock checking;
it supplied no Rust compilation, test or performance result. Restore the locked
combined release run including M18-M19 after this dependency repair. Do not infer
code acceptance from successful dependency resolution.
