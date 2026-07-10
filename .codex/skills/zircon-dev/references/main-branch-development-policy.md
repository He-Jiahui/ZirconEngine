# ZirconEngine Main-Branch Development Policy

This repository develops directly on `main`. This policy overrides generic workflow advice from imported skills.

- Stay in the existing checkout on `main`.
- Do not create git worktrees.
- Do not create, switch to, or ask for feature branches.
- Do not treat "branch first, implement second" as an allowed default in `zirconEngine`.
- If another skill suggests `using-git-worktrees` or `finishing-a-development-branch`, apply this policy instead and continue on `main`.
- If `git branch --show-current` is not `main`, stop and ask the user how they want to reconcile that state with the repository policy.
- Finish work by running the required verification, summarizing changes, and reporting remaining risks. Do not present merge, PR, or branch-cleanup menus as the default closeout flow for `zirconEngine`.
