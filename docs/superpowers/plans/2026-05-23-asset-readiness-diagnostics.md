# Asset Readiness Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a read-only typed asset readiness report that explains root and dependency readiness causes while preserving current import diagnostics through runtime registration.

**Architecture:** The implementation stays in `zircon_runtime::asset::facade` for report DTOs and graph traversal, with `zircon_runtime::core::resource::ResourceManager` remaining authoritative for identity, records, diagnostics, dependency IDs, payload residency, and events. The only lower-layer behavior change is preserving caller-supplied ready-record diagnostics in `ResourceManager::register_ready`; render material readiness stays in `zircon_runtime::graphics` and is documented as a downstream boundary.

**Tech Stack:** Rust, Cargo, `zircon_runtime`, `zircon_runtime::asset::facade`, `zircon_runtime::core::resource`, existing runtime lib tests under `zircon_runtime/src/asset/tests/facade.rs` and `zircon_runtime/src/core/resource/tests.rs`.

---

## Current Baseline

- The root workspace is fixed around `zircon_app`, `zircon_runtime`, `zircon_editor`, `zircon_runtime_interface`, and `zircon_hub`; this plan touches only `zircon_runtime` code/docs plus this plan document and the active coordination note.
- `zircon_runtime/src/asset/facade/load_state.rs` already defines `AssetLoadState`, `DependencyLoadState`, `RecursiveDependencyLoadState`, and `AssetLoadStates`.
- `zircon_runtime/src/asset/facade/manager.rs` already exposes typed `load_state`, `dependency_load_state`, `recursive_dependency_load_state`, `load_states`, and loaded predicates. Those queries are read-only and do not call `ensure_resident`.
- `ResourceRecord.diagnostics` is the existing shared diagnostic storage under `zircon_runtime_interface/src/resource/resource_record.rs` and `zircon_runtime_interface/src/resource/diagnostic.rs`.
- `zircon_runtime/src/asset/project/manager/scan_and_import.rs` already copies importer diagnostics and unresolved dependency diagnostics into project `ResourceRecord`s.
- `zircon_runtime/src/core/resource/manager/payload_ops.rs::register_ready` currently clears `record.diagnostics`, which can drop successful-import diagnostics during runtime resource sync.
- `docs/zircon_runtime/asset/facade.md` owns the asset facade module documentation.
- Active coordination warning: `.codex/sessions/20260523-0805-asset-mesh-metadata-slice.md` owns mesh metadata and glTF propagation files. Do not edit `zircon_runtime/src/asset/assets/mesh/**`, glTF importer propagation files, glTF plugin files, or mesh docs for this plan.
- Known broader validation blockers may include unrelated `cargo fmt --all --check` differences in `zircon_runtime/src/scene/world/schedule.rs`, active Cargo/Rust queues, and root lockfile dependency drift. Record those blockers; do not fix them in this plan.

## File Structure

- Create `zircon_runtime/src/asset/facade/readiness.rs`: define `AssetReadinessReport`, `AssetReadinessNode`, `AssetDependencyReadiness`, typed report helpers, synthetic diagnostics, and dependency graph traversal.
- Modify `zircon_runtime/src/asset/facade/mod.rs`: add `mod readiness;` and re-export the new report DTOs.
- Modify `zircon_runtime/src/asset/mod.rs`: re-export the new facade report DTOs from the crate-level asset surface.
- Modify `zircon_runtime/src/core/resource/manager/payload_ops.rs`: stop clearing caller-supplied diagnostics in `register_ready`.
- Modify `zircon_runtime/src/core/resource/tests.rs`: add focused resource-manager coverage for diagnostics preservation and stale diagnostic clearing.
- Modify `zircon_runtime/src/asset/tests/facade.rs`: add focused readiness report coverage for root diagnostics, dependency rows, missing records, wrong-kind roots, non-resident roots, direct/recursive classification, and cycles.
- Modify `docs/zircon_runtime/asset/facade.md`: document the new report surface, diagnostics preservation, read-only behavior, Bevy/Fyrox alignment, and render readiness boundary.
- Update `.codex/sessions/20260523-0744-asset-stack-next-slice-design.md`: record planning status, implementation status, validation evidence, and blockers during execution.

## Milestone 1: Resource Diagnostic Preservation

### Goal

Preserve current ready-record diagnostics through `ResourceManager::register_ready` so the later facade report can surface diagnostics that importers and dependency resolution already produced.

### In-Scope Behaviors

- A ready registration with diagnostics stores those diagnostics in the registry record.
- A later ready registration for the same resource with an empty diagnostic list replaces the previous diagnostics with an empty list.
- Failed reload diagnostics and error records keep their existing behavior.
- Revision behavior remains governed by `next_ready_revision`; this plan does not special-case diagnostics for revision calculation.

