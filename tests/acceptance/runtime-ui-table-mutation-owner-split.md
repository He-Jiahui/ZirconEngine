# Runtime UI table mutation owner split acceptance

## Scope

Runtime 15 structure-convention repair for the default table interaction family. The slice moves table property mutation behavior out of the route root and into:

`zircon_runtime/src/ui/surface/surface/default_interactions/table/mutation.rs`

## Baseline problem

`table/mod.rs` owned pointer routing, resize and sort orchestration, table state mutation, and role predicates in one 677-line module. That mixed multiple behavior families in a root module and violated the repository's module-owner and root-file size direction.

## Required invariants

- `table/mod.rs` remains the table route and orchestration owner.
- `table/mutation.rs` owns all six table mutation methods.
- Property names and `UiValue` payload shapes are unchanged.
- Rejected property mutations still return `false` and do not publish binding updates.
- Accepted property mutations still append the returned binding report exactly once.
- Sibling table modules can use the shared mutation entry without widening it outside the table family.
- `table/mod.rs` remains below 540 lines and `table/mutation.rs` remains below 240 lines.

## Design references

- Fyrox separates grid, list, tree, and scroll behavior into named UI modules under `dev/Fyrox/fyrox-ui/src`.
- Godot separates tree, item-list, grid-container, and scroll behavior into named owners under `dev/godot/scene/gui`.
- The implementation follows that behavior-family separation while retaining Zircon's existing retained-surface contracts.

## Test inventory

- `tools/tests/test_runtime_ui_table_module_structure.py`
- Rust formatting checks for `table/mod.rs` and `table/mutation.rs`
- Focused `zircon_runtime` compilation and relevant table interaction tests when the shared workspace baseline permits them
- Scoped diff and stale-owner checks

## Evidence

- `python -m unittest tools.tests.test_runtime_ui_table_module_structure`: passed 2/2.
- `rustfmt --edition 2021 --check` for `table/mod.rs` and `table/mutation.rs`: passed.
- Scoped `git diff --check`: passed; only Git's existing LF/CRLF checkout warnings were emitted.
- Windows `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1`: passed in 51.72 seconds with 417 existing warnings.
- Windows `cargo test ... table_pointer_routes`: two runs reached the 304-second command limit without diagnostics or test results; both are recorded as timeout/no-result.
- WSL `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 2 ... table_pointer_routes`: the cold build reached the library-test target, then failed before running filtered tests on active out-of-scope text-layout work at `zircon_runtime/src/ui/text/layout_engine/visual_order.rs:79` with `E0282` (`Vec<_>` type annotation required). The blocker is not part of this slice and is owned by the active Runtime text-layout session.
- Post-split source sizes: `table/mod.rs` 497 lines; `table/mutation.rs` 194 lines.

## Decision

The owner split is implemented and passes its structure, formatting, diff-health, and runtime library compilation gates. The behavioral regression gate is blocked before test execution by the unrelated active text-layout compile error, so this slice remains `in_progress` and is not recorded as fully accepted or complete.
