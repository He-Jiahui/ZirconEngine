# Engine review current-evidence row reconciliation

Date: 2026-07-10

Status: `engine_review_findings_current_evidence_rows_reconciled_static_passed`

The numbered review output contained 30 literal placeholder status tokens after the parent-plan/output-record migration. They made closed and pending review rows look precise while carrying no resolvable status identity.

All placeholder tokens are now replaced by the concrete record-status anchor above. This anchor means only that the current evidence row is reconciled and guarded; it does not promote the row's broader Cargo, FPS, RenderDoc, plugin, UI, or workspace gate. Rows with more specific status anchors, including F6, D10, and D11, retain those exact anchors.

Verification:

- literal placeholder occurrences: 30 -> 0;
- concrete reconciliation anchor occurrences: 31;
- `review_numbered_output_uses_concrete_status_anchors_instead_of_placeholders`: 1/1;
- scoped rustfmt: passed.

## Existing default-feature binary baseline

The available default-feature test binary predates the current review-guard owner tree. Its `code_review_findings` filter selected 297 tests and reported 120 passed / 177 failed / 0 ignored / 7141 filtered in 16.30 seconds.

Failure classification:

- 50 direct review-guard failures;
- 127 nested `structure_convention::test_file_budget::code_review_findings` failures;
- the nested failures name retired pre-split owners and status mirrors, so this binary is retained as a historical drift baseline rather than current-source acceptance evidence;
- a fresh current-source default-feature binary remains required before the aggregate review filter can be promoted.

Status: `runtime_15_code_review_findings_old_binary_120_passed_177_failed_structure_drift_classified_fresh_filter_pending`.

## Current target-client binary follow-up

A later current-source `target-client` lib-test binary selected 298 `code_review_findings` tests and reported 138 passed / 160 failed / 0 ignored / 7151 filtered in 14.16 seconds. The failure set is 34 direct review guards plus 126 nested structure/file-budget guards. Relative to the historical default-feature binary, the direct review failures decreased from 50 to 34 and the nested failures from 127 to 126; the aggregate is still not green and the profiles are not interchangeable.

The newly added placeholder-integrity guard passes 1/1 in this binary. This proves the priority output no longer carries literal placeholder statuses in a compiled current-source profile, while leaving the remaining review and structure rows open.

Status: `runtime_15_code_review_findings_target_client_138_passed_160_failed_placeholder_guard_passed`.