### Dependencies

- Existing `ResourceRecord.diagnostics` field.
- Existing `ResourceManager::register_ready` storage path.
- Existing `ResourceManager::fail_reload` behavior covered by `manager_failed_reload_keeps_last_good_payload_and_emits_events`.

### Implementation Slices

- [ ] **Slice 1: Add resource-manager diagnostics preservation test**

  Edit `zircon_runtime/src/core/resource/tests.rs` after `register_ready_is_idempotent_for_unchanged_records`.

  Add this test:

  ```rust
  #[test]
  fn register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics() {
      let manager = ResourceManager::new();
      let locator = locator("res://materials/diagnostic.zmaterial");
      let id = ResourceId::from_locator(&locator);
      let diagnostic = ResourceDiagnostic::error("shader contract warning");
      let record = ResourceRecord::new(id, ResourceKind::Material, locator.clone())
          .with_diagnostics(vec![diagnostic.clone()]);

      manager.register_ready(record, TestPayload { name: "material" });

      assert_eq!(
          manager
              .registry()
              .get(id)
              .expect("record exists")
              .diagnostics,
          vec![diagnostic]
      );

      manager.register_ready(
          ResourceRecord::new(id, ResourceKind::Material, locator),
          TestPayload { name: "material" },
      );

      assert!(
          manager
              .registry()
              .get(id)
              .expect("record exists")
              .diagnostics
              .is_empty(),
          "a clean ready record must replace stale diagnostics"
      );
  }
  ```

- [ ] **Slice 2: Preserve caller-supplied ready diagnostics**

  Edit `zircon_runtime/src/core/resource/manager/payload_ops.rs`.

  Replace the diagnostic-clearing block in `register_ready`:

  ```rust
  record.state = ResourceState::Ready;
  record.diagnostics.clear();
  record.revision = previous
      .as_ref()
      .map_or(1, |current| next_ready_revision(current, &record));
  ```

  With this code:

  ```rust
  record.state = ResourceState::Ready;
  record.revision = previous
      .as_ref()
      .map_or(1, |current| next_ready_revision(current, &record));
  ```

  Do not merge previous diagnostics into the new record. The caller-supplied `ResourceRecord.diagnostics` is the current truth for this registration.

- [ ] **Slice 3: Update active coordination note**

  Edit `.codex/sessions/20260523-0744-asset-stack-next-slice-design.md` and record that Milestone 1 touches `zircon_runtime/src/core/resource/manager/payload_ops.rs` and `zircon_runtime/src/core/resource/tests.rs`.

### Testing Stage: Resource Diagnostic Preservation

- [ ] Run the focused resource test only after the implementation slices are complete:

  ```powershell
  cargo test -p zircon_runtime --lib register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  ```

  Expected: the new test passes.

- [ ] Run existing related resource regression coverage:

  ```powershell
  cargo test -p zircon_runtime --lib manager_failed_reload_keeps_last_good_payload_and_emits_events --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  ```

  Expected: the existing failed-reload diagnostic behavior still passes.

- [ ] Debug/correction loop:

  If either test fails, inspect `payload_ops.rs`, `reload_ops.rs`, and `revision.rs` in that order. Do not edit asset facade, importer, watcher, render, editor, plugin, or lockfile files to make this milestone pass.

### Exit Evidence

- The two focused resource tests above pass, or a concrete unrelated Cargo/build blocker is recorded in the active session note.
- `payload_ops.rs` no longer clears current ready-record diagnostics.
- No asset facade API has changed yet.

## Milestone 2: Typed Asset Readiness Report Facade

### Goal

Add a typed, read-only `ProjectAssetManager::readiness_report<TAsset>(handle)` query that returns root readiness, aggregate load states, and per-dependency readiness rows without mutating resource residency.

### In-Scope Behaviors

- `AssetReadinessReport` contains `root`, `load_states`, and `dependencies`.
- `AssetReadinessNode` contains root id, optional locator, optional kind, optional revision, root load state, and diagnostics.
- `AssetDependencyReadiness` contains dependency id, optional locator, optional kind, optional revision, depth, direct flag, load state, and diagnostics.
- Missing roots and wrong-kind roots return value reports with `NotLoaded` aggregate states and synthetic error diagnostics.
- Correct roots use the same `AssetLoadState::from_resource` payload-residency semantics as `load_states`.
- Dependency rows are derived from `ResourceRecord.dependency_ids`, not from importer metadata or render material dependency sets.
- Missing dependency records become failed rows with synthetic diagnostics.
- Duplicate dependencies keep the shallowest depth and set `direct = true` if any direct edge exists.
- Cycles terminate without duplicate infinite rows.
- The query does not call `ensure_resident`, `load_artifact_by_id`, importers, watchers, graphics, or render preparation.

