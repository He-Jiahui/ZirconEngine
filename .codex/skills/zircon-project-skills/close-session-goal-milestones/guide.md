# Close Session Goal Milestones

Close the authorized scope using actual validation, required review and applicable failure resolution. The retired coordinator's candidate, milestone, commit, lease and close-goal APIs are not prerequisites. Its historical records remain preserved for Jenkins migration.

## Before closeout

1. Apply `cross-session-coordination`: inspect current diffs and the exact owned scope; preserve foreign edits, staged files, source attribution and live locks.
2. Resolve applicable canonical `failure-*.md` handoffs before accepting dependent work. Repair the lowest shared cause and validate upward.
3. Reuse validation and review evidence for unchanged scope. Re-run only affected checks after a relevant change/failure; pending tickets and dry runs are not passing evidence.
4. Inventory every owned changed path, including new files, in a scoped manifest with current hashes. A deletion must have an identified base and explicit ownership. Do not absorb or unstage another task's work.
5. Use `write-plan-output-records` for an accepted outcome. Write only the authorized child output; global plan definitions and indexes remain read-only without maintenance authorization.

## Authorization and acceptance

- Routine completion does not require a commit. Committing, pushing, publishing and external messages each require authorization; milestone completion does not grant it.
- If committing is authorized, inspect the exact staged diff against the owned manifest and preserve foreign staged paths. Use a specific Conventional Commit subject. Do not delete a live index lock or bypass an applicable validation/review gate.
- Do not automatically invoke old coordinator submission/notification actions or manually send/backfill WeCom messages.
- Local Cargo command results provide command evidence only. Jenkins migration and formal acceptance remain pending until their declared gates are implemented and met.

## Finish by mode

- **Milestone:** Require the declared testing stage, required review and applicable failure resolution; record one concise accepted outcome, then continue the remaining authorized milestones.
- **Goal:** Require all plan items and acceptance gates, resolve applicable failures, and reconcile remaining owned changes. Do not create an empty commit or mark incomplete work complete. Update a tool-managed Goal only when its objective is achieved.
- In the terminal report identify delivered scope, actual verification, pending external gates, and any authorized commit result. Report foreign diagnostics without changing them.

Incomplete tests/review, unresolved applicable failures, foreign staged conflicts or unverified scope delay acceptance. Continue independent authorized work and keep pending acceptance open under [receipt lifecycle](../cross-session-coordination/references/receipt-lifecycle.md).
