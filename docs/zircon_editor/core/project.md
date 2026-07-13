---
related_code:
  - zircon_editor/src/core/project/mod.rs
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/core/project/created_project.rs
  - zircon_editor/src/core/project/error.rs
  - zircon_editor/src/core/project/filesystem.rs
  - zircon_editor/src/core/project/new_project_draft.rs
  - zircon_editor/src/core/project/new_project_template.rs
  - zircon_editor/src/core/project/opened_project.rs
  - zircon_editor/src/core/project/recent_project_entry.rs
  - zircon_editor/src/core/project/recent_project_validation.rs
  - zircon_editor/src/core/project/stored_recent_project_entry.rs
  - zircon_editor/src/core/project/stored_startup_session.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_runtime_interface/src/project/template_pack/mod.rs
  - zircon_runtime_interface/src/project/template_pack/render.rs
  - templates/projects/renderable-empty/zircon-project.toml
  - templates/projects/renderable-empty/export/desktop_windows.zpreset
  - templates/projects/renderable-empty/assets/shaders/pbr_shader.zmeta
implementation_files:
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/core/project/created_project.rs
  - zircon_editor/src/core/project/error.rs
  - zircon_editor/src/core/project/filesystem.rs
  - zircon_editor/src/core/project/new_project_draft.rs
  - zircon_editor/src/core/project/new_project_template.rs
  - zircon_editor/src/core/project/opened_project.rs
  - zircon_editor/src/core/project/recent_project_entry.rs
  - zircon_editor/src/core/project/recent_project_validation.rs
  - zircon_editor/src/core/project/stored_recent_project_entry.rs
  - zircon_editor/src/core/project/stored_startup_session.rs
plan_sources:
  - user: 2026-07-11 Plan10 M1 slice 1.2 ProjectAuthority hard cut
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_editor/src/core/project/tests/recent_projects.rs
  - zircon_editor/src/core/project/tests/boundary.rs
  - zircon_editor/src/core/project/tests/mod.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/host/manager/support.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion.rs
  - zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
doc_type: module-detail
---

# Editor ProjectAuthority

## Purpose and ownership

`zircon_editor::core::project` is the headless-safe Editor authority for project identity, creation, opening, recent-project persistence shapes, and project validation. UI code only projects its results. The former `ui/workbench/startup` declarations and template generator were deleted; there is no compatibility re-export or second template owner.

## Creation transaction

`NewProjectDraft` uses the shared `zircon_runtime_interface::project::validate_project_name` contract, so Editor and Hub accept only one portable filename component and reject path separators, prefixes, reserved Windows names, trailing aliases, control characters, and surrounding whitespace with typed errors. `ProjectAuthority::create_project` validates the location and target shape, renders the shared interface template pack, writes every packaged entry into a unique sibling staging directory, creates the canonical `.zircon` layout through `ProjectPaths`, and loads/saves the manifest through the current typed runtime `ProjectManifest` API. The final sibling rename is the commit point.

Existing non-empty targets, files, symlinks, Windows reparse points, invalid manifests, and I/O failures return typed `ProjectAuthorityError` variants. An existing empty target is moved to an empty rollback directory before commit. Commit failure restores it. If commit and restore both fail, `CommitRollbackFailed` retains both I/O sources and the target/backup paths, while the unique backup remains on disk for recovery. Pre-commit failures remove staging. After commit, removal of the now-redundant empty backup is best-effort so a complete target cannot be reported as a failed half-project.

The versioned source truth is `templates/projects/renderable-empty/`; the Editor no longer embeds PBR, cube, scene, material, or export-preset strings. The current shader template is surface schema v2 with `zr_material_surface`, not the retired entry-point schema, and its `.zmeta` uses the canonical `Shader` asset-kind spelling rather than a parser fallback. New projects also receive a manifest-owned `desktop_windows` export profile plus `export/desktop_windows.zpreset`; the preset references that profile and uses the shared `zircon.export-preset` version envelope.

## Open and derived layout

Opening accepts either a project root or `zircon-project.toml`, resolves an absolute canonical root, rejects linked path components, loads the current manifest, and ensures the five regenerable directories below `.zircon`: `cache`, `registry`, `autosave`, `play`, and `thumbnails`. Physical `library/` ownership and the former `library_root()`/`runtime_cache_root()` APIs do not exist.

`ProjectAuthority::probe_project` and `probe_draft` perform the same canonical path and typed manifest parse without mutating the derived layout. Welcome actions and snapshots use this authority probe; the draft DTO no longer owns a weaker existence-only open check.

## Recent projects

Recent entries persist `ProjectManifestSummary`, path, and timestamp. The summary name is the only display-name truth. Validation is projected dynamically and is not persisted. Remembering the same path replaces its summary, sorts by timestamp, and enforces the eight-entry limit. JSON encode/decode failures retain their `serde_json::Error` sources through `ProjectAuthorityError`.

## Test status

Source tests cover complete template copy, v2 name rewrite, unsafe project-name rejection, actual reopen and manifest probe, `.zircon` layout, non-empty refusal, commit failure restoration, commit-plus-restore failure backup preservation, Summary roundtrip/refresh, and the core-to-UI dependency guard. The renderable-template regression now opens and scans the created project through `ProjectManager`; Editor Manager fixtures use one `ProjectAuthority`-backed helper instead of treating document save as project creation. On 2026-07-13, a current-source Windows lib-test binary ran `tests::host::manager:: --nocapture --test-threads=1` with 83 passed and 0 failed; the focused renderable-template test passed 1/1.
