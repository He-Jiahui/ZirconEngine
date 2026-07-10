# `binding.rs` Anti-Pattern Reference

Use this reference when a binding or root module has started collecting multiple unrelated subsystems.

## Why `zircon_editor/src/ui/binding.rs` Is A Red Flag

As of 2026-04-20, `zircon_editor/src/ui/binding.rs` is the kind of file that becomes dangerous once it starts collecting several editor UI domains. The line count matters less than the shape:

- `SelectionCommand`
- `AssetCommand`
- `WelcomeCommand`
- `DraftCommand`
- `InspectorFieldChange`
- `DockCommand`
- `ViewportCommand`
- `EditorUiBindingPayload`
- `EditorUiBinding`
- `EditorUiRouter`
- payload parsing helpers
- enum/string conversion helpers

That is not one responsibility. It is several subsystems and several behavior families collapsed into one file:

- asset-related behavior
- inspector/draft behavior
- dock behavior
- viewport behavior
- binding payload representation
- parsing and encoding helpers
- routing
- error handling

This is exactly the kind of module this skill exists to prevent.

## Required Boundary Corrections

- `binding.rs` itself must become a thin wiring file.
- Each top-level declaration gets its own file.
- Similar declarations live under a domain folder, not flat beside unrelated concerns.
- Parsing, encoding, decoding, name mapping, and routing move out of declaration files once they stop being trivial.

## One Acceptable Target Shape

```text
zircon_editor/src/ui/
  binding.rs
  binding/
    editor_ui_binding.rs
    editor_ui_binding_error.rs
    editor_ui_router.rs
    payload.rs
    selection/
      selection_command.rs
      encode.rs
      decode.rs
    asset/
      asset_command.rs
      encode.rs
      decode.rs
    welcome/
      welcome_command.rs
      encode.rs
      decode.rs
    draft/
      draft_command.rs
      encode.rs
      decode.rs
    inspector/
      inspector_field_change.rs
      encode.rs
      decode.rs
    dock/
      dock_command.rs
      encode.rs
      decode.rs
    viewport/
      viewport_command.rs
      display_mode.rs
      grid_mode.rs
      projection_mode.rs
      transform_space.rs
      view_orientation.rs
      encode.rs
      decode.rs
```

This is an example shape, not the only legal tree. The non-negotiable part is the boundary discipline:

- root `binding.rs` stays structural,
- each declaration owns one file,
- behavior families get their own files,
- domain folders expose subsystem ownership directly.

## Review Questions

Before accepting a module layout, ask:

- Can a reviewer tell asset code from viewport code by path alone?
- Can a new viewport command land without reopening dock or asset files?
- Does `binding.rs` still contain real logic, or only wiring?
- Are declarations isolated from parsing/encoding helpers?

If any answer is "no", keep splitting.
