# Runtime15 Hard-Cutover Test-File Boundary Classification

status: implementation_reviewed_broad_audit_current_remaining_debt
date: 2026-07-22
base_head: 6debc3e43aed7ed3ee9c7e25e38388bdd209981a

## Scope

This Runtime15 infrastructure slice makes the hard-cutover migration-smell
audit apply the repository test-file convention consistently. It changes only
the audit owner, its focused Python test, and this numbered child record.

Exact scope:

- `.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py`
- `tools/tests/test_hard_cutover_migration_smells.py`
- `docs/plans/zircon_runtime/runtime/15/2026-07-22-hard-cutover-test-file-boundary-classification.md`

## Failure Evidence

`zircon_runtime_interface/src/ui/focus_tests.rs` is a test owner, but the
hard-cutover audit excluded only `tests/` directories and the exact
`tests.rs` filename. The current repository scan therefore counted its test
string `legacy navigation contract deserializes` as production debt.

Focused RED command:

```text
python -m unittest tools.tests.test_hard_cutover_migration_smells.HardCutoverMigrationSmellsTests.test_test_suffix_files_do_not_count_as_production_migration_debt
```

Result: exit 1, 1 test executed, `source_file_count` was 1 rather than 0.

## Implementation

- Exclude files ending in `_tests.rs` from production Rust inventory, matching
  the repository's existing runtime naming/test-layout convention.
- Keep `tests/` and exact `tests.rs` exclusions unchanged.
- Add a temporary-repository regression whose `_tests.rs` file contains
  `legacy` wording without relying on an outer `#[cfg(test)]` module.
- Do not classify or suppress the real production debts in
  `zircon_runtime_interface/src/ui/surface/hit.rs` or
  `zircon_editor/src/ui/preferences/persistence.rs`.

## Focused Validation

| Gate | Result |
|---|---|
| Python compile for audit and test | passed |
| scoped `git diff --check` | passed; line-ending conversion warnings only |
| new `_tests.rs` boundary regression | passed |
| existing direct `#[cfg(test)]` module boundary regression | passed |
| combined focused execution | 2 passed / 0 failed, 0.033s |
| current full repository audit | production files 10834; unclassified 11 -> 10; remaining exact owner groups are Editor preferences 9 and Runtime Interface hit route 1 |

Current source hashes:

| Path | SHA256 |
|---|---|
| `hard_cutover_migration_smells.py` | `77fde1f787be5bd14e6e33113d005081eac44a5db8e1b1741688dce0a8bb01e9` |
| `test_hard_cutover_migration_smells.py` | `e462363bc6eab588d23aba4f995f7668662c85e9d1076cc0024c710e16bad0a6` |

## Remaining Boundaries

- The full hard-cutover gate remains red by design: the ten current production
  references above are not reclassified or waived by this test-boundary slice.
- `UiHitPath::with_route` remains owned by the active Editor Layout18 session
  and is not absorbed by this slice.
- Editor appearance-preference migration naming remains an Editor owner
  decision; this slice does not turn it into an allowed classification.
- The clean architecture mirror
  `docs/engine-architecture/hard-cutover-migration-smells-m1.md` still records
  the older `9593 / 85 / 0` inventory from commit
  `facb719f4da98953ec83f682175389916da51b6b`; a future architecture-document
  owner must refresh it from the current `10834 / 10` audit without widening
  this exact three-path implementation slice.
- No Cargo, managed commit, fixed return, or Runtime15 parent completion is
  claimed.

Independent focused review returned Critical 0 / Important 0 / Minor 0. It
verified that `focus_tests.rs` is mounted by a `#[cfg(test)]` path module, the
new rule is suffix-bounded, ordinary production `.rs` files remain audited,
and no compatibility classification or product-owner exemption was added.