### Dependencies

- Milestone 1 preserves diagnostics through ready registration.
- Existing `AssetLoadStates` aggregate semantics.
- Existing `ResourceManager::registry`, `runtime_state`, typed `get`, and `get_untyped` query APIs.

### Implementation Slices

- [ ] **Slice 1: Add readiness report module and DTOs**

  Create `zircon_runtime/src/asset/facade/readiness.rs` with these imports and public DTO shapes:

  ```rust
  use std::collections::{HashMap, HashSet, VecDeque};

  use super::{Asset, AssetLoadState, AssetLoadStates, Handle};
  use crate::asset::{AssetId, AssetKind, AssetUri, ProjectAssetManager};
  use crate::core::resource::{ResourceDiagnostic, ResourceMarker, ResourceRecord};

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct AssetReadinessReport {
      pub root: AssetReadinessNode,
      pub load_states: AssetLoadStates,
      pub dependencies: Vec<AssetDependencyReadiness>,
  }

  impl AssetReadinessReport {
      pub fn is_loaded(&self) -> bool {
          self.load_states.is_loaded()
      }

      pub fn is_loaded_with_direct_dependencies(&self) -> bool {
          self.load_states.is_loaded_with_direct_dependencies()
      }

      pub fn is_loaded_with_dependencies(&self) -> bool {
          self.load_states.is_loaded_with_dependencies()
      }
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct AssetReadinessNode {
      pub id: AssetId,
      pub locator: Option<AssetUri>,
      pub kind: Option<AssetKind>,
      pub revision: Option<u64>,
      pub load_state: AssetLoadState,
      pub diagnostics: Vec<ResourceDiagnostic>,
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct AssetDependencyReadiness {
      pub id: AssetId,
      pub locator: Option<AssetUri>,
      pub kind: Option<AssetKind>,
      pub revision: Option<u64>,
      pub depth: u32,
      pub direct: bool,
      pub load_state: AssetLoadState,
      pub diagnostics: Vec<ResourceDiagnostic>,
  }
  ```

  Keep these types free of handles, leases, trait objects, renderer objects, or project-manager references.

- [ ] **Slice 2: Add public typed report method**

  In `readiness.rs`, add this public `ProjectAssetManager` impl:

  ```rust
  impl ProjectAssetManager {
      pub fn readiness_report<TAsset: Asset>(
          &self,
          handle: Handle<TAsset>,
      ) -> AssetReadinessReport {
          let record = self.resource_manager().registry().get(handle.id()).cloned();
          let load_states = self.load_states(handle);
          let root = self.readiness_root_node::<TAsset>(handle, record.as_ref());
          let dependencies = record
              .as_ref()
              .filter(|record| record.kind == TAsset::Marker::KIND)
              .map(|record| self.collect_dependency_readiness(record.id, &record.dependency_ids))
              .unwrap_or_default();

          AssetReadinessReport {
              root,
              load_states,
              dependencies,
          }
      }
  }
  ```

  This method must return an `AssetReadinessReport` for every handle. Do not return `Result` and do not call `load<TAsset>` or `ensure_loaded`.

- [ ] **Slice 3: Add root node construction helpers**

  In `readiness.rs`, add these private helpers below the public method:

  ```rust
  impl ProjectAssetManager {
      fn readiness_root_node<TAsset: Asset>(
          &self,
          handle: Handle<TAsset>,
          record: Option<&ResourceRecord>,
      ) -> AssetReadinessNode {
          let Some(record) = record else {
              return AssetReadinessNode {
                  id: handle.id(),
                  locator: None,
                  kind: None,
                  revision: None,
                  load_state: AssetLoadState::NotLoaded,
                  diagnostics: vec![ResourceDiagnostic::error(format!(
                      "missing asset record {}",
                      handle.id()
                  ))],
              };
          };

          let mut diagnostics = record.diagnostics.clone();
          if record.kind != TAsset::Marker::KIND {
              diagnostics.push(ResourceDiagnostic::error(format!(
                  "asset {} was {:?}, not {:?}",
                  record.primary_locator,
                  record.kind,
                  TAsset::Marker::KIND
              )));
              return AssetReadinessNode {
                  id: record.id,
                  locator: Some(record.primary_locator.clone()),
                  kind: Some(record.kind),
                  revision: Some(record.revision),
                  load_state: AssetLoadState::NotLoaded,
                  diagnostics,
              };
          }

          let load_state = AssetLoadState::from_resource(
              Some(record),
              self.resource_manager().runtime_state(record.id),
              self.resource_manager()
                  .get::<TAsset::Marker, TAsset>(handle.resource_handle())
                  .is_some(),
          );

          AssetReadinessNode {
              id: record.id,
              locator: Some(record.primary_locator.clone()),
              kind: Some(record.kind),
              revision: Some(record.revision),
              load_state,
              diagnostics,
          }
      }
  }
  ```

  If the compiler reports that `ResourceMarker` is unused or unnecessary for `TAsset::Marker::KIND`, remove that import only. Do not weaken the typed kind check.

