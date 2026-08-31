---
related_code:
  - zircon_editor/src/core/project/mod.rs
  - zircon_editor/src/core/project/authority/mod.rs
  - zircon_editor/src/core/project/preflight_manifest_reader.rs
  - zircon_editor/src/core/project/project_preflight/preflight_receipt.rs
  - zircon_editor/src/core/project/created_project.rs
  - zircon_editor/src/core/project/error.rs
  - zircon_editor/src/core/project/filesystem.rs
  - zircon_editor/src/core/project/new_project_draft.rs
  - zircon_editor/src/core/project/new_project_template.rs
  - zircon_editor/src/core/project/opened_project.rs
  - zircon_editor/src/core/project/project_probe.rs
  - zircon_editor/src/core/project/recent_project_entry.rs
  - zircon_editor/src/core/project/recent_project_validation.rs
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/project/scene_load_job.rs
  - zircon_editor/src/core/project/stored_recent_project_entry.rs
  - zircon_editor/src/core/project/stored_startup_session.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/core/document/scene_route.rs
  - zircon_editor/src/core/document/scene_reload.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_scene_document_submission.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench/startup/editor_startup_session_document_welcome_pane_snapshot.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace/active_scene_reload_conflict.rs
  - zircon_runtime_interface/src/project/template_pack/mod.rs
  - zircon_runtime_interface/src/project/template_pack/render.rs
  - templates/projects/renderable-empty/zircon-project.toml
  - templates/projects/renderable-empty/export/desktop_windows.zpreset
  - templates/projects/renderable-empty/assets/shaders/pbr_shader.zmeta
implementation_files:
  - zircon_editor/src/core/project/authority/mod.rs
  - zircon_editor/src/core/project/created_project.rs
  - zircon_editor/src/core/project/error.rs
  - zircon_editor/src/core/project/filesystem.rs
  - zircon_editor/src/core/project/new_project_draft.rs
  - zircon_editor/src/core/project/new_project_template.rs
  - zircon_editor/src/core/project/opened_project.rs
  - zircon_editor/src/core/project/project_probe.rs
  - zircon_editor/src/core/project/recent_project_entry.rs
  - zircon_editor/src/core/project/recent_project_validation.rs
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/project/scene_load_job.rs
  - zircon_editor/src/core/project/stored_recent_project_entry.rs
  - zircon_editor/src/core/project/stored_startup_session.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/core/document/scene_route.rs
  - zircon_editor/src/core/document/scene_reload.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/startup/create_or_open.rs
  - zircon_editor/src/ui/host/startup/recent_projects.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - user: 2026-07-11 Plan10 M1 slice 1.2 ProjectAuthority hard cut
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_editor/src/ui/host/startup/create_or_open.rs::tests::opened_project_is_not_reopened_just_to_update_recents
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_editor/src/core/project/tests/recent_projects.rs
  - zircon_editor/src/core/project/tests/scene_document.rs
  - zircon_editor/src/core/document/scene_route_tests.rs
  - zircon_editor/src/core/document/scene_reload_tests.rs
  - zircon_editor/src/core/project/tests/boundary.rs
  - zircon_editor/src/core/project/tests/mod.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/host/manager/support.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/host/manager/project_generation_projection.rs
  - zircon_editor/src/tests/host/manager/ui_asset_reference_and_promotion.rs
  - zircon_editor/src/tests/host/manager/ui_asset_session_preview.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
doc_type: module-detail
---

# Editor ProjectAuthority

## Purpose and ownership

`zircon_editor::core::project` is the headless-safe Editor authority for project identity, creation, opening, recent-project persistence shapes, and project validation. UI code only projects its results. The former `ui/workbench/startup` declarations and template generator were deleted; there is no compatibility re-export or second template owner.

Before any project-derived runtime, editor, script, or native code is materialized, `ProjectAuthority` performs bounded data-only preflight. A current v3 manifest yields `ProjectIdentity = CanonicalDescriptorIdentity + ProjectGuid + ProjectManifestDigest`; the canonical descriptor is physical-only and never a display path. v1/v2 manifests produce an explicit migration decision and intentionally have no complete project identity or project-derived composition capability.

## Creation transaction

`NewProjectDraft` uses the shared `zircon_runtime_interface::project::validate_project_name` contract, so Editor and Hub accept only one portable filename component and reject path separators, prefixes, reserved Windows names, trailing aliases, control characters, and surrounding whitespace with typed errors. `ProjectAuthority::create_project` validates the location and target shape, renders the shared interface template pack, writes every packaged entry into a unique sibling staging directory, creates the canonical `.zircon` layout through `ProjectPaths`, and loads/saves the current v3 manifest through the typed runtime `ProjectManifest` API. Template rendering mints one non-nil persistent `ProjectGuid` per project before staging; neither first open nor preflight may invent one. The final sibling rename is the commit point.

