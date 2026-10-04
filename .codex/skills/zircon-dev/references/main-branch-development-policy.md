# Checkout policy

Use the existing shared `main` checkout by default. Do not create or switch branches or worktrees merely because a generic workflow suggests it.

An explicit user-authorized branch, checkout, or worktree takes precedence. Read-only inspection may proceed on any branch. Before a mutation on an unexpected branch, establish whether the task authorizes that checkout; ask only when the unresolved choice affects the actual write.

Preserve existing edits and other Sessions' ownership. Never switch, reset, stash, or discard their work to enforce the default. Prepare authorized shared-main integration with an exact manifest, current file hashes, required review and validation evidence, and an inspected staged diff. Coordinator registration and commit APIs are retired; commits still require explicit authorization.
