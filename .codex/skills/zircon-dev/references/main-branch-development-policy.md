# Checkout policy

Keep the shared `main` checkout for synchronization and inspection. Implement tasks on their own branch/worktree and use native Git within the granted action scope under [the management standard](../../../../docs/tools/agent-artifact-management.md).

An explicit user-authorized branch, checkout, or worktree takes precedence. Read-only inspection may proceed on any branch. Before a mutation on an unexpected branch, establish whether the task authorizes that checkout; ask only when the unresolved choice affects the actual write. Preserve a dirty shared original checkout; do not reset, stash, clean or bulk-commit foreign work to establish a new task tree.

Record repository, branch/worktree, base SHA, exact task paths and preexisting changes. Resolve the actual Git directory with `git rev-parse --absolute-git-dir` and check that tree's `index.lock` before every Git write. A linked worktree's `.git` is a file. Preserve unknown locks, foreign staged paths and same-file edits; lock age or empty bytes do not authorize removal.

When the existing task authorization covers committing, the Agent may stage and commit its exact task files directly. Preserve any explicitly applicable gate; do not impose a whole-tree hash or compilation gate on ordinary commits. A milestone completion alone does not grant commit authority. Inspect the exact staged scope and report the actual SHA; retain validation failures and original-owner repair handoffs independently of the commit.

Local integration, rollback and worktree removal retain their own authorized scope. Fix input SHAs and the actual result tree, preserve conflicts and unknown deliveries, and verify writers have ended and unique changes are delivered before removing an owned tree. Check shared main cleanliness only when synchronizing it, immediately before and after that operation. Push, remote PR creation/merge, publication and external notifications still require their corresponding authorization.
