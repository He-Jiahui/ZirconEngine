---
related_code:
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
implementation_files:
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
---

# Template Action Invocation Contract

`UiTemplateActionInvocation` keeps canonical editor commands distinct from parameterized routes. Its serialized target is `{ kind, id }`:

- `kind = "action"` names one registered editor command and carries no payload.
- `kind = "route"` names a local or operation route and may carry typed payload values.

This is a hard-cut contract. The previous flat `{ route, payload }` invocation shape is not a compatibility input, and command actions must not retain a route alias or a route payload.

Asset document validation requires exactly one non-empty target, rejects an action reference that declares both targets, and rejects payload on a command action. A route action may still carry typed payload. This structural gate runs before host side-effect policy reporting, so malformed plugin and runtime assets cannot load and then degrade to a silent no-op when clicked.

The interface crate owns the typed target declaration and serde shape. Runtime projection preserves the authored target kind. Editor dispatch turns action targets into `EditorCommand` bindings and routes them through the command registry's existence, capability, and `when` policy before executing the matching event or operation. Route targets continue through parameterized operation dispatch.

`zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs` covers both target kinds: route roundtrip preserves typed payload values, while action roundtrip preserves the command id and omits an empty payload. Editor retained-host tests prove that an action reaches the canonical command path and that a disabled command records the registry failure instead of executing or falling back to a route alias.
