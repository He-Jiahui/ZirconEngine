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
[the build declaration](validation/guide.md#declare-every-build). Disable default
features and enable only those needed by the affected modules; required
transitive dependencies remain part of that declared build.
Prefer the owning leaf crate over an aggregate runtime/editor build when it
covers the change.

Full builds are prohibited, including `--workspace`, `--all`, `--all-features`,
`--all-targets`, implicit root-wide selection and equivalent all-package loops
or matrices. Wave/release gates and manifest/lockfile/toolchain changes supply
no exception. Inspect dependencies when scope is unclear. Use
`tools/dev/dev-module-build.ps1` with an explicit `-Package` for supported
selections; Jenkins executes the scoped build and reuses compatible artifacts.

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
hand off to Jenkins. Jenkins owns branches/worktrees/merges, generation, builds,
artifact reuse and cleanup under the
[management standard](../../../docs/tools/agent-artifact-management.md).
The independent local Cargo wrapper remains available only for an explicitly
authorized local exception; it is not a fallback for a pending Jenkins request.
Compilation products and compiler caches must physically remain below drive-root
`D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`.

Source snapshots and backup copies follow the [coordinator ownership policy](../jenkins-coordination/references/incremental-patch-validation.md): agents hand off patches and hashes; the independent Jenkins coordinator prepares the sources.

Open only guidance needed by the changed behavior or boundary. A small local edit does not require loading every architecture, execution, review, or closeout guide.
