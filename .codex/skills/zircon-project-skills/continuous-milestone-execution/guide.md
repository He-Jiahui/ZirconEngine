# Continuous Milestone Execution

Complete the user's authorized milestone or plan scope, including required validation and repair. Ordinary task autonomy does not require an explicit Goal-mode phrase.

Use [the execution loop](execution-loop/index.md) as the common workflow. An explicit audit-only, planning-only, or approval checkpoint remains a real boundary.

- Read the current milestone and the code, tests, and CI constraints relevant to the next step. Reuse unchanged material already read.
- Execute dependency-ready work, resolving routine choices from existing requirements.
- Diagnose failures at their owning layer and validate the affected batch under the repository policy.
- For Jenkins engineering validation, use [the incremental patch workflow](../../jenkins-coordination/references/incremental-patch-validation.md). Submit scoped patches against a reusable baseline; let Jenkins apply them and run the relevant build and tests. Do not repeatedly copy the project or substitute snapshot reconstruction when the patch interface is unavailable.
- Continue independent work when a dependency is pending. Use [receipt lifecycle](../cross-session-coordination/references/receipt-lifecycle.md) rather than automatic turn release or repeated polling.
- Complete only the requested scope. Starting another Goal, publishing, committing, or notifying requires the applicable authorization.
