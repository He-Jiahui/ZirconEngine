---
related_code:
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/keymap_overrides.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/module.rs
plan_sources:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
tests:
  - zircon_editor/src/core/commands/keymap/tests.rs
  - zircon_editor/src/core/settings/tests.rs
  - zircon_editor/src/tests/editor_event/runtime/keymap_settings.rs
doc_type: module-detail
---

# Editor Keymap Settings

## Purpose

The built-in TOML asset remains the immutable default keymap. `EditorKeymap` parses that asset,
builds the chord signature index, and applies a typed `EditorKeymapOverrides` delta. It does not
own a private persistence format.

## Settings Authority

`EDITOR_KEYMAP_OVERRIDES_KEY` is registered in `SettingsRegistry` with
`SettingSchema::KeymapOverrides` and User scope. The shared registry resolves Session, Project,
User, then default precedence before `editor_keymap_overrides()` returns the effective typed delta.

`SettingsStore` is the only disk owner. It writes the strict `zircon.editor.settings` versioned
envelope at the configured User or Project path and rejects unwrapped or migrated legacy payloads
atomically. The retired `zircon.editor.keymap-user-layer` document, its v0 fixture, and
`commands/keymap/persistence.rs` are not compatibility inputs.

`EDITOR_KEYMAP_NAME` resolves to `EditorKeymapService`, a dynamic projection over the same
`SettingsAuthority`. The service refreshes only when the immutable override payload changes, so a
previously resolved manager service and retained-host keyboard dispatch observe later overrides
without retaining a startup keymap as a second authority.

## Override Semantics

`EditorKeymapOverrides` is a sorted map from validated `EditorOperationPath` to an optional
`EditorKeyChord`. `Some(chord)` adds or replaces a binding; `None` is an explicit tombstone.
Omitted commands continue to inherit the built-in preset. `EditorKeymap::with_overrides()`
materializes the effective sorted bindings and rebuilds the lookup index without reconstructing a
second persistence document.

The Rust tests cover registry precedence, current settings round-trip, rejection of the retired
private document, tombstone behavior, conflicts, keyboard fallback, and the indexed hot path.
Editor08 still owns when-domain routing and product conflict presentation; Editor17 owns settings
page contribution and hot application.
