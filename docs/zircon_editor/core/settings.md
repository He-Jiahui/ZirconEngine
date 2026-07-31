# Editor Settings

`zircon_editor::core::settings` owns the editor's typed, layered settings contract. It is usable by retained-host and headless code; UI panels are consumers, not persistence owners.

## Registry

Register a `SettingDefinition` before reading or writing a value. A definition has a validated lowercase dot-separated `SettingsKey`, a maximum durable scope, a `SettingSchema`, default value, restart flag, and slash-separated category path.

`SettingsRegistry::resolve` always uses this precedence:

| Priority | Layer |
| --- | --- |
| 1 | Session |
| 2 | Project |
| 3 | User |
| 4 | Definition default |

The definition scope is the highest durable layer the setting may occupy:

| Definition scope | Allowed writes |
| --- | --- |
| User | User, Session |
| Project | User, Project, Session |
| Session | Session |

Every write validates the schema and emits a `SettingChange` containing the key, written scope, monotonic revision, and `requires_restart`. Loading a durable layer validates all values before replacing that layer, so an unknown key, invalid value, or forbidden scope leaves the prior layer unchanged.

## Persistence

`SettingsStore` persists only User and Project layers:

- User: `~/.zircon/editor/settings.toml`.
- Project: `<project-root>/.zircon/settings.toml`.
- Session: never written to disk.

`ZIRCON_EDITOR_APPEARANCE_PREFERENCES` remains the User-root override required by the editor architecture plan. Its value is a directory root, not a settings-file path. An existing file value is rejected; no old appearance path is reinterpreted.

The `.toml` file name is the planned location. Its contents are the canonical Plan11 versioned text envelope with schema id `zircon.editor.settings` and schema version 1, not a separately parsed TOML dialect. Version zero and unwrapped/legacy payloads fail closed. There is no old preferences reader, migration, fallback parser, or dual write path.

Writes create a unique file in the target directory, flush its contents, atomically replace the target, and synchronize the parent directory where the platform supports it. A failed read at retained-host startup logs a warning and resolves registered defaults; it never resurrects the retired preferences module.

## Built-in Setting

The current built-in definition is `editor.appearance.design_tokens`:

- Scope: User with a Session override permitted by the registry.
- Schema/value: strong `EditorDesignTokens`, never a JSON string.
- Default: `EditorDesignTokens::workbench_dark()`.
- Startup: `editor_design_tokens_at_startup()` loads the User layer and applies the resolved token set before the retained host is created.

## Extension Rules

New settings must add a typed `SettingValue`/`SettingSchema` pair when an existing scalar domain is insufficient. Do not store structured configuration as JSON or TOML text inside `SettingValue::String`.

The following first-batch migrations remain open in their owning plans and must use this registry/store rather than add a private persistence path:

- Editor08 keymap user layer: `docs/plans/zircon_editor/editor/08/failure-2026-07-23-settings-registry-keymap-user-layer-migration.md`.
- Editor05 project snap steps: `docs/plans/zircon_editor/editor/05/failure-2026-07-23-settings-registry-project-snap-step-migration.md`.
- Editor14 job-category quotas: `docs/plans/zircon_editor/editor/14/failure-2026-07-23-settings-registry-job-category-quota-migration.md`.
- Editor13 script-build batch window: `docs/plans/zircon_editor/editor/13/failure-2026-07-23-settings-registry-script-build-batch-window-migration.md`.

Each migration must prove current-shell round-trip, schema rejection, scope precedence, and removal of its prior private file I/O or configuration fallback.
