# ZirconEngine Unit-Test Skeleton Wave 1

Use this reference when creating the first empty test tree. Wave 1 is limited to the shared, lower-layer runtime-internal surfaces plus the three root packages:

1. `zircon_runtime::core::math`
2. `zircon_runtime::core::runtime`
3. `zircon_runtime::core::manager`
4. `zircon_runtime::core::framework`
5. `zircon_runtime`
6. `zircon_editor`
7. `zircon_app`

Create the directories and empty files first. Fill assertions after the tree compiles and the parent hooks are in place.

## `zircon_runtime::core::math`

```text
zircon_runtime/
  src/
    core/
      math/
        tests/
          mod.rs
          transform.rs
          projection.rs
          viewport.rs
```

`zircon_runtime/src/core/math/tests/mod.rs`

```rust
mod transform;
mod projection;
mod viewport;
```

## `zircon_runtime::core::runtime`

```text
zircon_runtime/
  src/
    core/
      runtime/
        tests/
          mod.rs
          registry_name.rs
          descriptors.rs
          registration.rs
          activation.rs
          resolution.rs
          handle_facades.rs
```

`zircon_runtime/src/core/runtime/tests/mod.rs`

```rust
mod registry_name;
mod descriptors;
mod registration;
mod activation;
mod resolution;
mod handle_facades;
```

## `zircon_runtime::core::manager`

```text
zircon_runtime/
  src/
    core/
      manager/
        tests/
          mod.rs
          resolver.rs
          service_names.rs
```

`zircon_runtime/src/core/manager/tests/mod.rs`

```rust
mod resolver;
mod service_names;
```

## `zircon_runtime::core::framework`

```text
zircon_runtime/
  src/
    core/
      framework/
        tests/
          mod.rs
          animation.rs
          asset.rs
          foundation.rs
          input.rs
          physics.rs
          render.rs
          scene.rs
          script.rs
          ui.rs
```

`zircon_runtime/src/core/framework/tests/mod.rs`

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

- `zircon_runtime/src/tests/mod.rs`
- `zircon_editor/src/tests/mod.rs`
- `zircon_app/src/tests/mod.rs`

Keep these trees package-oriented, but route shared contract coverage down into `zircon_runtime/src/core/**/tests/` whenever the behavior belongs to a runtime-internal layer.

## Parent Hook Checklist

Update these files before filling test bodies:

- `zircon_runtime/src/core/math/mod.rs`
- `zircon_runtime/src/core/runtime/mod.rs`
- `zircon_runtime/src/core/manager/mod.rs`
- `zircon_runtime/src/core/framework/mod.rs`
- `zircon_runtime/src/lib.rs`
- `zircon_editor/src/lib.rs`
- `zircon_app/src/lib.rs`

Each should expose:

```rust
#[cfg(test)]
mod tests;
```

## Wave 1 Exit Condition

Wave 1 is ready for real assertions only after:

- the test trees above exist,
- the parent hooks compile,
- existing inline root test blocks in `zircon_runtime::core::manager` and `zircon_runtime::core::framework` have been removed or migrated,
- new test files stay grouped by subsystem instead of collapsing back into a single file.
