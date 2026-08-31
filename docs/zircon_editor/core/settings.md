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
  - zircon_editor/src/core/settings/mutation.rs
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

Numeric schemas own their minimum, maximum, and positive step. The shared
schema operation applies checked integer stepping or finite float
quantization/clamping; UI surfaces select only decrement or increment and must
not switch on setting keys to recover step constants.

Color schemas similarly own a positive `channel_step`. A typed
`SettingColorChannel` selects Red, Green, Blue, or Alpha and the shared schema
operation applies one saturating `u8` step. UI surfaces do not own color bounds,
step constants, or key-specific color branches.

`SettingsAuthority` validates mutations, publishes one immutable generation
snapshot, and exposes pre-parsed typed slots for design tokens, keymap
overrides, command-palette MRU, and viewport snap steps. Stable consumers must
not parse settings keys, walk layered maps, or allocate projection values.

The immutable `SettingsCatalog` also builds one locale-neutral category-path
index when the authority is created. A Settings Workbench category resolves its
direct keys through that index, then calls `resolved_settings` once. The
resulting `ResolvedSettingsBatch` freezes the exact generation, typed values,
and precedence sources under one authority lock. Retained conversion and paint
consume only that batch; they never query the authority per row or during a
frame.

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

`SettingsMutationCoordinator` is the Context-owned mutation and source-binding
boundary. It retains the writable User source, the active Project binding
generation, the authority, and the persistence service. Product `set`, `clear`,
and command-palette MRU mutations use this owner. Persistent writes are rejected
before changing authority state when their source is unavailable, unbound, or
invalid. Viewport and Workbench code do not own stores or persistence tickets.

The mutation receipt distinguishes unchanged, Session-applied,
persistence-queued, and applied-but-not-admitted states. The coordinator retains
at most the User document and one active Project document. Deferred admission
and failed tickets remain available to typed retry; shutdown refuses to report
success while a deferred document has never entered the I/O lane.

The coordinator also owns a fixed User/Project persistence-health snapshot.
Every submit or retry gets an observation token before Runtime11 admission, and
the lane's terminal observer updates health only while both the physical
document identity and token are current. Project switching therefore rejects
late callbacks from the retired binding. Health subscribers receive an
immutable snapshot after coordinator locks are released; they never retain a
ticket or poll lane diagnostics. Pending admission and worker write failure are
projected to the context notification authority. The Settings title bar consumes
the same immutable health generation and shows a retry icon only for retryable
Project/User states. Its action carries only the scope and calls
`retry_pending(scope)`; the retained host does not own or reconstruct a
persistence ticket. Failure notifications and health attributes share one
prepared retained-tree refresh, and stable notification frames do not poll
health or lane diagnostics.

`SettingsPersistenceService` converts a typed changed key and scope into
Runtime11 bounded keyed I/O. Before authority mutation, the coordinator reserves
a process-monotonic `SettingsFileGeneration` for the validated physical target.
Its lane identity is the scope and physical target path, not the setting key,
and lane ordering uses this file generation rather than an authority revision.
Different-key bursts for one file therefore share one active plus one
latest-pending lane. Authority revision remains diagnostic change metadata.
The worker serializes the authority's currently bound layer and does the
filesystem work; callers on the UI or frame path receive a mutation receipt
instead of performing filesystem work.

A persistence request and deferred admission retain their original
`SettingsStore`, physical target and `SettingsFileGeneration`. A failed ticket
may be retried only as that same typed request; retry does not accept a caller
supplied Store. Project changes cancel unstarted tickets and the authority
verifies the target binding before a worker writes, preventing a stale Project
A request from serializing Project B state into A. At host shutdown,
`flush_then_shutdown().finish()` must complete:
it returns success only when its fence succeeds, so a final persistence failure
cannot become a successful editor exit.

Workbench bool toggles, numeric steppers, schema-backed enum selection,
commit-only string fields, typed key-chord selectors, typed RGBA color editors, and
reset-to-effective-source actions use shared
action identities and indexed row hit testing. Enum options come only from
`SettingSchema::Enum`; the host
validates the selected variant again before submitting it, and a changed enum
receipt replaces the selected value batch and closes the popup in one retained
tree refresh. String fields reuse the host text-focus draft; character input
only redraws that draft, while Enter revalidates `SettingSchema::String` and
submits once through the Context mutation coordinator. Chord values are
`EditorKeyChord`, not arbitrary strings. A key selector enters an exclusive
host `chord_capture`: IME is disabled, Escape cancels, modifier-only events wait
for a primary key, and the first valid chord commits once before global keymap
dispatch can observe it. An unchanged boundary step does no batch refresh or
paint request. Enum and color popups share one retained active-editor state
identified by schema kind, setting key, and projected row. Color values cross
the retained bridge as four integer channels; `#RRGGBBAA` is display-only. The
row paints an alpha checkerboard swatch and the popup exposes bounded R/G/B/A
steppers that submit through the same Context mutation coordinator.
This source path is not yet an accepted product contract: the
default registry has no standalone Chord or Color definition, and keymap
conflict/unbind behavior, structured editors, enum keyboard/scroll behavior,
general accessibility behavior, and real-window validation remain open. A queued
`SettingsFileGeneration` is a
scheduling and receipt identity, not yet an immutable durable byte projection or
digest, and full document encoding still needs to move outside authority/project
locks. A UI-local store, retry queue, or direct
`SettingsAuthority::set` path would create a second persistence lifecycle and
is prohibited.

An open Settings Workbench retains only projection revision metadata and the
selected category value batch. Closed windows do not read extension revision
state. For an open window, a settings-generation change refreshes only the
selected category batch; contribution generation, enabled capabilities, or
locale changes rebuild the directory projection. Plugin-template and Settings
consumers keep independent accepted revisions so one consumer's materialization
failure cannot block or falsely acknowledge the other.

## Extension Rules

New structured settings add a typed `SettingValue` and `SettingSchema`; they
must not embed JSON or TOML inside `SettingValue::String`. New consumers use
the authority snapshot or its bounded delta contract and must not introduce a
second settings cache, reader, scheduler, fallback parser, or persistence path.