- [ ] **Slice 4: Add dependency traversal helpers**

  In `readiness.rs`, add dependency traversal helpers below the root helpers:

  ```rust
  impl ProjectAssetManager {
      fn collect_dependency_readiness(
          &self,
          root_id: AssetId,
          dependency_ids: &[AssetId],
      ) -> Vec<AssetDependencyReadiness> {
          let mut rows = Vec::new();
          let mut row_by_id = HashMap::new();
          let mut expanded = HashSet::new();
          expanded.insert(root_id);

          let mut queue = VecDeque::new();
          for dependency_id in dependency_ids {
              queue.push_back((*dependency_id, 1_u32, true));
          }

          while let Some((dependency_id, depth, direct)) = queue.pop_front() {
              let record = self.resource_manager().registry().get(dependency_id).cloned();
              let row = self.dependency_readiness_row(dependency_id, record.as_ref(), depth, direct);
              upsert_dependency_row(&mut rows, &mut row_by_id, row);

              let Some(record) = record else {
                  continue;
              };
              if !expanded.insert(dependency_id) {
                  continue;
              }
              for nested in &record.dependency_ids {
                  queue.push_back((*nested, depth + 1, false));
              }
          }

          rows
      }

      fn dependency_readiness_row(
          &self,
          dependency_id: AssetId,
          record: Option<&ResourceRecord>,
          depth: u32,
          direct: bool,
      ) -> AssetDependencyReadiness {
          let Some(record) = record else {
              return AssetDependencyReadiness {
                  id: dependency_id,
                  locator: None,
                  kind: None,
                  revision: None,
                  depth,
                  direct,
                  load_state: AssetLoadState::Failed,
                  diagnostics: vec![ResourceDiagnostic::error(format!(
                      "missing asset dependency record {}",
                      dependency_id
                  ))],
              };
          };

          AssetDependencyReadiness {
              id: record.id,
              locator: Some(record.primary_locator.clone()),
              kind: Some(record.kind),
              revision: Some(record.revision),
              depth,
              direct,
              load_state: AssetLoadState::from_resource(
                  Some(record),
                  self.resource_manager().runtime_state(record.id),
                  self.resource_manager().get_untyped(record.id).is_some(),
              ),
              diagnostics: record.diagnostics.clone(),
          }
      }
  }

  fn upsert_dependency_row(
      rows: &mut Vec<AssetDependencyReadiness>,
      row_by_id: &mut HashMap<AssetId, usize>,
      row: AssetDependencyReadiness,
  ) {
      if let Some(index) = row_by_id.get(&row.id).copied() {
          let existing = &mut rows[index];
          if row.depth < existing.depth {
              existing.depth = row.depth;
          }
          existing.direct |= row.direct;
          return;
      }

      row_by_id.insert(row.id, rows.len());
      rows.push(row);
  }
  ```

  Preserve breadth-first ordering so direct dependencies appear before nested dependencies.

- [ ] **Slice 5: Re-export report DTOs**

  Edit `zircon_runtime/src/asset/facade/mod.rs`.

  Add the module declaration:

  ```rust
  mod readiness;
  ```

  Add the public exports:

  ```rust
  pub use readiness::{AssetDependencyReadiness, AssetReadinessNode, AssetReadinessReport};
  ```

  Edit `zircon_runtime/src/asset/mod.rs`.

  Update the facade re-export block to include the new report types:

  ```rust
  pub use facade::{
      Asset, AssetDependencyReadiness, AssetEvent, AssetEventReceiver, AssetLoadState,
      AssetLoadStates, AssetReadinessNode, AssetReadinessReport, Assets, DependencyLoadState,
      Handle, RecursiveDependencyLoadState,
  };
  ```

