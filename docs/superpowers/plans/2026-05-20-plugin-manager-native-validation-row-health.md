# Plugin Manager Native Validation Row Health Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show NativeDynamic validation health as a compact per-row Plugin Manager metadata label without changing ABI, runtime behavior, or Plugin Manager actions.

**Architecture:** Add a runtime-owned aggregate health helper on `NativePluginLoadReport`, then let `zircon_editor` project that helper into existing Plugin Manager status/view/payload data. The row renderer inserts the label into the existing metadata line only when a loaded native behavior validation report exists.

**Tech Stack:** Rust, Cargo, `zircon_runtime::plugin::native_plugin_loader`, `zircon_editor` retained host pane projection, Markdown docs.

---

## Inputs

- Approved spec: `docs/superpowers/specs/2026-05-20-plugin-manager-native-validation-row-health-design.md`
- Runtime loader docs: `docs/zircon_runtime/plugin/native_plugin_loader/index.md`
- Plugin Manager docs: `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md`
- Current NativeDynamic validation owner: `zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs`
- Current editor native status owner: `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs`

## Current Baseline

- `NativePluginBehaviorValidationReport` already exposes `health: NativePluginBehaviorHealth`.
- `LoadedNativePlugin` already exposes `runtime_behavior_health()` and `editor_behavior_health()`.
- `NativePluginLoadReport` already aggregates diagnostics, but it does not expose a per-plugin aggregate health helper.
- `EditorPluginStatus` does not carry native validation row text.
- `ModulePluginStatusViewData`, `ModulePluginStatusPayload`, and retained-host `ModulePluginStatusData` do not carry native validation row text.
- `pane_data_conversion/module_plugins.rs` builds row metadata directly as `package_source | load_state | packaging | target_modes`.
- `host_lifecycle.rs` is oversized, so this plan only permits a mechanical field copy there. New logic belongs in smaller owner modules.

## File Structure

Modify these files:

- `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs`: add `behavior_health_for_plugin(...)` and unit tests for aggregate worst-health behavior.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/status/mod.rs`: register a new helper module.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_validation.rs`: create a focused helper that converts runtime aggregate health into `Native: Clean`, `Native: Degraded`, or `Native: Invalid`.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs`: attach the helper output to native plugin status rows.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs`: set builtin-only rows to an empty native validation label.
- `zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs`: add the editor status field.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`: add the view-data field.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs`: add the payload field.
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs`: copy the field into payloads.
- `zircon_editor/src/ui/retained_host/host_contract/data/panes.rs`: add the retained-host contract field.
- `zircon_editor/src/ui/retained_host/app/host_lifecycle.rs`: mechanically copy `EditorPluginStatus.native_validation` into `ModulePluginStatusViewData.native_validation`.
- `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/module_plugins.rs`: copy the field and insert it into row metadata only when non-empty.
- `zircon_editor/src/tests/host/manager/minimal_host_contract.rs`: assert missing-library native rows do not fabricate validation health.
- `zircon_editor/src/tests/host/pane_presentation.rs`: assert payload projection preserves the field.
- `docs/zircon_runtime/plugin/native_plugin_loader/index.md`: document the aggregate health report helper and scoped evidence.
- `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md`: document Plugin Manager row health projection and scoped evidence.

Do not modify ABI declarations, plugin callback signatures, action IDs, `zircon_hub/**`, broad UI Material files, profiling files, ECS files, ZrVM files, render files, or asset pipeline files.

## Milestone 1: Runtime Aggregate Health Helper

### Goal

Expose a runtime-owned `NativePluginLoadReport` helper that computes the worst validation health for one plugin id across loaded runtime and editor behavior reports.

### In-Scope Behaviors

- Return `None` when the plugin has no loaded validation reports.
- Return `Some(Clean)` when all available reports are clean.
- Return `Some(Degraded)` when at least one available report is degraded and none are invalid.
- Return `Some(Invalid)` when any available report is invalid.
- Do not inspect diagnostics text to infer health.
- Do not change existing diagnostic aggregation behavior.

### Dependencies

- `NativePluginBehaviorHealth` exists and is re-exported from `zircon_runtime::plugin`.
- `LoadedNativePlugin::runtime_behavior_health()` and `LoadedNativePlugin::editor_behavior_health()` already exist.

### Implementation Slices

- [ ] **Slice 1.1: Add aggregate helper to `NativePluginLoadReport`**

  In `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs`, extend the imports:

  ```rust
  use super::{LoadedNativePlugin, NativePluginBehaviorHealth, NativePluginCandidate};
  ```

  Add this public method inside `impl NativePluginLoadReport` after `descriptor_diagnostics(...)`:

  ```rust
      pub fn behavior_health_for_plugin(
          &self,
          plugin_id: &str,
      ) -> Option<NativePluginBehaviorHealth> {
          let mut aggregate = None;
          for plugin in self.loaded.iter().filter(|plugin| plugin.plugin_id == plugin_id) {
              for health in [
                  plugin.runtime_behavior_health(),
                  plugin.editor_behavior_health(),
              ]
              .into_iter()
              .flatten()
              {
                  aggregate = Some(worst_behavior_health(aggregate, health));
              }
          }
          aggregate
      }
  ```

  Add this private helper below `sorted_deduped(...)`:

  ```rust
  fn worst_behavior_health(
      current: Option<NativePluginBehaviorHealth>,
      candidate: NativePluginBehaviorHealth,
  ) -> NativePluginBehaviorHealth {
      match (current, candidate) {
          (Some(NativePluginBehaviorHealth::Invalid), _) | (_, NativePluginBehaviorHealth::Invalid) => {
              NativePluginBehaviorHealth::Invalid
          }
          (Some(NativePluginBehaviorHealth::Degraded), _)
          | (_, NativePluginBehaviorHealth::Degraded) => NativePluginBehaviorHealth::Degraded,
          (Some(NativePluginBehaviorHealth::Clean), NativePluginBehaviorHealth::Clean)
          | (None, NativePluginBehaviorHealth::Clean) => NativePluginBehaviorHealth::Clean,
      }
  }
  ```

- [ ] **Slice 1.2: Add focused runtime tests for aggregation**

  Add a `#[cfg(test)] mod tests` block to `native_plugin_load_report.rs` if one does not already exist. Include this test support and tests:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::plugin::native_plugin_loader::{
          NativePluginDescriptor, NativePluginEntryReport, NativePluginBehaviorValidationReport,
          ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
      };

      #[test]
      fn native_plugin_load_report_aggregates_behavior_health_by_worst_severity() {
          let report = NativePluginLoadReport {
              discovered: Vec::new(),
              diagnostics: Vec::new(),
              loaded: vec![
                  loaded_plugin_with_health(
                      "clean_plugin",
                      Some(NativePluginBehaviorHealth::Clean),
                      Some(NativePluginBehaviorHealth::Clean),
                  ),
                  loaded_plugin_with_health(
                      "degraded_plugin",
                      Some(NativePluginBehaviorHealth::Clean),
                      Some(NativePluginBehaviorHealth::Degraded),
                  ),
                  loaded_plugin_with_health(
                      "invalid_plugin",
                      Some(NativePluginBehaviorHealth::Invalid),
                      Some(NativePluginBehaviorHealth::Degraded),
                  ),
              ],
          };

          assert_eq!(
              report.behavior_health_for_plugin("clean_plugin"),
              Some(NativePluginBehaviorHealth::Clean)
          );
          assert_eq!(
              report.behavior_health_for_plugin("degraded_plugin"),
              Some(NativePluginBehaviorHealth::Degraded)
          );
          assert_eq!(
              report.behavior_health_for_plugin("invalid_plugin"),
              Some(NativePluginBehaviorHealth::Invalid)
          );
          assert_eq!(report.behavior_health_for_plugin("missing_plugin"), None);
      }

      fn loaded_plugin_with_health(
          plugin_id: &str,
          runtime_health: Option<NativePluginBehaviorHealth>,
          editor_health: Option<NativePluginBehaviorHealth>,
      ) -> LoadedNativePlugin {
          LoadedNativePlugin {
              plugin_id: plugin_id.to_string(),
              library_path: std::path::PathBuf::from(format!("{plugin_id}.test.dll")),
              descriptor: Some(NativePluginDescriptor {
                  abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
                  plugin_id: plugin_id.to_string(),
                  package_manifest: None,
                  runtime_entry_name: None,
                  editor_entry_name: None,
                  requested_capabilities: Vec::new(),
              }),
              runtime_entry_report: runtime_health.map(|health| {
                  entry_report(plugin_id, crate::plugin::PluginModuleKind::Runtime, health)
              }),
              editor_entry_report: editor_health.map(|health| {
                  entry_report(plugin_id, crate::plugin::PluginModuleKind::Editor, health)
              }),
              library: this_process_library(),
          }
      }

      fn entry_report(
          plugin_id: &str,
          module_kind: crate::plugin::PluginModuleKind,
          health: NativePluginBehaviorHealth,
      ) -> NativePluginEntryReport {
          NativePluginEntryReport {
              plugin_id: plugin_id.to_string(),
              module_kind,
              package_manifest: None,
              diagnostics: Vec::new(),
              negotiated_capabilities: Vec::new(),
              behavior: None,
              behavior_validation: validation_report(plugin_id, module_kind, health),
          }
      }

      fn validation_report(
          plugin_id: &str,
          module_kind: crate::plugin::PluginModuleKind,
          health: NativePluginBehaviorHealth,
      ) -> NativePluginBehaviorValidationReport {
          NativePluginBehaviorValidationReport {
              abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
              module_kind,
              plugin_id: plugin_id.to_string(),
              is_stateless: Some(true),
              state_schema_version: Some(0),
              command_manifest_schema: None,
              event_manifest_schema: None,
              has_command_manifest: false,
              has_event_manifest: false,
              has_invoke_command: true,
              has_save_state: false,
              has_restore_state: false,
              has_unload: true,
              diagnostics: if health == NativePluginBehaviorHealth::Clean {
                  Vec::new()
              } else {
                  vec![format!("{plugin_id} validation {health:?}")]
              },
              health,
          }
      }

      fn this_process_library() -> libloading::Library {
          #[cfg(unix)]
          {
              libloading::os::unix::Library::this().into()
          }
          #[cfg(windows)]
          {
              libloading::os::windows::Library::this()
                  .expect("current process library handle should be available")
                  .into()
          }
      }
  }
  ```

  If visibility of `LoadedNativePlugin.library` blocks this sibling-module fixture, move the aggregation tests to `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs` and keep the same assertions against `NativePluginLoadReport::behavior_health_for_plugin(...)`.

### Testing Stage

- [ ] Run runtime focused tests:

  ```powershell
  cargo test -p zircon_runtime --lib native_plugin_load_report --locked --jobs 1
  ```

  Expected result: the new aggregation test passes and existing `zircon_runtime` warnings, if any, are recorded without being treated as this milestone's failure unless they are new errors.

- [ ] If the focused filter misses the new test because of module naming, run the exact test name:

  ```powershell
  cargo test -p zircon_runtime --lib native_plugin_load_report_aggregates_behavior_health_by_worst_severity --locked --jobs 1
  ```

  Expected result: one test passes.

- [ ] If either command fails, fix the lowest runtime report/helper layer first, then rerun the same command before moving to Milestone 2.

### Lightweight Checks

- [ ] When the code compiles poorly enough to block editing feedback, use:

  ```powershell
  cargo check -p zircon_runtime --lib --locked --jobs 1
  ```

### Exit Evidence

- Runtime aggregate helper exists.
- Runtime tests prove clean, degraded, invalid, and absent behavior.
- Existing diagnostics methods are unchanged.

## Milestone 2: Editor Status And Row Projection

### Goal

Carry the runtime aggregate health helper through editor plugin status, pane payload, retained-host contract data, and row metadata rendering.

### In-Scope Behaviors

- Native rows with validation reports show `Native: Clean`, `Native: Degraded`, or `Native: Invalid`.
- Rows without validation reports use an empty label and keep the old metadata shape.
- `load_state`, actions, packaging, target modes, capabilities, and diagnostics remain unchanged.
- The only edit in oversized `host_lifecycle.rs` is a mechanical field copy.

### Dependencies

- Milestone 1 helper is available as `NativePluginLoadReport::behavior_health_for_plugin(...)`.

### Implementation Slices

- [ ] **Slice 2.1: Add editor status field**

  In `zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs`, add this field after `load_state`:

  ```rust
      pub native_validation: String,
  ```

  In `zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs`, set builtin rows to empty:

  ```rust
                      native_validation: String::new(),
  ```

- [ ] **Slice 2.2: Add native validation label helper**

  In `zircon_editor/src/ui/host/editor_manager_plugins_export/status/mod.rs`, add:

  ```rust
  mod native_validation;
  ```

  Create `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_validation.rs`:

  ```rust
  use zircon_runtime::plugin::{NativePluginBehaviorHealth, NativePluginLoadReport};

  pub(super) fn native_validation_label(
      report: &NativePluginLoadReport,
      plugin_id: &str,
  ) -> String {
      match report.behavior_health_for_plugin(plugin_id) {
          Some(NativePluginBehaviorHealth::Clean) => "Native: Clean".to_string(),
          Some(NativePluginBehaviorHealth::Degraded) => "Native: Degraded".to_string(),
          Some(NativePluginBehaviorHealth::Invalid) => "Native: Invalid".to_string(),
          None => String::new(),
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn native_validation_label_omits_absent_health() {
          let report = NativePluginLoadReport::default();

          assert_eq!(native_validation_label(&report, "missing"), "");
      }
  }
  ```

- [ ] **Slice 2.3: Attach label in native status projection**

  In `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs`, add:

  ```rust
  use super::native_validation::native_validation_label;
  ```

  Inside the `for package in native_packages` loop, after `let load_state = ...`, compute:

  ```rust
              let native_validation = native_validation_label(&native_report, &package.id);
  ```

  Pass `native_validation` into `native_plugin_status(...)`:

  ```rust
                  native_validation,
  ```

  When merging into an existing builtin row, assign the label after `existing.load_state = load_state;`:

  ```rust
              existing.native_validation = native_validation;
  ```

  Update the helper signature:

  ```rust
  fn native_plugin_status(
      package: &PluginPackageManifest,
      manifest: &ProjectManifest,
      mut diagnostics: Vec<String>,
      load_state: String,
      native_validation: String,
      optional_features: Vec<EditorPluginFeatureStatus>,
  ) -> EditorPluginStatus {
  ```

  Set the field in the returned status:

  ```rust
          native_validation,
  ```

- [ ] **Slice 2.4: Add view, payload, and retained-host fields**

  In `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`, add after `load_state`:

  ```rust
      pub native_validation: SharedString,
  ```

  In `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs`, add after `load_state`:

  ```rust
      pub native_validation: String,
  ```

  In `zircon_editor/src/ui/retained_host/host_contract/data/panes.rs`, add after `load_state`:

  ```rust
      pub native_validation: SharedString,
  ```

  In `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs`, copy the field:

  ```rust
              native_validation: plugin.native_validation.to_string(),
  ```

  In `zircon_editor/src/ui/retained_host/app/host_lifecycle.rs`, copy the field in `ModulePluginStatusViewData` construction:

  ```rust
                              native_validation: plugin.native_validation.into(),
  ```

  In `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/module_plugins.rs`, copy the field in `to_host_contract_module_plugin_status(...)`:

  ```rust
          native_validation: data.native_validation,
  ```

- [ ] **Slice 2.5: Insert label into row metadata when non-empty**

  In `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/module_plugins.rs`, replace the direct metadata `format!(...)` body with this helper call:

  ```rust
              module_plugin_meta_text(&plugin),
  ```

  Add this helper near `module_plugin_row_actions(...)`:

  ```rust
  fn module_plugin_meta_text(plugin: &ModulePluginStatusViewData) -> String {
      let mut parts = vec![
          plugin.package_source.to_string(),
          plugin.load_state.to_string(),
      ];
      if !plugin.native_validation.is_empty() {
          parts.push(plugin.native_validation.to_string());
      }
      parts.push(plugin.packaging.to_string());
      parts.push(plugin.target_modes.to_string());
      parts.join(" | ")
  }
  ```

- [ ] **Slice 2.6: Update editor tests and fixtures**

  In every `ModulePluginStatusViewData` fixture listed by `rg "ModulePluginStatusViewData \{" zircon_editor/src`, add:

  ```rust
              native_validation: "".into(),
  ```

  In `zircon_editor/src/tests/host/pane_presentation.rs`, set the fixture value to prove payload propagation:

  ```rust
              native_validation: "Native: Clean".into(),
  ```

  Add this assertion in the `ModulePluginsV1` payload match:

  ```rust
                  assert_eq!(payload.plugins[0].native_validation, "Native: Clean");
  ```

  In `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/module_plugins.rs`, update `module_plugin_status_fixture()` with:

  ```rust
              native_validation: "".into(),
  ```

  Add this test next to the existing module plugin row projection tests:

  ```rust
      #[test]
      fn module_plugins_row_metadata_includes_native_validation_when_present() {
          let mut pane = module_plugins_pane_fixture();
          let mut plugin = module_plugin_status_fixture();
          plugin.package_source = "native".into();
          plugin.load_state = "loaded with diagnostics".into();
          plugin.native_validation = "Native: Degraded".into();
          plugin.packaging = "NativeDynamic".into();
          plugin.target_modes = "EditorHost".into();
          pane.native_body.module_plugins.plugins = model_rc(vec![plugin]);

          let data = to_host_contract_module_plugins_pane_from_host_pane(
              &pane,
              PaneContentSize::new(480.0, 260.0),
          );

          let meta = (0..data.nodes.row_count())
              .filter_map(|row| data.nodes.row_data(row))
              .find(|node| node.control_id.as_str() == "ModulePluginMeta.physics")
              .expect("module plugin metadata should be projected");
          assert_eq!(
              meta.text.to_string(),
              "native | loaded with diagnostics | Native: Degraded | NativeDynamic | EditorHost"
          );
      }
  ```

  In `zircon_editor/src/tests/host/manager/minimal_host_contract.rs`, after the existing missing-library native status assertions, add:

  ```rust
      assert_eq!(native.native_validation, "");
  ```

  After the later `native_status` lookup near the enabled status assertions, add:

  ```rust
      assert_eq!(native_status.native_validation, "");
  ```

### Testing Stage

- [ ] Run editor native status tests:

  ```powershell
  cargo test -p zircon_editor --lib native_plugin_status_report --locked --jobs 1
  ```

  Expected result: tests covering native status report behavior pass; missing-library native rows keep an empty validation label.

- [ ] Run Plugin Manager payload/projection tests:

  ```powershell
  cargo test -p zircon_editor --lib module_plugins --locked --jobs 1
  ```

  Expected result: Plugin Manager payload and retained-host row projection tests pass.

- [ ] If a projection test fails after adding the field, fix the lowest missing copy first in this order: `EditorPluginStatus`, `ModulePluginStatusViewData`, `ModulePluginStatusPayload`, retained-host `ModulePluginStatusData`, row metadata helper.

### Lightweight Checks

- [ ] When needed before the testing stage, use:

  ```powershell
  cargo check -p zircon_editor --lib --locked --jobs 1
  ```

### Exit Evidence

- Editor status DTO carries the native validation label.
- Plugin Manager payload carries the label.
- Retained row metadata includes the label only when non-empty.
- Existing action IDs and `load_state` values remain unchanged.

## Milestone 3: Docs, Formatting, And Scoped Acceptance

### Goal

Update module documentation and run scoped validation for the runtime/editor row-health slice.

### In-Scope Behaviors

- Document the runtime aggregate health helper.
- Document Plugin Manager row metadata projection.
- Record exact scoped validation evidence.
- Do not claim full workspace green if unrelated active-session changes block workspace-wide commands.

### Dependencies

- Milestone 1 and Milestone 2 implementation and tests are complete.

### Implementation Slices

- [ ] **Slice 3.1: Update runtime loader docs**

  In `docs/zircon_runtime/plugin/native_plugin_loader/index.md`:

  - Add `NativePluginLoadReport::behavior_health_for_plugin(...)` to the Report Flow section.
  - State that the helper returns worst health across runtime/editor behavior validation reports and returns `None` when no behavior validation report exists.
  - Append the exact Milestone 1 validation commands and results to Acceptance Evidence.

- [ ] **Slice 3.2: Update Plugin Manager docs**

  In `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md`:

  - Add `zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_validation.rs` to `related_code` and `implementation_files`.
  - Mention that Plugin Manager rows now include `Native: Clean`, `Native: Degraded`, or `Native: Invalid` in the existing metadata line when native behavior validation exists.
  - State that details remain in row diagnostics and that `load_state` and actions are unchanged.
  - Append scoped Milestone 2 and Milestone 3 validation evidence to the `tests` header or validation section.

- [ ] **Slice 3.3: Update coordination note**

  In `.codex/sessions/20260520-1320-plugin-manager-native-validation-ux.md`:

  - Set `status` to `active-implementation-row-health` during implementation.
  - Record touched modules as the runtime load report helper, editor status projection, Plugin Manager payload/projection, and docs.
  - Record validation commands and any blockers.

### Testing Stage

- [ ] Run scoped formatting checks:

  ```powershell
  cargo fmt -p zircon_runtime --check
  cargo fmt -p zircon_editor --check
  ```

  Expected result: both package formatting checks pass. If `cargo fmt --all --check` is attempted and fails on unrelated active-session files, record the unrelated file path and keep scoped package evidence.

- [ ] Run runtime focused validation:

  ```powershell
  cargo test -p zircon_runtime --lib native_plugin_load_report --locked --jobs 1
  ```

  Expected result: runtime aggregate helper tests pass.

- [ ] Run editor focused validation:

  ```powershell
  cargo test -p zircon_editor --lib native_plugin_status_report --locked --jobs 1
  cargo test -p zircon_editor --lib module_plugins --locked --jobs 1
  ```

  Expected result: editor native status and Plugin Manager projection tests pass.

- [ ] Run package-level type checks if the focused tests did not already compile both affected crates cleanly:

  ```powershell
  cargo check -p zircon_runtime --lib --locked --jobs 1
  cargo check -p zircon_editor --lib --locked --jobs 1
  ```

  Expected result: both checks pass or only pre-existing unrelated warnings remain.

- [ ] If validation fails, diagnose from lower to upper layers:

  - Runtime report helper and tests first.
  - Editor native status label next.
  - View/payload/retained-host field copies next.
  - Row metadata rendering last.

### Lightweight Checks

- [ ] No additional lightweight checks are needed after the testing stage unless a correction loop edits Rust again.

### Exit Evidence

- Runtime docs mention the aggregate helper and scoped test evidence.
- Plugin Manager docs mention the row health label, diagnostics ownership, and scoped test evidence.
- Focused formatting and tests pass for touched crates.
- Session note records evidence and blockers.

## Overall Acceptance Criteria

- Native plugin rows show `Native: Clean`, `Native: Degraded`, or `Native: Invalid` only when runtime load reports contain behavior validation health.
- Native rows without loaded behavior validation omit the native validation segment.
- Detailed validation diagnostics remain in existing row diagnostics.
- `load_state` remains focused on discovery/load status.
- Plugin Manager action labels and action IDs remain unchanged.
- No ABI declarations, callback signatures, or native fixture behavior are changed.
- Documentation under `docs/zircon_runtime/...` and `docs/editor-and-tooling/...` is updated with related code, tests, and evidence.
- No unrelated active-session files are modified.

## Execution Notes

- Work from the existing `main` checkout unless the user explicitly requests a worktree.
- Do not commit unless the user explicitly requests a commit.
- Preserve unrelated dirty worktree changes.
- If `cargo fmt --all --check` fails on unrelated files, do not reformat or revert those files; record the blocker and keep scoped package formatting evidence.
- If full workspace build/test is requested later, use the repository validator or shared target directory policy to avoid unnecessary local target bloat.
