---
related_code:
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/keymap/persistence.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
  - tests/fixtures/serialization/editor-keymap-user-layer/v0/keymap-user-layer.json
plan_sources:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
tests:
  - zircon_editor/src/core/commands/keymap/tests.rs
  - tools/tests/test_editor11_keymap_version_shell_contract.py
doc_type: module-detail
---

# Editor Keymap Persistence

## Purpose

`EditorKeymap` combines an immutable built-in keymap with a persisted user delta. The built-in TOML
asset remains the default owner. The user file stores only changed command bindings and explicit
unbinds, so new built-in commands remain inherited after an upgrade.

## Wire Contract

The user layer uses the shared `$zircon` canonical JSON envelope with schema id
`zircon.editor.keymap-user-layer` and schema version 1. Its payload is a sorted `bindings` map:

- a chord string adds or replaces one command binding;
- `null` explicitly unbinds a command inherited from the built-in layer;
- an omitted command continues to inherit the built-in value.

The payload never carries a second `version` field. An unwrapped payload with the same `bindings`
shape is version 0 and migrates through the explicit `MigrationChain`; the historical shape is
locked by a repository fixture. A future envelope version is rejected rather than interpreted as
the current format.

## Load And Save

`EditorKeymap` owns typed base bindings and a separate typed user delta. Effective bindings are a
sorted derived projection; serialization reads the delta owner directly instead of reconstructing
intent from the effective map. This preserves an explicit `null` tombstone while a plugin command
is absent, so the command remains unbound if a later plugin version restores its default.

`apply_user_layer` validates every command id through `EditorOperationPath` and parses every chord
before constructing a replacement keymap. One invalid id or chord rejects the complete layer and
leaves the base keymap unchanged. Path save always emits the latest canonical envelope; path load
returns the effective keymap and `migrated_from` so the future settings owner can request a
canonical resave after migration. Load, write, and I/O errors retain their typed sources, allowing
the settings owner to distinguish a missing user file from corruption or a future schema.

The current slice owns persistence and layering only. Conflict diagnostics and when-domain routing
remain Editor08 M2.2 work; the future Editor17 settings registry owns the user path and change event.