- [ ] **Slice 6: Add facade readiness tests**

  Edit `zircon_runtime/src/asset/tests/facade.rs`.

  Update the top asset import list to include the new DTOs:

  ```rust
  use crate::asset::{
      AlphaMode, AssetDependencyReadiness, AssetEvent, AssetLoadState, AssetLoadStates,
      AssetReference, AssetUri, Assets, DependencyLoadState, Handle, MaterialAsset, MeshAsset,
      ProjectAssetManager, RecursiveDependencyLoadState, ShaderAsset, ShaderEntryPointAsset,
      ShaderSourceLanguage, TextureAsset, UiLayoutAsset, UiV2ViewAsset,
  };
  ```

  Add this helper near the existing helper functions:

  ```rust
  fn diagnostic_messages(diagnostics: &[ResourceDiagnostic]) -> Vec<&str> {
      diagnostics
          .iter()
          .map(|diagnostic| diagnostic.message.as_str())
          .collect()
  }
  ```

  Add this test after `load_states_separate_root_direct_and_recursive_dependency_state`:

  ```rust
  #[test]
  fn readiness_report_exposes_loaded_dependency_rows_and_record_diagnostics() {
      let manager = ProjectAssetManager::default();
      let texture_diagnostic = ResourceDiagnostic::error("texture importer warning");
      let texture = record("res://textures/report.png", ResourceKind::Texture)
          .with_diagnostics(vec![texture_diagnostic.clone()]);
      let texture_id = texture.id;
      manager
          .assets::<TextureAsset>()
          .insert(texture, texture_asset("res://textures/report.png"))
          .expect("texture handle");

      let shader = record("res://shaders/report.wgsl", ResourceKind::Shader);
      let shader_id = shader.id;
      manager
          .assets::<ShaderAsset>()
          .insert(shader, shader_asset("res://shaders/report.wgsl"))
          .expect("shader handle");

      let root_diagnostic = ResourceDiagnostic::error("material shader contract warning");
      let mut material = record("res://materials/report.zmaterial", ResourceKind::Material)
          .with_diagnostics(vec![root_diagnostic.clone()]);
      material.dependency_ids = vec![shader_id, texture_id];
      let material_handle = manager
          .assets::<MaterialAsset>()
          .insert(material, material_asset("res://shaders/report.wgsl"))
          .expect("material handle");

      let report = manager.readiness_report(material_handle);

      assert_eq!(report.load_states.load_state, AssetLoadState::Loaded);
      assert!(report.is_loaded_with_dependencies());
      assert_eq!(report.root.diagnostics, vec![root_diagnostic]);
      assert_eq!(report.dependencies.len(), 2);
      let texture_row = report
          .dependencies
          .iter()
          .find(|row| row.id == texture_id)
          .expect("texture dependency row");
      assert_eq!(texture_row.depth, 1);
      assert!(texture_row.direct);
      assert_eq!(texture_row.load_state, AssetLoadState::Loaded);
      assert_eq!(texture_row.diagnostics, vec![texture_diagnostic]);
  }
  ```

  Add this test after `load_states_for_missing_wrong_kind_and_non_resident_roots_do_not_restore_payloads`:

  ```rust
  #[test]
  fn readiness_report_marks_missing_and_wrong_kind_roots_without_restoring_payloads() {
      let manager = ProjectAssetManager::default();
      let resource_manager = manager.resource_manager();
      let missing = Handle::<TextureAsset>::new(ResourceId::new());

      let missing_report = manager.readiness_report(missing);
      assert_eq!(missing_report.root.id, missing.id());
      assert_eq!(missing_report.root.load_state, AssetLoadState::NotLoaded);
      assert_eq!(missing_report.load_states, manager.load_states(missing));
      assert!(missing_report.dependencies.is_empty());
      assert!(diagnostic_messages(&missing_report.root.diagnostics)
          .iter()
          .any(|message| message.contains("missing asset record")));

      let material_record = record(
          "res://materials/report-wrong-kind.zmaterial",
          ResourceKind::Material,
      );
      let wrong_kind = Handle::<TextureAsset>::new(material_record.id);
      resource_manager.register_ready(
          material_record,
          material_asset("res://shaders/report-wrong-kind.wgsl"),
      );
      let wrong_kind_report = manager.readiness_report(wrong_kind);

      assert_eq!(wrong_kind_report.root.kind, Some(ResourceKind::Material));
      assert_eq!(wrong_kind_report.root.load_state, AssetLoadState::NotLoaded);
      assert_eq!(wrong_kind_report.load_states, manager.load_states(wrong_kind));
      assert!(wrong_kind_report.dependencies.is_empty());
      assert!(diagnostic_messages(&wrong_kind_report.root.diagnostics)
          .iter()
          .any(|message| message.contains("not Texture")));

      let texture_record = record("res://textures/report-non-resident.png", ResourceKind::Texture);
      let non_resident = manager
          .assets::<TextureAsset>()
          .insert(
              texture_record,
              texture_asset("res://textures/report-non-resident.png"),
          )
          .expect("texture handle");
      let lease = manager
          .assets::<TextureAsset>()
          .acquire(non_resident)
          .expect("resident lease");
      drop(lease);

      let non_resident_report = manager.readiness_report(non_resident);
      assert_eq!(non_resident_report.root.load_state, AssetLoadState::NotLoaded);
      assert_eq!(non_resident_report.load_states, manager.load_states(non_resident));
      assert!(resource_manager.get_untyped(non_resident.id()).is_none());
  }
  ```

  Add this test after `recursive_dependency_load_state_marks_missing_dependency_as_failed`:

  ```rust
  #[test]
  fn readiness_report_marks_missing_dependency_records_as_failed_rows() {
      let manager = ProjectAssetManager::default();
      let missing_id = ResourceId::from_stable_label("readiness missing dependency");
      let mut material = record("res://materials/report-missing.zmaterial", ResourceKind::Material);
      material.dependency_ids = vec![missing_id];
      let material_handle = manager
          .assets::<MaterialAsset>()
          .insert(
              material,
              material_asset("res://shaders/report-missing-dependency.wgsl"),
          )
          .expect("material handle");

      let report = manager.readiness_report(material_handle);

      assert_eq!(report.load_states.dependency_load_state, DependencyLoadState::Failed);
      assert_eq!(report.load_states.recursive_dependency_load_state, RecursiveDependencyLoadState::Failed);
      assert_eq!(report.dependencies.len(), 1);
      let row = &report.dependencies[0];
      assert_eq!(row.id, missing_id);
      assert_eq!(row.locator, None);
      assert_eq!(row.kind, None);
      assert_eq!(row.revision, None);
      assert_eq!(row.depth, 1);
      assert!(row.direct);
      assert_eq!(row.load_state, AssetLoadState::Failed);
      assert!(diagnostic_messages(&row.diagnostics)
          .iter()
          .any(|message| message.contains("missing asset dependency record")));
  }
  ```

  Add this test after `readiness_report_exposes_loaded_dependency_rows_and_record_diagnostics`:

  ```rust
  #[test]
  fn readiness_report_keeps_shallowest_direct_dependency_row_and_terminates_cycles() {
      let manager = ProjectAssetManager::default();

      let mut texture = record("res://textures/report-cycle.png", ResourceKind::Texture);
      let texture_id = texture.id;
      let mut shader = record("res://shaders/report-cycle.wgsl", ResourceKind::Shader);
      let shader_id = shader.id;
      texture.dependency_ids = vec![shader_id];
      shader.dependency_ids = vec![texture_id];

      manager
          .assets::<TextureAsset>()
          .insert(texture, texture_asset("res://textures/report-cycle.png"))
          .expect("texture handle");
      manager
          .assets::<ShaderAsset>()
          .insert(shader, shader_asset("res://shaders/report-cycle.wgsl"))
          .expect("shader handle");

      let mut material = record("res://materials/report-cycle.zmaterial", ResourceKind::Material);
      material.dependency_ids = vec![shader_id, texture_id];
      let material_handle = manager
          .assets::<MaterialAsset>()
          .insert(material, material_asset("res://shaders/report-cycle.wgsl"))
          .expect("material handle");

      let report = manager.readiness_report(material_handle);

      assert_eq!(report.dependencies.len(), 2);
      let shader_row = dependency_row(&report.dependencies, shader_id);
      assert_eq!(shader_row.depth, 1);
      assert!(shader_row.direct);
      let texture_row = dependency_row(&report.dependencies, texture_id);
      assert_eq!(texture_row.depth, 1, "direct edge must win over nested cycle path");
      assert!(texture_row.direct);
  }
  ```

  Add this helper near `diagnostic_messages`:

  ```rust
  fn dependency_row(
      rows: &[AssetDependencyReadiness],
      id: ResourceId,
  ) -> &AssetDependencyReadiness {
      rows.iter()
          .find(|row| row.id == id)
          .expect("dependency row")
  }
  ```

