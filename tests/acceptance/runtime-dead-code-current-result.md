# Runtime 15 dead-code current-source result

Date: 2026-07-10

Status: `runtime_15_dead_code_numbered_output_current_owner_reconciliation_13_passed`

The current target-client aggregate exposed four `runtime_dead_code` failures. All four were evidence-routing drift after parent plans became route-only; production suppression scanning itself already passed.

Current-source reconciliation:

- Runtime 15 evidence reads the numbered structure output archive;
- F10/F12 priority evidence reads the numbered review output archive;
- the F10 row again carries its specific closed top-row status instead of the generic row-reconciliation status;
- the F12 current-state guard locates the numbered output record rather than requiring the retired parent `S10` table row.

Verification: standalone folder-backed `structure_convention` harness selected 13 `runtime_dead_code` tests and passed 13/13. The production scan remains zero-hit. No runtime production behavior changed.