Existing non-empty targets, files, symlinks, Windows reparse points, invalid manifests, and I/O failures return typed `ProjectAuthorityError` variants. An existing empty target is moved to an empty rollback directory before commit. Commit failure restores it. If commit and restore both fail, `CommitRollbackFailed` retains both I/O sources and the target/backup paths, while the unique backup remains on disk for recovery. Pre-commit failures remove staging. After commit, removal of the now-redundant empty backup is best-effort so a complete target cannot be reported as a failed half-project.

The versioned source truth is `templates/projects/renderable-empty/`; the Editor no longer embeds PBR, cube, scene, material, or export-preset strings. The current shader template is surface schema v2 with `zr_material_surface`, not the retired entry-point schema, and its `.zmeta` uses the canonical `Shader` asset-kind spelling rather than a parser fallback. New projects also receive a manifest-owned `desktop_windows` export profile plus `export/desktop_windows.zpreset`; the preset references that profile and uses the shared `zircon.export-preset` version envelope.

## Open and derived layout

Opening accepts either a project root or `zircon-project.toml`, resolves an absolute canonical root, rejects linked path components, loads the current manifest, and ensures the five regenerable directories below `.zircon`: `cache`, `registry`, `autosave`, `play`, and `thumbnails`. Physical `library/` ownership and the former `library_root()`/`runtime_cache_root()` APIs do not exist.

`ProjectAuthority::open_project` returns an `OpenedProject` that owns a prepared `ProjectManager` generation with its parsed manifest/registry index and summary. Runtime `AssetManager::open_prepared_project` performs the single source inventory/import scan on that instance; Editor asset projection, watcher, workspace document, save and locator consumers then reuse its snapshot instead of passing a root path through layers and reopening it. `ProjectProbe` remains the lightweight manifest-only value for recent-project and explicit probe flows; it is not a second manager or inventory truth.

`ProjectAuthority::probe_project` and `probe_draft` perform canonical path and typed manifest parsing without mutating the derived layout. The retained welcome host schedules creation-target validation and existing-project probing through the Editor Job System only when the draft changes. `welcome_pane_snapshot` copies the cached result and performs no filesystem validation. A newer draft cancels the old ticket; only the current active probe can update the session projection.

`EditorProjectDocument` loads from an explicit `&ProjectManager` and saves to an explicit `(&ProjectManager, AssetUri)` target. `DocumentLifecycleAuthority` owns the active scene identity; a save without one is rejected rather than inferring `ProjectManifest::default_scene`. The startup path explicitly activates the manifest-selected scene once, and every later picker route replaces that same identity. The former path-taking entry points and their `_from_path.rs` files were deleted, including test-only use sites; a fixture must create the project through `ProjectAuthority`, open/scan one manager, and pass that generation onward. This prevents a test helper from preserving a production-forbidden reopen architecture.

## Scene source and document routing

`ProjectAuthority` is also the headless owner of project-scene source identity. It accepts only project-owned `res://` scene URIs ending in `.scene.toml`, rejects linked/reparse path components, opens existing sources through the active `ProjectManager` generation, and creates a new scene through a unique staging source that is published without overwriting an existing target. Direct creation synchronizes that generation's registry; a failed import rolls the source back and reconciles the registry before it returns an error.

`SceneDocumentRoute` owns the separate document transition. A picker carries an opaque project-session ticket to the route, which validates the active project session before resolving a scene. Before it loads or publishes any target, the route asks the host to admit the transition; the current host rejects a dirty Global scene history, so it cannot clear world/history/selection as an implicit discard. After a scene document is built but before the authoring world changes, the route reserves the validated lifecycle identity. For creation, the runtime catalog import and the editor asset-catalog projection refresh must also succeed after staging cleanup before the host installs the authoring world. A successful installation is followed by an infallible lifecycle commit under the same route gate, so a lifecycle error cannot be reported after the old world/history has been replaced. Catalog or installation failure removes the source and reconciles both catalog views without publishing the reservation. The route deliberately owns neither picker presentation nor menu actions, and host installers receive `ProjectSceneDocument`, preserving the selected URI while keeping filesystem paths inside the project authority. Save/Discard/Cancel presentation is owned by the retained close coordinator over this admission port.