### Testing Stage: Facade Readiness Report Acceptance

- [ ] Run focused readiness report tests:

  ```powershell
  cargo test -p zircon_runtime --lib readiness_report --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  ```

  Expected: all `readiness_report_*` tests pass.

- [ ] Run the focused facade suite:

  ```powershell
  cargo test -p zircon_runtime --lib facade --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  ```

  Expected: all facade-filtered runtime lib tests pass, including the pre-existing load-state tests.

- [ ] Run scoped runtime lib/test type checking:

  ```powershell
  cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics
  ```

  Expected: `zircon_runtime` lib and tests check successfully.

- [ ] Debug/correction loop:

  If readiness report tests fail, diagnose in this order: DTO re-exports, root record kind handling, payload residency check, dependency row traversal, duplicate row merge, then test fixture setup. Do not patch importer, watcher, artifact, render, editor, plugin, mesh metadata, glTF propagation, or lockfile behavior for this milestone.

### Exit Evidence

- Focused readiness report tests pass, or a concrete unrelated Cargo/build blocker is recorded.
- The existing facade suite passes, or any failure is diagnosed and either fixed in-scope or recorded as unrelated.
- Scoped `cargo check -p zircon_runtime --lib --tests --locked` passes, or a concrete unrelated blocker is recorded.
- `ProjectAssetManager::readiness_report<TAsset>` is read-only and does not call acquisition or restore APIs.

