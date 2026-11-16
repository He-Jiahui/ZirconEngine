---
name: zircon-dev
description: Edit ZirconEngine Rust code or Cargo configuration, keep engine paths short, and select scoped validation when due.
---

# Zircon Dev

Work from the repository root and follow [checkout ownership](references/main-branch-development-policy.md). Reuse [the delivery policy](../zircon-engineering/SKILL.md) when already read.

Inspect the current Cargo manifest and touched code to identify ownership. `zircon_app`, `zircon_runtime`, and `zircon_editor` are primary architectural roles; supporting packages retain their own contracts. Script and VM work is one subsystem of this workspace.

## Minimal module builds

Before every compilation-capable `check`, `build` or `test` command, explicitly
announce the engine modules, Cargo packages, selected features, targets/test
filters, profile, exact command and coverage reason to the user. Follow
[the build declaration](validation/guide.md#declare-every-build). For ordinary work, disable default
features and enable only those needed by the affected modules; required
transitive dependencies remain part of that declared build.
Prefer the owning leaf crate over an aggregate runtime/editor build when it
covers the change.

Explicit `--workspace` or package-set builds required by the current task or
owning validation documentation are allowed under
[declared gates](../../../docs/testing-and-validation/milestone-validation-policy.md#declared-workspace-and-package-set-gates).
Blind `--all`, `--all-features`, `--all-targets`, implicit root-wide selection
and undeclared all-package loops/matrices remain prohibited unless the user
explicitly authorizes that scope. Wave/release or manifest/lockfile/toolchain
work alone supplies none. John's full local Windows/MSVC U09 regression follows
[the U09 exception](../../../docs/testing-and-validation/milestone-validation-policy.md#authorized-windowsmsvc-u09-regression).
Inspect dependencies when ordinary scope is unclear. Use
`tools/dev/dev-module-build.ps1` with an explicit `-Package` for supported
selections; the Agent executes locally and reuses compatible artifacts under their native locks.

## Read for the current operation

| Operation | Guidance |
| --- | --- |
| Name, rename or audit engine files and folders | [Path naming](path-names.md) |
| Edit Rust modules or organize tests | [Editing guide](workflow/guide.md); select its relevant references |
| Change subsystem ownership or a shared public boundary | [Architecture](../zircon-project-skills/zr-architecture-first-engineering/SKILL.md) |
| Run Cargo or report a build/test result | [Validation](validation/guide.md) |
| Report broader acceptance or pending evidence | [Reporting](reporting.md) |

Ordinary implementation slices use read-only formatting and structural checks.
When scoped compilation or generated output is due under
[the validation policy](../../../docs/testing-and-validation/milestone-validation-policy.md),
the Agent executes the scoped local tools directly under the
[management standard](../../../docs/tools/agent-artifact-management.md).
Native Git/worktrees, generation, local Cargo and compatible artifact reuse are
the ordinary workflow.
Compilation products, build directories, compiler caches and temporary compiler
outputs must physically remain below drive-root `D:\cargo-targets`,
`E:\cargo-targets`, or `F:\cargo-targets`, retaining 35 GiB and native locks.

Edit authorized canonical paths in the task worktree. Do not create ad hoc source snapshots, backup copies or validation source trees; preserve foreign changes, source identity and historical receipts. Commit and remote actions retain their own actual authorization.

Open only guidance needed by the changed behavior or boundary. A small local edit does not require loading every architecture, execution, review, or closeout guide.
