# Zircon Rust Editing

Use the current Cargo manifest and touched source owners. Keep the converged app/runtime/editor roles and supporting-package ownership; [the architecture baseline](../../zircon-project-skills/zr-architecture-first-engineering/references/system-architecture-baseline.md) owns the detailed kernel and service contracts.

- Keep production roots (`binding.rs`, `lib.rs`, `main.rs`, and `mod.rs`) structural. Put substantial parsing, routing, mutation, or domain orchestration in coherent child modules.
- Organize source by subsystem ownership. Use line count as a warning; a small fix in a large file does not require unrelated reorganization.
- Retire old paths with their authorized consumers. Avoid migration-only shims or re-exports; user-required coexistence remains an explicit exception.
- Use `src/tests/` for crate and public-surface unit tests. Module-local `tests.rs` or `tests/mod.rs` can cover private helpers.
- Comment key invariants and non-obvious control flow. Avoid comments that merely repeat code.

## Select by changed boundary

| Current edit | Reference |
| --- | --- |
| Create or reorganize production modules | [Module layering](structure/module-layering.md) |
| Change root wiring or binding surfaces | [Root entry rules](structure/root-entry-files.md) |
| Change behavior or retire compatibility paths | [Refactor rules](refactor-rules.md) |
| Cross crate boundaries or shared runtime contracts | [Workspace map](workspace-map.md) and [architecture](../../zircon-project-skills/zr-architecture-first-engineering/SKILL.md) |
| Reorganize unit-test trees | [Test ownership](testing/mod-rs-map.md) |
| Add a responsibility to a growing file | [Module boundaries](../../zircon-project-skills/zr-module-boundary-discipline/guide.md); [large-file guidance](../../zircon-project-skills/modularize-large-files/guide.md) when needed |

Open only the applicable references and reuse unchanged context. [The validation policy](../../../../docs/plans/milestone-validation-policy.md) controls Cargo cadence; editing or writing a test does not itself require an immediate build.