## Milestone 3: Documentation, Hygiene, And Acceptance

### Goal

Document the new facade surface and run the milestone-level hygiene checks needed before reporting the asset readiness diagnostics slice complete.

### In-Scope Behaviors

- `docs/zircon_runtime/asset/facade.md` explains the readiness report DTOs, read-only semantics, diagnostics preservation, dependency row behavior, reference-engine alignment, and render-readiness boundary.
- The document header remains machine-readable and includes new implementation files and tests.
- The active session note records final validation evidence and blockers.
- Formatting/diff hygiene is checked without claiming workspace-wide success unless workspace validation is actually run and passes.

### Dependencies

- Milestone 1 diagnostics preservation is implemented or explicitly blocked.
- Milestone 2 readiness report facade is implemented or explicitly blocked.

### Implementation Slices

- [ ] **Slice 1: Update asset facade documentation header**

  Edit `docs/zircon_runtime/asset/facade.md`.

  Add these entries to `related_code` and `implementation_files`:

  ```yaml
  - zircon_runtime/src/asset/facade/readiness.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  ```

  Add this plan to `plan_sources`:

  ```yaml
  - docs/superpowers/specs/2026-05-23-asset-readiness-diagnostics-design.md
  - docs/superpowers/plans/2026-05-23-asset-readiness-diagnostics.md
  ```

  Add these test identifiers to `tests` after implementation evidence is available:

  ```yaml
  - zircon_runtime/src/core/resource/tests.rs::register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_exposes_loaded_dependency_rows_and_record_diagnostics
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_marks_missing_and_wrong_kind_roots_without_restoring_payloads
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_marks_missing_dependency_records_as_failed_rows
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_keeps_shallowest_direct_dependency_row_and_terminates_cycles
  ```

- [ ] **Slice 2: Document readiness report behavior**

  In `docs/zircon_runtime/asset/facade.md`, update the public surface section with this content:

  ```markdown
  - `AssetReadinessReport` is the typed diagnostic snapshot for a requested asset handle. It contains the root readiness node, aggregate `AssetLoadStates`, and per-dependency readiness rows.
  - `AssetReadinessNode` carries the requested asset id, optional locator/kind/revision metadata, root load state, and `ResourceDiagnostic` entries from the current record plus synthetic missing/wrong-kind diagnostics.
  - `AssetDependencyReadiness` carries dependency id, optional locator/kind/revision metadata, direct/recursive depth, direct-edge classification, load state, and diagnostics.
  - `ProjectAssetManager::readiness_report<TAsset>(handle)` is read-only. It does not restore payloads, run importers, touch artifacts, invoke graphics, or mutate lease-driven residency.
  ```

  Add or update a dependency diagnostics paragraph with this content:

  ```markdown
  Readiness reports walk `ResourceRecord.dependency_ids` breadth-first. Direct dependencies are depth `1`; nested dependencies use increasing depth. Duplicate dependency rows keep the shallowest depth and set `direct = true` when any direct edge exists. Missing dependency records produce failed rows with synthetic `ResourceDiagnostic::error(...)` entries so editor repair views do not collapse into indefinite loading.
  ```

  Add or update a diagnostics preservation paragraph with this content:

  ```markdown
  Ready resource registration preserves the diagnostics supplied on the current `ResourceRecord`. A clean successful reimport supplies an empty diagnostics list and clears stale diagnostics; a successful import with warnings or unresolved dependency diagnostics keeps those diagnostics visible after runtime sync.
  ```

  Add or update a render boundary paragraph with this content:

  ```markdown
  Asset readiness reports stop at source/import/resource/dependency readiness. Render-specific material readiness remains owned by `RenderMaterialReadinessReport` and graphics resource preparation, where shader contracts, texture upload support, fallback policy, and GPU/device constraints are known.
  ```

