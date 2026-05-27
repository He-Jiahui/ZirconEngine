# Sound Editor Live Output Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the Sound Mixer editor surface to neutral live output-device manager data through a plugin-local controller and tested view model.

**Architecture:** `zircon_plugins/sound/editor` owns the editor controller and serialized view models. It consumes only `Arc<dyn SoundManager>` and neutral sound framework DTOs. `zircon_editor` core remains generic; the mixer toolbar gains stable control/event metadata but no native pane branch.

**Tech Stack:** Rust 2021, serde DTOs already available through `zircon_editor` dependency graph, sound framework DTOs, sound editor plugin registration tests, UI v2 TOML metadata.

---

## Source Map

- Create `zircon_plugins/sound/editor/src/live_output/mod.rs`: structural module boundary and re-exports.
- Create `zircon_plugins/sound/editor/src/live_output/model.rs`: editor-facing snapshot/action DTOs.
- Create `zircon_plugins/sound/editor/src/live_output/controller.rs`: `SoundEditorLiveOutputController` over `Arc<dyn SoundManager>`.
- Modify `zircon_plugins/sound/editor/src/lib.rs`: declare and re-export `live_output` only.
- Modify `zircon_plugins/sound/editor/mixer_console.v2.ui.toml`: replace the transport placeholder with refresh/start/stop controls and a picker/status slot.
- Create `docs/zircon_plugins/sound/editor.md`: module detail doc for the editor live-output boundary.
- Update `docs/engine-architecture/runtime-sound-extension.md`: record live-output editor model and validation evidence.
- Update `.codex/sessions/20260523-0748-sound-sequential-milestones.md`: record this active editor live wiring slice and evidence.

## Milestone 1: Plugin-Local Live Output Model

Goal: Add the editor-owned DTOs and controller without touching `zircon_editor` operation dispatch.

Implementation slices:

- [x] Add `SoundEditorOutputDeviceRow` with descriptor, selected/default/available flags, and diagnostic.
- [x] Add `SoundEditorOutputStatusModel` with descriptor, state, backend state, latency, render/callback counters, error, and diagnostics.
- [x] Add `SoundEditorOutputSnapshot` containing device rows, status, backend status, and diagnostics.
- [x] Add `SoundEditorOutputAction` for `Refresh`, `Configure`, `Start`, and `Stop`.
- [x] Add `SoundEditorOutputActionReport` with action, success, error, and best-effort snapshot.
- [x] Add `SoundEditorLiveOutputController` over `Arc<dyn SoundManager>` with `snapshot()` and `apply_action(...)`.
- [x] Keep `lib.rs` structural by only adding module declaration and curated re-exports.

Testing stage:

- [ ] Defer compile/test execution until Milestone 3 per repository milestone-first cadence.

Exit evidence:

- No CPAL type crosses into editor DTOs.
- `zircon_editor` core has no new sound-specific dependency or handler branch.

## Milestone 2: Mixer Toolbar Metadata And Docs

Goal: Make the Sound Mixer UI metadata discoverable for live output actions and document the boundary.

Implementation slices:

- [x] Replace the transport placeholder in `mixer_console.v2.ui.toml` with `SoundOutputDevicePicker`, `SoundOutputRefreshButton`, `SoundOutputStartButton`, `SoundOutputStopButton`, and `SoundOutputStatusPanel` controls.
- [x] Route button events to existing sound output operation paths.
- [x] Keep the picker as a metadata/control slot, not a concrete editor-core pane payload.
- [x] Create `docs/zircon_plugins/sound/editor.md` with required YAML frontmatter.
- [x] Update `docs/engine-architecture/runtime-sound-extension.md` and active session note.

Testing stage:

- [ ] Defer compile/test execution until Milestone 3.

Exit evidence:

- UI control IDs and operation routes are stable for future host binding.
- Docs list implementation files, plan/spec sources, and validation commands.

## Milestone 3: Tests And Acceptance Validation

Goal: Prove the editor live-output boundary and record evidence.

Implementation slices:

- [x] Add fake-manager unit tests for snapshot selection/status projection.
- [x] Add fake-manager unit tests for configure/start/stop action execution.
- [x] Add fake-manager unit tests for best-effort failure reports.
- [x] Update this plan's checkboxes/evidence.

Testing stage:

- [x] Run formatting for the sound editor package.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_editor
```

- [x] Run focused live-output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\editor\Cargo.toml" live_output --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-editor-live-output" --message-format short --color never
```

- [x] Run whitespace check.

```powershell
git diff --check -- "zircon_plugins\sound\editor" "docs\zircon_plugins\sound\editor.md" "docs\engine-architecture\runtime-sound-extension.md" "docs\superpowers\specs\2026-05-23-sound-editor-live-output-design.md" "docs\superpowers\plans\2026-05-23-sound-editor-live-output.md" ".codex\sessions\20260523-0748-sound-sequential-milestones.md"
```

Exit evidence:

- Formatting passes.
- Focused live-output tests pass after refreshing the plugin lockfile offline and retrying from a warmed target.
- Docs and session note record implementation files, validation commands, and next gaps.

Validation results:

- `cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_editor` passed.
- `cargo test --manifest-path "zircon_plugins\sound\editor\Cargo.toml" live_output --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-editor-live-output" --message-format short --color never` first failed because `zircon_plugins/Cargo.lock` needed the new editor `serde` dependency entry.
- `cargo generate-lockfile --manifest-path "zircon_plugins\Cargo.toml" --offline` refreshed the plugin lockfile without network access.
- Retrying the same locked test then stopped in unrelated active `zircon_editor` errors: missing transition helper exports in `node_projection.rs`, moved `transition_kind` in `view_projection.rs`, and a lifetime error in `transition_metadata.rs`.
- `cargo metadata --manifest-path "zircon_plugins\sound\editor\Cargo.toml" --locked --offline --no-deps --format-version 1` passed.
- On 2026-05-24 the same focused live-output command was rerun. The first rerun timed out after 20 minutes during dependency compilation with no Rust diagnostics and no residual matching Cargo/Rust processes; the warmed retry passed: 3 passed, 0 failed, 1 filtered out.
- Scoped `git diff --check` passed with line-ending warnings only.

## Acceptance Criteria

- Sound editor plugin exposes a tested live-output controller and serializable editor model over `SoundManager`.
- Device rows mark the currently configured descriptor as selected.
- Configure/start/stop actions return refreshed best-effort snapshots.
- Backend unavailable errors are represented in action reports and diagnostics without panic.
- Mixer toolbar metadata exposes output picker and output start/stop/refresh control routes.
- No sound-specific branch is added to `zircon_editor` core dispatch or pane payloads.
