# Checkout policy

Keep the shared `main` checkout clean. It is the synchronization and inspection checkout; task implementation must use a Jenkins-managed worktree on its own `codex/` branch. This is the user's 2026-10-17 checkout policy and supersedes the former shared-main default.

An explicit user-authorized branch, checkout, or worktree takes precedence. Read-only inspection may proceed on any branch. Before a mutation on an unexpected branch, establish whether the task authorizes that checkout; ask only when the unresolved choice affects the actual write.

Jenkins owns branch/worktree creation, switching, multi-worktree merges, release,
deletion and all other version-management writes. Agents use the assigned
checkout and edit only authorized canonical paths. A user-authorized checkout
does not itself authorize agents to run Git lifecycle operations outside Jenkins;
an explicit exception for those actions does. Follow the
[management standard](../../../../docs/tools/agent-artifact-management.md).

Before task work, Jenkins checks that main has no staged, working or untracked changes, unresolved Git operation or index lock. If main is dirty, preserve the exact bytes and reconcile their owners; never reset, stash, clean or bulk-commit foreign work to manufacture cleanliness. The user's explicit one-time local-main checkpoint on 2026-10-17 is a bootstrap exception, not standing permission to develop or commit on main.

Jenkins commits exact task paths on the task worktree branch, incorporates current main into an isolated candidate and validates that actual result. Publish the task branch and merge through a real PR under the applicable remote-action authorization. The scheduled queue verifies the PR's repository, source SHA, target main, actual merged state and result tree before fast-forwarding local main. Direct candidate-to-main publication is disabled. Main must be clean again immediately before and after synchronization. Keep branch commits, failures and original request identities until delivery and cleanup are verified. See [the delivery contract](../../../../docs/tools/jenkins-branch-integration.md).