- [ ] **Slice 3: Update active session note during closeout**

  Edit `.codex/sessions/20260523-0744-asset-stack-next-slice-design.md`.

  Record:

  ```markdown
  ## Current Step
  - Implementation complete; validation stage in progress.

  ## Touched Modules
  - `zircon_runtime/src/asset/facade/readiness.rs`
  - `zircon_runtime/src/asset/facade/mod.rs`
  - `zircon_runtime/src/asset/mod.rs`
  - `zircon_runtime/src/core/resource/manager/payload_ops.rs`
  - `zircon_runtime/src/core/resource/tests.rs`
  - `zircon_runtime/src/asset/tests/facade.rs`
  - `docs/zircon_runtime/asset/facade.md`

  ## Related Plans And Tests
  - `docs/superpowers/specs/2026-05-23-asset-readiness-diagnostics-design.md`
  - `docs/superpowers/plans/2026-05-23-asset-readiness-diagnostics.md`
  ```

  After validation, add the exact commands and pass/fail results. If this slice completes cleanly and no handoff is needed, archive or delete the session note per the cross-session coordination rules.

### Testing Stage: Final Hygiene And Acceptance

- [ ] Run formatting check for touched Rust files without traversing unrelated files first:

  ```powershell
  rustfmt --edition 2021 --check zircon_runtime/src/asset/facade/readiness.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/core/resource/manager/payload_ops.rs zircon_runtime/src/core/resource/tests.rs zircon_runtime/src/asset/tests/facade.rs
  ```

  Expected: formatting passes for touched Rust files. If this fails because of touched-file formatting, run `rustfmt --edition 2021` on the same file list, inspect the diff, and rerun the check.

- [ ] Attempt full workspace format check only after focused formatting passes:

  ```powershell
  cargo fmt --all --check
  ```

  Expected: pass, or fail only on known unrelated files. If it fails on unrelated `zircon_runtime/src/scene/world/schedule.rs`, record the blocker and do not edit that file for this plan.

- [ ] Run final focused resource and facade tests:

  ```powershell
  cargo test -p zircon_runtime --lib register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  cargo test -p zircon_runtime --lib readiness_report --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  cargo test -p zircon_runtime --lib facade --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1
  ```

  Expected: all focused tests pass.

- [ ] Run scoped runtime check:

  ```powershell
  cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics
  ```

  Expected: `zircon_runtime` lib and tests check successfully.

- [ ] Run scoped diff hygiene:

  ```powershell
  git diff --check -- zircon_runtime/src/asset/facade/readiness.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/core/resource/manager/payload_ops.rs zircon_runtime/src/core/resource/tests.rs zircon_runtime/src/asset/tests/facade.rs docs/zircon_runtime/asset/facade.md docs/superpowers/specs/2026-05-23-asset-readiness-diagnostics-design.md docs/superpowers/plans/2026-05-23-asset-readiness-diagnostics.md .codex/sessions/20260523-0744-asset-stack-next-slice-design.md
  ```

  Expected: no whitespace errors in touched files. Repository line-ending warnings are acceptable only if `git diff --check` exits successfully.

- [ ] Debug/correction loop:

  If an upper-layer facade test fails, re-check Milestone 1 diagnostics preservation and the existing load-state projection first. If Cargo fails before running these tests because of active shared build queues, root lockfile drift, or unrelated formatting, record the exact blocker and do not claim full acceptance.

### Exit Evidence

- Focused touched-file `rustfmt --edition 2021 --check ...` passes.
- `cargo fmt --all --check` passes, or the unrelated formatting blocker is recorded with file path and reason.
- Focused resource and facade tests pass.
- `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics` passes, or a concrete unrelated blocker is recorded.
- `git diff --check -- ...` passes for touched files.
- `docs/zircon_runtime/asset/facade.md` records implementation files, plan sources, tests, semantics, and validation evidence.

## Plan Self-Review

- Spec coverage: Milestone 1 covers diagnostics preservation; Milestone 2 covers the typed read-only report API, DTOs, dependency traversal, synthetic diagnostics, missing/wrong-kind/non-resident behavior, duplicate and cycle handling, and loaded predicates; Milestone 3 covers docs, render boundary documentation, validation, and coordination evidence.
- Placeholder scan: No unfinished-marker text or unspecified validation commands remain. Test names, file paths, target dir, and expected outcomes are explicit.
- Type consistency: DTO and method names match `docs/superpowers/specs/2026-05-23-asset-readiness-diagnostics-design.md`: `AssetReadinessReport`, `AssetReadinessNode`, `AssetDependencyReadiness`, and `ProjectAssetManager::readiness_report<TAsset>`.
- Scope check: The plan intentionally excludes asset identity, `.zmeta` schema, importer output contracts, watcher/hot reload scheduling, mesh metadata, glTF propagation, render preparation, editor UI rendering, plugin workspace work, root lockfile edits, and workspace-wide success claims without fresh evidence.
