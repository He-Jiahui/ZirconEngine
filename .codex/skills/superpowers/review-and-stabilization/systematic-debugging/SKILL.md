---
name: systematic-debugging
description: Investigate bugs or test failures when their cause is unclear; trace the failing contract and verify the repair.
---

# Systematic Debugging

Find a supported cause before choosing a fix. Scale investigation to the failure.

1. Capture the actual failure, relevant inputs, environment, and recent changes. Reproduce it when doing so adds evidence; reuse an existing reliable reproduction.
2. Trace the failing value or contract toward its owner. Compare a working path or reference implementation only as far as needed to distinguish the hypotheses.
3. Choose the smallest discriminating check. Record what the result rules in or out, then repair the responsible layer.
4. Validate the affected behavior and callers using [the validation policy](../../../../../docs/plans/milestone-validation-policy.md). Add a regression test when it protects behavior that could recur.

Repeated failures call for revisiting the hypothesis, boundaries, or missing evidence. An arbitrary attempt count does not require user approval. Ask when a consequential decision or unavailable information is actually needed; keep independent work moving.

Do not mask a failure with a fallback, weaker assertion, unrelated cleanup, or a speculative bundle of changes. Preserve others' work and diagnose existing failures separately from regressions introduced here.

Optional techniques:
- [Root cause tracing](root-cause-tracing.md) for a failure far from its origin.
- [Condition-based waiting](condition-based-waiting.md) for timing and asynchronous tests.
- [Defense in depth](defense-in-depth.md) when several trust boundaries each need validation.
- [Shared support diagnosis](../../../zircon-project-skills/support-first-regression-testing/guide.md) for layered ZirconEngine regressions.

These are references to select, not phases to read and execute for every bug.
