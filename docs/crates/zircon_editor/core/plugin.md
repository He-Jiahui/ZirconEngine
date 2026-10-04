---
related_code:
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/manager/project_registration.rs
  - zircon_editor/src/core/plugin/manager/project_selection.rs
  - zircon_editor/src/core/plugin/catalog_snapshot.rs
  - zircon_editor/src/core/plugin/catalog_store.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/native_backend.rs
  - zircon_editor/src/core/plugin/phases.rs
  - zircon_editor/src/core/plugin/sdk/lifecycle.rs
implementation_files:
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/manager/project_registration.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/core/plugin/manager/project_selection.rs
  - zircon_editor/src/core/plugin/manager/state.rs
  - zircon_editor/src/core/plugin/manager/lifecycle_replacement.rs
  - zircon_editor/src/core/plugin/catalog_snapshot.rs
  - zircon_editor/src/core/plugin/catalog_store.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/native_backend.rs
plan_sources:
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/12/failure-2026-07-17-editor-plugin-catalog-rebuild-and-deep-copy.md
tests:
  - tools/tests/test_editor12_plugin_manager_contract.py
  - tools/tests/test_editor12_plugin_catalog_store_contract.py
  - zircon_editor/src/core/plugin/manager/tests.rs
doc_type: module-detail
---

# Editor Plugin Manager

## Contract

`core::plugin::EditorPluginManager` is the single owner of the editor-plugin
catalog generation, lifecycle state, and active extension view. UI panes,
commandlets, and retained-host code consume the immutable manager snapshot;
they do not rebuild builtin descriptors, maintain a private catalog, load a
native library, or invoke a native entry callback while serving a stable read.

Catalog mutations prepare a candidate catalog and entries before one manager
snapshot publication. A published snapshot remains readable by existing `Arc`
holders, while a later publication receives a distinct generation. Package and
capability lookup remain ordered snapshot queries rather than caller-owned
collection copies. Conflicting runtime manifests for the same editor package
are rejected before candidate publication; the catalog never selects one input
silently.

The mutable catalog is crate-internal. External producers construct candidates
only through `EditorPluginManager::from_plugins` or
`EditorPluginManager::from_descriptors`; external consumers receive immutable
catalog snapshots and projections only. Initial catalog generation is always
one, regardless of how many plugin registrations the candidate contains.

## Project Selection

`EditorPluginManager::apply_project_manifest` consumes the completed
`ProjectPluginManifest` as desired enablement input for already-discovered
editor packages. Runtime-only selections remain runtime-owned; a missing editor
selection means disabled for that manager generation. The manager rejects a
duplicate selection for a known editor package and preflights every requested
transition before dispatching lifecycle callbacks. A valid batch publishes one
new snapshot at most, and an unchanged batch returns the existing `Arc`.

The editor host completes a project's manifest, materializes its selected
native editor registrations, and replaces only the manager's `Project` rows
before it applies enablement or publishes document activation messages. Close
and failed-open rollback clear those project rows; the close lifecycle message
is still published after a committed host close even if catalog cleanup reports
an error. This prevents a project-specific native registration from surviving
into another project. The same explicit project-open operation publishes one
`ProjectPluginStatusSnapshot` from its `NativePluginLoadReport` after the
manager generation is applied. Retained plugin panes clone that snapshot's
`Arc` and never reparse `zircon-project.toml` or rediscover native plugins.
Close and failed-open rollback clear the host-owned snapshot before a later
project can observe it. The retained host may cache a derived pane view only
by `Arc` identity; it has no mutable plugin facts and rebuilds when the host
publishes a different snapshot. Plugin panels do not maintain a private status
cache.

Native-aware plugin enablement, feature dependency changes, feature toggles,
packaging changes, and target-mode changes reuse their operation's native load
report to publish a successor project status only after the mutation succeeds.
An error leaves the last-good status `Arc` in place. Those publications are the
only invalidation signal the retained pane cache needs.

## Lifecycle Ownership

Loading and enablement own `Loaded`, `Enabled`, and `Disabled`; callers cannot
send those stages through the external dispatch APIs. Source owners for other
lifecycle stages must call `dispatch_lifecycle_event_to_active` only after
their own contracts are available. The manager captures the current `Active`
set once, invokes every member of that set, marks only a failed callback as
`Faulted`, and publishes the resulting catalog/state exactly once. A callback
failure therefore cannot prevent another already-active plugin from observing
the event.

Replacing an active plugin instance is also manager-owned: the old instance
receives `Disabled` then `Unloaded`, the candidate instance receives `Loaded`
then `Enabled`, and only a successfully active candidate receives
`HotReloaded`. Cleanup failure rejects the candidate and publishes only old
instances: already-cleaned entries remain `Revoking`, while the callback that
failed becomes `Faulted`. The manager never exposes a mixed old/new generation.
A later replacement retries only cleanup stages that have not already
succeeded, so an `Unloaded` retry cannot duplicate a completed `Disabled`
callback or bypass the old instance; after the full old batch retires, every
candidate is reset and activated together.

External event sources remain outside this module: play-mode transitions belong
to the play-mode owner, scene changes to the document-message owner, and asset
changes to the asset-index owner. The retained host must not synthesize these
events from rendering refreshes or filesystem reads.

## Native Development Reload

Debug native-plugin artifact watches are change producers, not schedulers. A
watch retains one latest-change timestamp and one optional `EditorJobSystem`
ticket; it has no private thread, channel, executor, or shutdown join. The
filesystem callback filters the exact loaded artifact, replaces the timestamp,
and signals the Host's coalescing background wake callback.

Retained Host tick merges the watch's 350 ms debounce deadline with the active
project-session heartbeat deadline. Once due, it submits one background Compile
job through the context-owned `EditorJobSystem`. Native reload jobs share the
`native_plugin_reload` mutex group, so different watches cannot run native host
mutation concurrently. Changes received while a ticket is pending or running
remain one timestamp and can schedule at most one successor job.

Removing a watch first unregisters the filesystem callback, then cancels the
ticket token and the pending job id without waiting. The job uses a weak native
host handle and checks cancellation before entering the host operation. Job
admission, completion, failure, progress, and editor shutdown therefore remain
owned by the same scheduler as other editor background work.

## Constraints

The module is a hard-cut owner. New consumers import `core::plugin` directly;
there is no legacy root-module documentation or compatibility facade. Native
dynamic loading and entry callbacks are explicit project-open/refresh work,
never a side effect of a status or pane read. Project opening obtains one native
load report and uses it for manifest completion, selected registration, and the
published status projection. Plugins01 still owns a non-scheduling published
native-discovery snapshot for refresh orchestration; Editor12 does not recreate
that authority in the UI. Plugin V2 document/action materialization is a
retained-host concern that consumes manager-owned generation data without
becoming a second catalog authority.

## Validation

The static manager and catalog-store contracts protect the ownership and
hard-cut boundaries. M1 source-bound Cargo validation remains the coordinator-
managed testing stage named by the owning Editor12 plan.
