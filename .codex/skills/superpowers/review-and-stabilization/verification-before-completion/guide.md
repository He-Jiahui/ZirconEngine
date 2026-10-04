# Verification Before Completion

Make claims proportional to observed evidence. Inspect the relevant command result before saying a change passes, is fixed, or is accepted.

- Reuse a recorded result when the relevant source, tests, configuration, toolchain, and environment still match. A new message, review, commit preparation, or closeout does not itself invalidate evidence.
- Re-run the affected checks after a relevant change, a failure, or a concrete uncertainty about that evidence. Expand coverage only when the affected contracts or failures justify it.
- Select checks from [the repository validation policy](../../../../../docs/plans/milestone-validation-policy.md). Documentation and other reversible low-impact changes need appropriate structural checks, not invented unit tests.
- Distinguish source inspection, a dry run, focused test success, broader acceptance, and pending external validation. A queued receipt is not a passing result.
- Report the checked scope and material limitations. If a required check cannot run, keep that acceptance claim open, explain the specific blocker, and continue independent authorized work.

For managed asynchronous validation, use [receipt lifecycle](../../../zircon-project-skills/cross-session-coordination/references/receipt-lifecycle.md). Do not repeat a submission to obtain a fresher-looking result.