Asset-event active-scene reload is a same-document generation refresh rather than a new document
transition. The target is the lifecycle-owned `ActiveSceneDocumentIdentity`; the manifest default
is used only for initial activation and cannot overwrite a secondary active scene.
`ProjectAuthority::submit_scene_open` runs the validated load through an interactive
`EditorJobSystem` ticket. The retained host keeps one ticket, coalesces overlapping requests, and
only moves the completed world into authoring preparation after a Runtime generation precheck.
`SceneDocumentReloadCoordinator` then serializes the exact project/scene/document activation
identity check with open/create/project-close routes. Each committed activation carries a monotonic
revision, preventing an old A load from committing after A-to-B-to-A. The coordinator rejects dirty local history into a typed retained conflict,
and repeats the Runtime generation check through terminal installation. The dedicated same-project
reload preserves project settings and the document binding while resetting history, selection, and
viewport state. Runtime level preparation remains unregistered until gateway replacement; failed
installation rolls its publication back. Admission pressure is retained by identity and Runtime
generation with 64/128/256 ms backoff; the fourth rejection becomes terminal for that generation,
so continuing watcher events cannot create an infinite maintenance loop. The asset accumulator and
admission retry compose the earliest deadline through the same maintenance-wake owner; an empty
asset refresh later in the tick therefore cannot erase a pending retry. A dirty conflict binds the
exact lifecycle identity and Runtime generation to one Decision Center ticket with Save, Discard,
and Keep Editing. Save invokes the same `SaveProject` command used by the workbench; this is the
single authority that persists the active scene and applies a Global-history save token. It does
not send the lifecycle document id through the toolkit/DirtyRegistry batch coordinator. After the
save, the conflict owner verifies the exact active scene identity and clean Global history before
queuing one latest-generation reload. Keep Editing suppresses the same generation, while a new
generation re-enters detection. The separate toolkit coordinator has explicit SaveAll/ClosePrompt
ownership: a non-owner cannot drain completion, Save All retains one queued request, and project or
native-window close cannot begin teardown before the current owner terminalizes.
Discard is explicit and clears history only inside the identity-checked, generation-fenced terminal
installation; failed preparation and superseded work leave local edits intact. Switching document
drops the obsolete conflict. Project close cancels the scene-load ticket only after the close
authority succeeds. Incremental load cancellation and removal of runtime-extension/level
preparation from the UI tick remain open.

Layout preset name projection uses the Runtime asset manager's manager-owned locator query. The query copies only active registry locators under the project read lock; it does not clone the complete `ProjectManager` or its registry/index maps. Explicit preset save/load may access the file through a generation snapshot and then requests a Runtime import refresh; listing never opens a manager, enumerates a directory, or parses every preset document. Runtime's current refresh is still a full import. Its transactional targeted replacement remains an open Runtime04 failure and is not claimed by this Editor projection slice.

## Recent projects

Recent entries persist `ProjectManifestSummary`, path, and timestamp. The summary name is the only display-name truth, while its v3 `project_guid` stays available for later typed identity construction. Validation is projected dynamically and is not persisted. A v1/v2 entry is marked migration-required rather than valid or automatically opened. Remembering the same path replaces its summary, sorts by timestamp, and enforces the eight-entry limit. JSON encode/decode failures retain their `serde_json::Error` sources through `ProjectAuthorityError`.

After a successful interactive open, startup passes the already validated root and manifest summary
directly to `remember_opened_project`. It no longer calls the public independent
`update_recent_project` path, which intentionally opens and validates an arbitrary caller-provided
path. This removes a duplicate canonicalize/read/manifest-parse from the common open-and-remember
flow while preserving the standalone update API.

## Test status

Source tests cover complete template copy with distinct persisted GUIDs, unsafe project-name rejection, data-only preflight, explicit v1/v2 migration handling, `.zircon` layout, non-empty refusal, commit failure restoration, commit-plus-restore failure backup preservation, Summary roundtrip/refresh, typed scene-load job output, and the core-to-UI dependency guard. The renderable-template regression now opens and scans the created project through `ProjectManager`; Editor Manager fixtures use one `ProjectAuthority`-backed helper instead of treating document save as project creation. Earlier Windows lib-test results predate the v3/typed-identity hard cut and are historical only.

The 2026-07-17/18 current slice also excludes the boundary test directory from its own forbidden-token scan without weakening production checks, removes `open_project_manager_for_paths` and `current_project_root`, and adds source guards for welcome/preset projection I/O plus generation-survival document tests. Welcome probing has deterministic Job tests for replacement, late completion, cancellation, job failure and submit failure. The new projection family lives in `tests/host/manager/project_generation_projection.rs`; the touched bootstrap test owner is 719 lines and the new behavior file is 255 lines, both below the test budget. Scoped formatting and diff checks pass. The current v3/typed-identity slice has not run Cargo; independent re-review and coordinator-managed Windows gates remain pending.
