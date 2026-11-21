---
name: zircon-change-guard
description: Statically check a scoped ZirconEngine patch before local delivery or BUG-marker closure, using current permissions, module reachability and recurring defect heuristics.
---

# Zircon Change Guard

Use on the final edited checkout before local delivery or a scoped handoff. The [working agreement](../../../AGENTS.md), [checkout policy](../zircon-dev/references/main-branch-development-policy.md), [Agent management standard](../../../docs/tools/agent-artifact-management.md) and [validation policy](../../../docs/testing-and-validation/milestone-validation-policy.md) remain authoritative.

## Scope and execution

- Record checkout, branch, base SHA and preexisting changes. Declare exact task paths; preserve foreign edits within shared files too.
- Agents manage authorized native Git operations directly. Before index/ref/worktree writes verify the real Git directory, lock and exact task scope; this checker is not a command interceptor or a sandbox.
- Use the current permission selection. Full Access needs no optional `zircon-strict` allowlist; an explicitly selected restricted profile retains its actual grants. Control configuration, hooks, `AGENTS.md`, `.jenkins/` and `.github/workflows/` also need applicable task authorization. Reuse standing authorization when it covers the action; record its actual origin.
- Edit canonical files directly. Keep backups, source copies, scratch trees and compiler products out of a handoff. File-based fixtures need a writable, owned output directory; the OS temporary-directory default alone grants no permission.
- Agents execute authorized generation and scoped local validation under the management standard, using `tools/dev/local-cargo.ps1` for Cargo. Read-only structural checks follow the validation policy.
- Compiler products and caches must physically remain in permitted drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets` namespaces. Static spelling checks do not establish physical identity, capacity or native-lock safety.

## Check the exact patch

Use an available general Python 3.11 or later from the repository root:

```powershell
python -B .codex/skills/zircon-change-guard/scripts/change_guard.py --json patch --git --base <base-sha> --path <file> [--path <file> ...]
```

Prefer exact filenames. Explicitly selected untracked files are included even when ignored by Git; an ignored file hidden inside a directory selection makes the input incomplete. Review the receipt's file list against the intended scope. To check an existing diff use `--json patch --patch <file> --path <file> ...` against its final patched checkout. Missing or mismatched source fails the check. `scan <paths>` audits whole files without checking agent write scope.

Agent patch mode requires an allowlist and readable, valid permission configuration; unavailable input exits 2. For contributor PRs, `patch --contributor --patch <file>` explicitly omits agent write grants and records that omission. Do not use an empty config file to disable checks.

Fix all errors. A reviewed false positive may use a comment `guard-allow(<rule>): <reason of at least eight characters>` on the flagged line or up to three lines above it; file-level findings accept the first 40 lines. Scope and input findings, conflict markers and comment-only marker removal cannot be suppressed. Record warnings and suppressions in the handoff.

Apply the relevant [defect classes](references/defect-classes.md). A behavior fix needs its code, regression test and marker removal in the same patch. Reachability is checked from Cargo targets through literal module/include edges; macro expansion, feature selection, types and actual test execution still require normal validation. Do not weaken assertions or disable tests to obtain a pass.

Exit codes are 0 for static pass, 1 for findings with errors and 2 for unusable input. A static pass is not build/product acceptance and grants no commit or external-action authorization.

## Local delivery and handoff

Follow [handoff](references/handoff.md). Preserve the full behavior change, exact patch-byte digest, findings, source identity, actual command results and pending gates. Keep original identities when reconciling unknown asynchronous outcomes.

When committing is already authorized, inspect and commit the exact task paths with native Git. Passing checks and completing a milestone do not grant commit, push, remote PR or notification permission. Report the actual SHA separately from validation and acceptance results.
