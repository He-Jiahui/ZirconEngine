---
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/jobs/quota_settings.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane/shutdown.rs
implementation_files:
  - zircon_editor/src/core/settings/startup.rs
  - zircon_editor/src/core/settings/registry.rs
  - zircon_editor/src/core/settings/change_log.rs
  - zircon_editor/src/core/settings/persistence.rs
  - zircon_editor/src/core/settings/io.rs
plan_sources:
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor/17/failure-2026-07-30-editor-settings-persistence-and-hot-projection.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-23-settings-registry-job-category-quota-migration.md
tests:
  - zircon_editor/src/core/settings/tests/
  - zircon_editor/src/core/jobs/tests/quota_settings_contract.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
doc_type: module-detail
---

# Editor Settings

`zircon_editor::core::settings` owns the editor's typed, layered settings
contract. `SettingsAuthority` is the sole mutable owner; retained-host,
workbench, EditorManager, and viewport code consume its immutable snapshots or
bounded deltas. UI panels never create a registry copy or own persistence.

## Authority And Snapshots

Definitions have a validated lowercase dot-separated `SettingsKey`, a maximum
durable scope, a `SettingSchema`, default value, restart flag, and
locale-neutral `SettingsPresentation`. Presentation stores validated label,
description, and non-empty category localization keys; it never stores a
localized display string or slash-separated category text. The authority keeps
those keys opaque and a UI projection resolves them through one captured
`EditorI18nService` locale. Resolution precedence is Session, Project, User,
then the definition default. A definition scope restricts the durable layers it
may occupy:

| Definition scope | Allowed writes |
| --- | --- |
| User | User, Session |
| Project | User, Project, Session |
| Session | Session |

`SettingsAuthority` validates mutations, publishes one immutable generation
snapshot, and exposes pre-parsed typed slots for design tokens, keymap
overrides, command-palette MRU, and viewport snap steps. Stable consumers must
not parse settings keys, walk layered maps, or allocate projection values.

`SettingsRegistry` is the authority's internal mutable representation. A
no-op write neither advances the generation nor emits a change. Consumers that
need changes use a cursor/delta API backed by entry, byte, and age budgets; a
cursor that falls behind receives `requires_snapshot` rather than retaining an
unbounded history.

At editor construction, the context composition root creates the one product
registry and registers each subsystem's built-in definitions. It passes that
registry to generic `SettingsStartup`, which loads the User layer atomically
and retains a typed `Loaded`, `Missing`, or `Invalid` provenance. The
composition root then resolves the complete restart-only job quota set once
from the loaded registry and runtime scheduler parallelism before moving the
same registry into `SettingsAuthority`. The current `EditorJobSystem` never
hot-applies quota changes; persisted changes become admission policy only in
the next editor context.

Project sources are loaded through the authority and cached for the active
binding. Switching or clearing a project invalidates that binding before the
next project source is read. Invalid User or Project source data is recorded as
invalid and does not partially replace a valid layer.

## Persistence And Shutdown

`SettingsStore` persists only User and Project layers:

- User: `~/.zircon/editor/settings.toml`.
- Project: `<project-root>/.zircon/settings.toml`.
- Session: never written to disk.

`ZIRCON_EDITOR_APPEARANCE_PREFERENCES` is the User-root override. Its value is
a directory root, not a settings-file path; a file is rejected. The settings
file is the Plan11 versioned envelope with schema id `zircon.editor.settings`
and schema version 1. Legacy or unwrapped payloads fail closed. Writes use the
shared atomic writer, including durable file replacement where supported.

`SettingsPersistenceService` converts a typed changed key, scope, and
generation into Runtime11 bounded keyed I/O. Its lane identity includes the
persistent target path, so the same key for different project roots cannot
coalesce. The worker serializes the authority's currently bound layer and does
the filesystem work; callers on the UI or frame path only receive a ticket.

A failed ticket may be retried only as the same typed request. Project changes
cancel unstarted tickets and the authority verifies the target binding before a
worker writes, preventing a stale Project A request from serializing Project B
state into A. At host shutdown, `flush_then_shutdown().finish()` must complete:
it returns success only when its fence succeeds, so a final persistence failure
cannot become a successful editor exit.

## Extension Rules

New structured settings add a typed `SettingValue` and `SettingSchema`; they
must not embed JSON or TOML inside `SettingValue::String`. New consumers use
the authority snapshot or its bounded delta contract and must not introduce a
second settings cache, reader, scheduler, fallback parser, or persistence path.
