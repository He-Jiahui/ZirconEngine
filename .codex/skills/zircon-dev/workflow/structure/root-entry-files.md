# Root Entry Files

Use this reference when touching or creating `binding.rs`, `lib.rs`, `main.rs`, or production `mod.rs` files in `zirconEngine`.

## Core Rule

Treat these files as structural entry surfaces, not as implementation homes.

They exist to expose module boundaries, wire narrow entry points, and make the tree easy to navigate. If meaningful behavior starts living here, the boundary is already drifting.

## Allowed Content

These files may contain:

- `mod ...;`
- `pub use ...;`
- minimal type aliases
- narrow constructors or pass-through entry helpers
- minimal registration or bootstrap calls that only hand work to child modules
- small crate- or subsystem-level doc comments when they improve navigation

## Prohibited Content

These files must not become long-term homes for:

- payload, config, or string parsing
- routing, dispatch tables, or command matching
- state mutation, orchestration, or workflow branching
- filesystem, network, rendering, or asset I/O behavior
- serialization, conversion, formatting, or schema translation
- multiple domain-specific enums, structs, or errors
- large `impl` blocks or helper clusters
- subsystem-specific business logic hidden behind “just glue”

If the file needs scrolling to explain what it really owns, the content belongs below it.

## File-Specific Rules

### `binding.rs`

Use `binding.rs` only as the visible boundary surface for a binding subsystem.

It may contain:

- child module declarations
- public re-exports
- one narrow façade type or thin handoff function when needed

It must not contain:

- payload parsing
- string-to-command mappings
- per-command enums for several domains
- routing logic
- encode or decode helpers
- host or UI state mutation

Target shape:

```text
src/
  binding.rs
  binding/
    payload.rs
    router.rs
    asset/
      asset_command.rs
      encode.rs
      decode.rs
    viewport/
      viewport_command.rs
      encode.rs
      decode.rs
```

### `lib.rs`

Use `lib.rs` as the crate surface, not as the crate body.

It may contain:

- top-level module declarations
- curated public re-exports
- the smallest crate entry wiring needed to expose the crate contract

It must not contain:

- most of the crate's implementation
- large subsystem constructors
- domain-specific branching and orchestration
- internal helpers that only exist because child modules were not created

Target shape:

```text
src/
  lib.rs
  runtime/
    mod.rs
    registration.rs
    activation.rs
  asset/
    mod.rs
    manager.rs
    import.rs
```

### `main.rs`

Use `main.rs` as the executable bootstrap shell.

It may contain:

- argument handoff to a child CLI or app module
- top-level startup wiring
- a narrow error boundary

It must not contain:

- full CLI parsing trees
- service graph construction for multiple subsystems
- runtime workflow logic
- editor or gameplay behavior
- subsystem-specific fallback branches

Target shape:

```text
src/
  main.rs
  app/
    mod.rs
    startup.rs
    run.rs
  cli/
    mod.rs
    parse.rs
```

### `mod.rs`

Use `mod.rs` as the folder boundary file for one subsystem.

It may contain:

- child module declarations
- public re-exports
- one narrow subsystem entry helper when it only delegates downward

It must not contain:

- mixed implementation from several child concerns
- behavior that belongs in `parse.rs`, `route.rs`, `snapshot.rs`, `mutation.rs`, or similar behavior files
- large sibling-specific helpers copied into the folder root

Target shape:

```text
src/
  world/
    mod.rs
    bootstrap.rs
    query.rs
    project_io.rs
    render.rs
```

## Red Lines That Force Downward Extraction

Move content out of the entry file immediately when any of these become true:

- the file owns more than one behavior family
- the file names or matches several domain variants
- the next edit adds “one more section” instead of one more child module
- helpers start sharing a prefix that obviously wants its own folder
- declarations and implementations are growing together in the same file
- the file is acting as parser, router, and executor at once
- reviewers need the file body to understand subsystem ownership

Do not wait for line count. A 150-300 line entry file with mixed roles is already wrong.

## Common Anti-Patterns

- `binding.rs` quietly owns asset, dock, viewport, and inspector command behavior.
- `lib.rs` contains most of the crate because “the crate is still small.”
- `main.rs` becomes a dumping ground for bootstrap plus real runtime logic.
- `mod.rs` grows into a hidden umbrella implementation because the folder already existed.
- a root file keeps accumulating private helpers that no caller outside the file should ever know about.

## Review Questions

Before accepting an entry file, ask:

- Can a reviewer understand the subsystem boundary from the path and file names alone?
- Does this file mostly declare and delegate, or does it secretly implement?
- Could the next feature land as a new child module without reopening unrelated logic here?
- If this subsystem grows 10x, would this file still stay short?

If any answer is no, extract downward before continuing.
