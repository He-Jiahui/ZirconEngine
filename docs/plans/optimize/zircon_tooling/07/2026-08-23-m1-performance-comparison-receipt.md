# Tooling 07 M1 performance comparison receipt

Plan: docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md

Milestone: M1

Status: integrated_validation_pending

Files: ["docs/plans/optimize/zircon_tooling/07/2026-08-23-m1-performance-comparison-receipt.md", "tools/tests/test_validate_performance_comparison_receipt.py", "tools/validate_performance_comparison_receipt.py"]

## Scope

- Bind each Render19 comparison receipt to its reported scenario ID, scenario inputs,
  raw artifacts, Build Set, independent capture tickets, and raw sample statistics.
- Reject every self-asserted baseline update until a Coordinator promotion contract
  can bind its trusted policy, worker ticket, candidate snapshot, and artifact receipt.
- Reject an accepted comparison when either the median or P95 point estimate or
  bootstrap upper confidence bound exceeds the declared regression budget.
- Reproduce a conservative bootstrap confidence interval from the supplied raw samples.

## Validation status

Coordinator validation ticket `4a26d4acec104b85bea9bbe15976d607` passed the
five-module Python tooling batch: 80 tests in 46.462 seconds, exit 0, on
2026-08-23. The batch includes direct CLI execution of the comparison validator
and a dedicated P95 bootstrap upper-bound regression case.

Independent review found that the current Render19 sidecar schema does not yet
carry Coordinator-owned worker ticket/candidate, managed artifact, Build Set,
or machine-manifest identities. The validator therefore remains fail-closed for
baseline promotion until Tooling 06 and the M0 sidecar contract provide those
identities. This record does not establish an accepted performance baseline or
a performance qualification result.
