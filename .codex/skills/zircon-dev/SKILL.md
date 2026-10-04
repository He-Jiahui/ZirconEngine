---
name: zircon-dev
description: Edit ZirconEngine Rust code or Cargo configuration, and select scoped native validation when it is due.
---

# Zircon Dev

Work from the repository root and follow [checkout ownership](references/main-branch-development-policy.md). Reuse [the delivery policy](../zircon-engineering/SKILL.md) when already read.

Inspect the current Cargo manifest and touched code to identify ownership. `zircon_app`, `zircon_runtime`, and `zircon_editor` are primary architectural roles; supporting packages retain their own contracts. Script and VM work is one subsystem of this workspace.

## Read for the current operation

| Operation | Guidance |
| --- | --- |
| Edit Rust modules or organize tests | [Editing guide](workflow/guide.md); select its relevant references |
| Change subsystem ownership or a shared public boundary | [Architecture](../zircon-project-skills/zr-architecture-first-engineering/SKILL.md) |
| Run Cargo or report a build/test result | [Validation](validation/guide.md) |
| Report broader acceptance or pending evidence | [Reporting](reporting.md) |

Ordinary implementation slices use formatting and structural checks. Run the scoped Cargo batch when its validation gate is due, or earlier for the exceptions in [the validation policy](../../../docs/plans/milestone-validation-policy.md). Use `tools/dev/local-cargo.ps1`; Windows is the default. Compilation products and compiler caches must physically remain below drive-root `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`.

Source snapshots and backup copies follow the [coordinator ownership policy](../jenkins-coordination/references/incremental-patch-validation.md): agents hand off patches and hashes; the independent Jenkins coordinator prepares the sources.

Open only guidance needed by the changed behavior or boundary. A small local edit does not require loading every architecture, execution, review, or closeout guide.
