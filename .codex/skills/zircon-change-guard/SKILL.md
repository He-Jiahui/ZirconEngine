---
name: zircon-change-guard
description: Statically check a scoped ZirconEngine patch before Jenkins handoff or BUG-marker closure, using write grants, module reachability and recurring defect heuristics.
---

# Zircon Change Guard

Use on the final edited checkout before preparing a patch handoff. The [working agreement](../../../AGENTS.md), [checkout policy](../zircon-dev/references/main-branch-development-policy.md), [source ownership](../jenkins-coordination/references/incremental-patch-validation.md), [management standard](../../../docs/tools/agent-artifact-management.md) and [validation policy](../../../docs/plans/milestone-validation-policy.md) remain authoritative.

## Scope and execution

- Record checkout, branch, base SHA and preexisting changes. Declare exact task paths; preserve foreign edits within shared files too.
- Local Git queries are read-only. Jenkins owns operations that modify the index, refs or checkout, including staging, committing, switching branches and managing worktrees. This checker is not a command interceptor or a sandbox.
- Each agent path needs a write grant in the configured `zircon-strict` profile. Control configuration, hooks, `AGENTS.md`, `.jenkins/` and `.github/workflows/` also need explicit user authorization. Reuse standing authorization when it covers the action; record its actual origin.
- Edit canonical files directly. Keep backups, source copies, scratch trees and compiler products out of a handoff. File-based test fixtures need an explicitly writable, owned output directory; the OS temporary-directory default alone grants no permission.
- Jenkins owns generation, compilation, execution and cleanup. A user-authorized local Cargo exception uses `tools/dev/local-cargo.ps1`. Read-only structural checks can run locally under the validation policy.
- Compiler products and caches must physically remain in permitted namespaces under drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets`. Static path spelling checks do not establish physical storage identity.

## Check the exact patch

Use Python 3.11 or later from the repository root:

```powershell
& '.jenkins/runtime/python/python.exe' -B .codex/skills/zircon-change-guard/scripts/change_guard.py --json patch --git --base <base-sha> --path <file> [--path <file> ...]
```

Prefer exact filenames. Explicitly selected untracked files are included even when ignored by Git; an ignored file hidden inside a directory selection makes the input incomplete. Review the receipt's file list against the intended handoff. To check an existing diff use `--json patch --patch <file> --path <file> ...` against its final patched checkout. Missing or mismatched source fails the check. `scan <paths>` audits whole files without checking agent write scope.

Agent patch mode requires an allowlist and readable, valid permission configuration; unavailable input exits 2. For contributor PRs, `patch --contributor --patch <file>` explicitly omits agent write grants and records that omission. Do not use an empty config file to disable checks.

Fix all errors. A reviewed false positive may use a comment `guard-allow(<rule>): <reason of at least eight characters>` on the flagged line or up to three lines above it; file-level findings accept the first 40 lines. Scope and input findings, conflict markers and comment-only marker removal cannot be suppressed. Record warnings and suppressions in the handoff.

Apply the relevant [defect classes](references/defect-classes.md). A behavior fix needs its code, regression test and marker removal in the same patch. Reachability is checked from Cargo targets through literal module/include edges; macro expansion, feature selection, types and actual test execution still require their normal validation. Do not weaken assertions or disable tests to obtain a pass.

Exit codes are 0 for static pass, 1 for findings with errors and 2 for unusable input. A static pass grants no build, Jenkins acceptance or external action.

## Handoff

Follow [handoff](references/handoff.md) after verifying the current Jenkins interface and authorization. Keep one request and attempt through query/reconcile; queued, pending and dry-run states leave acceptance open. Preserve the full behavior change when Jenkins classifies it; do not trim it to obtain `comments_only`.

Commit only through an accepted, authorized flow with `ALLOW_COMMIT=true` and the applicable `COMMIT_AUTHORIZATION`. User authorization for commit, push and notification is separate from passing checks. Report the exact patch-byte digest, warnings/suppressions, observed stage receipts and pending gates.
