# ZirconEngine Source Module Layering

Use this reference when creating or reorganizing non-test Rust modules in the workspace.

Also read `root-entry-files.md` when the touched file is `binding.rs`, `lib.rs`, `main.rs`, or production `mod.rs`.

## Core Rules

- One functionality-level module declaration belongs in one file.
- If a module grows multiple coherent responsibilities, convert it into a folder-backed subtree immediately instead of adding more sections to the same file.
- Group similar modules into folders so the tree shows subsystem structure at a glance.
- Keep crate roots (`lib.rs`, `main.rs`) and intermediate `mod.rs` files mostly navigational.
- Keep `binding.rs` under the same discipline as other root entry files: boundary surface only, real behavior below it.
- Do not use flat accumulation as the default. A growing subsystem should become a directory, not a longer file list or a larger root file.

## What Stays In Root Files

These files should stay short and structural whenever possible:

- `binding.rs`
- `lib.rs`
- `main.rs`
- `<subsystem>/mod.rs`

They may contain:

- `mod ...;`
- `pub use ...;`
- small glue helpers
- minimal crate or subsystem entry wiring

They should not become the long-term home for mixed business logic, parsing, state mutation, I/O, rendering, orchestration, and helper code all together.

For the hard per-file red lines and target shapes, see `root-entry-files.md`.

## Default Growth Pattern

Start simple:

```text
src/
  lib.rs
  state.rs
  module.rs
```

When one concern splits into multiple parts, convert it:

```text
src/
  lib.rs
  state/
    mod.rs
    snapshot.rs
    mutation.rs
    selection.rs
```

When a subsystem has multiple neighboring concerns, group them:

```text
src/
  lib.rs
  runtime/
    mod.rs
    registry.rs
    activation.rs
    resolution.rs
    handles.rs
```

## Folder-First Triggers

Move from a single file to a folder when any of these become true:

- the file now contains multiple behavior clusters,
- different edits regularly touch different regions of the same file,
- the module needs both public API wiring and several internal helpers,
- an entry file starts collecting parser, router, or executor behavior,
- the file name is becoming a fake umbrella for unrelated logic,
- the easy next step is “append one more section”.

Do not wait for the file to become huge before splitting it. Structure drift is already a problem before line count becomes extreme.

## Naming Guidance

- Name modules by behavior or role, not by vague convenience.
- Prefer `registry.rs`, `activation.rs`, `selection.rs`, `project_io.rs`, `render_extract.rs`.
- Avoid broad catch-all names like `util.rs`, `helpers.rs`, `misc.rs`, or `common.rs` unless that module has one tight responsibility.
- If several files share the same domain prefix, that is usually a folder signal.

## Recommended Subtree Shapes

### Public subsystem with internal parts

```text
src/
  asset/
    mod.rs
    manager.rs
    worker_pool.rs
    requests.rs
    builtins.rs
```

### State-heavy subsystem

```text
src/
  editor/
    mod.rs
    state/
      mod.rs
      bootstrap.rs
      intents.rs
      viewport.rs
      project_io.rs
```

### Runtime or pipeline subsystem

```text
src/
  runtime/
    mod.rs
    descriptors.rs
    registration.rs
    activation.rs
    resolution.rs
```

## Anti-Patterns

- a large `lib.rs` that owns most of the crate logic,
- a `binding.rs`, `lib.rs`, `main.rs`, or `mod.rs` file that implements more than it declares,
- one file containing parsing, validation, execution, and persistence together,
- flat sibling files for every small variant with no grouping,
- repeated “temporary” helper dumping into catch-all modules,
- adding tests with better structure than the production modules they are testing.

## Alignment Rule

Test trees should reflect production structure, and production structure should be at least as disciplined as the test tree. Do not create a carefully layered `src/tests/` hierarchy while leaving the real implementation flattened into broad umbrella files.
