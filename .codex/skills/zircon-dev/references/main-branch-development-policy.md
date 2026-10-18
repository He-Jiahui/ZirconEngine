# Checkout policy

Keep the shared `main` checkout clean. It is the synchronization and inspection checkout; task implementation must use a Jenkins-managed worktree on its own `codex/` branch. This is the user's 2026-10-17 checkout policy and supersedes the former shared-main default.

An explicit user-authorized branch, checkout, or worktree takes precedence. Read-only inspection may proceed on any branch. Before a mutation on an unexpected branch, establish whether the task authorizes that checkout; ask only when the unresolved choice affects the actual write.

Jenkins owns branch/worktree creation, switching, multi-worktree merges, release,
deletion and all other version-management writes. Agents use the assigned
checkout and edit only authorized canonical paths. A user-authorized checkout
does not itself authorize agents to run Git lifecycle operations outside Jenkins;
an explicit exception for those actions does. Follow the
[management standard](../../../../docs/tools/agent-artifact-management.md).

The latest 2026-10-17 user instruction permits immediate scoped branch commits even when other sessions have edits on main. Preserve those edits; never reset, stash, clean or bulk-commit foreign work. Check main cleanliness only when synchronizing main. A task commit needs no source-hash reconciliation, validation recipe, complete compilation or formal acceptance.

Jenkins commits exact task paths on the task worktree branch and optionally checks the combined candidate. Validation is advisory: failures retain their logs and produce original-owner repair handoffs while PR integration may continue. Publish the task branch and merge through a real PR under the applicable remote-action authorization. The scheduled queue observes the real PR and fast-forwards local main from its actual remote merge result; it does not require equality with the optionally checked local tree. Main must be clean immediately before and after synchronization. Keep branch commits, failures and original request identities. See [the delivery contract](../../../../docs/tools/jenkins-branch-integration.md).
