# Sound Dynamic Event Execution Design

## Summary

Add sound-runtime execution for registered dynamic event handlers. The current runtime can register event descriptors, register handler descriptors, submit invocations, and produce deterministic delivery DTOs. This slice adds a runtime-local executor registry so actual Rust handler callbacks can run in deterministic order while preserving the neutral dispatch/report contracts.

## Scope

This slice covers:

- neutral execution result/report DTOs,
- a sound-runtime executor registry keyed by `(plugin_id, handler_id)`,
- manager APIs to register/unregister executors and execute pending dynamic events,
- deterministic execution ordering using the existing handler priority/plugin/handler ordering,
- per-handler success/failure/skipped reporting,
- cleanup when events or handlers are unregistered,
- focused runtime tests and documentation updates.

This slice does not cover:

- stable ABI callbacks through `zircon_runtime_interface`,
- dynamic-library plugin callback invocation,
- editor operation routing for sound events,
- async event execution or thread-pool scheduling.

## Architecture

`zircon_runtime::core::framework::sound` owns serializable execution report DTOs and trait API shape. It must not own Rust callback trait objects.

`zircon_plugins/sound/runtime` owns callback storage because it is concrete runtime behavior. Executors are stored in `SoundEngineState` as runtime-local closures keyed by plugin and handler ID. The existing descriptor list remains the discoverable contract; executors are executable implementations for those descriptors.

## Execution Flow

1. A plugin registers a `SoundDynamicEventHandlerDescriptor` for a known event.
2. The same plugin registers an executor callback for `(plugin_id, handler_id)`.
3. Callers submit `SoundDynamicEventInvocation` values as before.
4. `execute_dynamic_events()` drains pending invocations, computes deterministic deliveries, and executes matching callbacks.
5. Each delivery produces one execution result: succeeded, failed with error text, or skipped because no executor was registered.
6. One failing handler does not stop later handlers.

## Error Handling

Executor registration fails if the target handler descriptor is not registered. Unregistering a missing executor returns the same typed missing handler error used for handler descriptors. Handler execution failures are captured in reports instead of failing the whole batch. Event/handler unregister cleanup removes matching executors and pending invocations so stale callback keys do not survive descriptor removal.

## Testing

Add focused runtime tests for:

- executor registration requires an existing handler,
- execution invokes handlers in deterministic priority/plugin/handler order,
- missing executors are reported as skipped,
- failing executors record failure and later handlers still run,
- unregistering handlers/events removes executors and pending work.

Validation should run sound runtime formatting, neutral sound rustfmt, focused dynamic event tests when possible, and whitespace checks. If unrelated workspace code blocks Cargo before sound compilation, record the exact external diagnostic.

## Remaining Follow-Up After This Slice

After this slice, dynamic events have local runtime code execution. The remaining dynamic event gap becomes ABI/dynamic-library plugin callback execution and editor-host operation routing.
