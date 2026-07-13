---
related_code:
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_runtime_interface/src/project/template_pack/mod.rs
  - zircon_runtime_interface/src/project/template_pack/render.rs
  - templates/projects/renderable-empty/zircon-project.toml
implementation_files:
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
plan_sources:
  - user: 2026-07-11 Plan10 M1 slice 1.2 Hub template and Summary hard cut
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
tests:
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/editor_recent_sync.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs
  - zircon_runtime_interface/src/project/tests/template_pack.rs
doc_type: module-detail
---

# Hub project creation and recent identity

## Shared template packaging

Hub creation consumes `render_project_template` from `zircon_runtime_interface`; it has no copied Rust template strings and no runtime dependency on a source checkout. The interface binary embeds the version-controlled repository template at build time and exposes neutral rendered entries. A source-tree audit test requires every versioned template file to appear in the pack and rejects links or reparse points.

Hub validates project names through the same portable single-component interface contract as Editor, then writes entries into a sibling staging directory and commits with rename. It rejects non-empty targets and linked ancestors, preserves an existing empty target for rollback, and removes staging on every pre-commit error. If commit and restore both fail, typed `CommitRollbackFailed` retains both I/O sources and leaves the backup intact; request validation is propagated through `CreateProjectRequestError` rather than collapsed into a string. A successful result includes the shared manifest Summary used by downstream lifecycle work.

## Recent project truth

`RecentProject` persists `summary: ProjectManifestSummary`, `path`, and `last_opened_unix_ms`; the removed `display_name` field is not accepted as a second identity. Project creation, import, editor-session sync, and Hub config load parse or refresh the summary from `zircon-project.toml` when the project exists. Merge deduplicates by normalized filesystem key, keeps the newest entry, and uses Summary name only as a deterministic tie-breaker.

The Editor session wire shape uses the same Summary field. Old display-name-only session records are intentionally not a compatibility input.

## Test status

Unit-test source covers template copy, current manifest equality, shared unsafe-name rejection, `.zircon` layout, non-empty rollback, commit failure restoration, commit-plus-restore failure backup preservation, Summary persistence, refresh, merge, and config roundtrip. Cargo is deferred to the Plan10 M1 testing stage; this slice records only rustfmt and static evidence.
