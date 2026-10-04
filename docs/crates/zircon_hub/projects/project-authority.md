---
related_code:
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime_interface/src/project/template_pack/mod.rs
  - zircon_runtime_interface/src/project/template_pack/render.rs
  - templates/projects/renderable-empty/zircon-project.toml
implementation_files:
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
plan_sources:
  - user: 2026-07-11 Plan10 M1 slice 1.2 Hub template and Summary hard cut
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
tests:
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - zircon_runtime_interface/src/project/tests/template_pack.rs
doc_type: module-detail
---

# Hub project launch and recent identity

## Editor-owned project creation

Hub does not write template files or own a second project-creation transaction. It validates the portable single-component project name and request shape, resolves only the prospective target path, constructs the typed Editor launch command, and schedules the launch through the existing background action worker. The removed `zircon_hub::projects::create_project` implementation has no compatibility wrapper or fallback.

The launched Editor is the sole filesystem and project-session authority. It acquires the target creation lease, publishes provisionally, activates the project under the session guard, and reports Hub Ready only after the retained host's first presented frame. Hub retains the supervised `Child` and waits on the background thread until either that target-qualified terminal handshake arrives or the child actually exits; an arbitrary cold-start duration cannot become a false terminal failure. Each background action owns a task-ID-bound cancellation token. A cancellation request for the current ID terminates and reaps a still-starting Editor, while a stale ID cannot affect the next queued action; an already committed terminal mailbox wins the observation race. After Ready, the `Child` moves to the Hub's single condition-variable-backed reaper instead of being dropped or receiving a per-process watcher thread. Only Ready reparses the manifest Summary, registers recents/history, and selects the project. Failed launch, cancellation, activation, or real child exit does not publish a Hub success record. Direct synchronous dispatch of `HubAction::CreateProject` is rejected so callers cannot bypass the background lifecycle.

## Recent project truth

`RecentProject` persists `summary: ProjectManifestSummary`, `path`, and `last_opened_unix_ms`; the removed `display_name` field is not accepted as a second identity. Project creation, import, editor-session sync, and Hub config load parse or refresh the summary from `zircon-project.toml` when the project exists. Merge deduplicates by normalized filesystem key, keeps the newest entry, and uses Summary name only as a deterministic tie-breaker.

The Editor session wire shape uses the same Summary field. Old display-name-only session records are intentionally not a compatibility input.

## Test status

Unit-test source covers data-only launch preparation, zero Hub target writes before launch, Ready-gated registration, shared unsafe-name rejection, Summary persistence, refresh, merge, and config roundtrip. Editor project tests own template copy, creation lease, publication, activation rollback, and empty-target restoration. Cargo remains subject to the coordinator-managed validation lane; source tests are contracts rather than accepted runtime evidence.
