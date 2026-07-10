# ZirconEngine Unit-Test Tree Map

Use this reference when replacing inline `mod tests { ... }` blocks with structured test trees.

## Layout Rules

- Default to one package-level `src/tests/` tree per root package and one `tests/` tree per substantial runtime-internal layer.
- Keep `mod.rs` files navigational only. Put assertions in child files or child folders.
- Group similar test areas into subfolders with their own `mod.rs`.
- Use module-local `tests.rs` or `tests/mod.rs` only when you need direct access to private helpers inside a source module.
- Replace large inline test blocks with `#[cfg(test)] mod tests;` and move the bodies out.

## Runtime-Internal Test Trees

### `zircon_runtime::core::math`

- Parent hook: `zircon_runtime/src/core/math/mod.rs`
- `zircon_runtime/src/core/math/tests/mod.rs`

```rust
mod transform;
mod projection;
mod viewport;
```

### `zircon_runtime::core::runtime`

- Parent hook: `zircon_runtime/src/core/runtime/mod.rs`
- `zircon_runtime/src/core/runtime/tests/mod.rs`

```rust
mod registry_name;
mod descriptors;
mod registration;
mod activation;
mod resolution;
mod handle_facades;
```

### `zircon_runtime::core::manager`

- Parent hook: `zircon_runtime/src/core/manager/mod.rs`
- `zircon_runtime/src/core/manager/tests/mod.rs`

```rust
mod resolver;
mod service_names;
```

### `zircon_runtime::core::framework`

- Parent hook: `zircon_runtime/src/core/framework/mod.rs`
- `zircon_runtime/src/core/framework/tests/mod.rs`

```rust
mod animation;
mod asset;
mod foundation;
mod input;
mod physics;
mod render;
mod scene;
mod script;
mod ui;
```

## Root Package Test Trees

### `zircon_runtime`

- Parent hook: `zircon_runtime/src/lib.rs`
- `zircon_runtime/src/tests/mod.rs`

```rust
mod builtin;
mod foundation;
mod asset;
mod graphics;
mod input;
mod scene;
mod script;
mod ui;
mod extensions;
```

### `zircon_editor`

- Parent hook: `zircon_editor/src/lib.rs`
- `zircon_editor/src/tests/mod.rs`

```rust
mod editing;
mod host;
mod scene;
mod ui;
mod workbench;
```

### `zircon_app`

- Parent hook: `zircon_app/src/lib.rs`
- `zircon_app/src/tests/mod.rs`

```rust
mod entry;
mod runtime_presenter;
```

## Module-Local Exceptions

Use module-local test folders when private helpers are the real unit-test surface. Keep those exceptions inside the owning `zircon_runtime` or `zircon_editor` module tree rather than reviving deleted top-level crates.

## Do Not Regress To

- One giant `lib.rs` test block per package
- Flat `src/tests/*.rs` dumps once a package or runtime-internal layer grows multiple test domains
- Mixing unrelated subsystems inside one test file just because they compile together
