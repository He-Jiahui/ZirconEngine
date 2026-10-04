# Shared execution loop

1. Identify the authorized outcome, explicit checkpoints, current owner, and next dependency-ready step.
2. Read only the missing or changed context needed for that step. Check shared ownership when overlap is plausible.
3. Implement a coherent slice. Choose routine details using current requirements and repository conventions.
4. At the appropriate validation boundary, run the affected batch from [the validation policy](../../../../../docs/plans/milestone-validation-policy.md). Reuse still-valid results and review.
5. Repair failures at the responsible layer, then re-run the affected checks. Continue independent slices when another owner or decision blocks a dependency.
6. Record only required durable evidence in the existing owner. Use [plan output records](../../write-plan-output-records/guide.md) when a milestone record is due.
7. Continue until the authorized scope is complete or an explicit checkpoint requires the user's decision. For asynchronous work follow [receipt lifecycle](../../cross-session-coordination/references/receipt-lifecycle.md); pending is not accepted.

A question, a failed attempt, a completed slice, or a queued ticket does not by itself stop all work. Ask only about a material missing requirement or authorization. Do not invent additional work after the requested outcome is achieved.
