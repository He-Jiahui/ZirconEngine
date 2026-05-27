# Asset Facade Load-State Convergence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a coherent typed asset load-state query surface that reports root, direct dependency, and recursive dependency state without introducing a second asset store.

**Architecture:** The implementation stays in `zircon_runtime::asset::facade` and reads existing `ResourceManager` records, runtime state, dependency IDs, and payload residency. `ResourceManager` remains authoritative for identity, payload storage, diagnostics, revisions, dependency IDs, and events; facade query methods never trigger `ensure_resident` or artifact IO.

**Tech Stack:** Rust, Cargo, `zircon_runtime`, `zircon_runtime::asset::facade`, `zircon_runtime::core::resource`, existing lib tests under `zircon_runtime/src/asset/tests/facade.rs`.

---

## Current Baseline

- `zircon_runtime/src/asset/facade/load_state.rs` defines `AssetLoadState` and `RecursiveDependencyLoadState`.
- `zircon_runtime/src/asset/facade/manager.rs` exposes `load<TAsset>`, `handle<TAsset>`, `assets<TAsset>`, `load_state`, `recursive_dependency_load_state`, `asset_load_state_by_id`, and event subscription helpers.
- `recursive_dependency_load_state` already walks `ResourceRecord.dependency_ids` recursively with cycle protection and precedence `Failed > Reloading > Loading > NotLoaded > Loaded`.
- `Assets<TAsset>::load_state` already projects root state from record state, runtime state, and typed payload presence.
- `docs/zircon_runtime/asset/facade.md` owns the module documentation for this behavior.
- Known broader blocker: Windows workspace-wide validation may fail before Zircon tests because of root `Cargo.lock` / `wgpu-hal` mixed `windows` crate drift. Do not fix that in this plan unless separately instructed.

## File Structure

- Modify `zircon_runtime/src/asset/facade/load_state.rs`: add `DependencyLoadState`, `AssetLoadStates`, helper predicates, and conversions from `AssetLoadState`.
- Modify `zircon_runtime/src/asset/facade/manager.rs`: add direct dependency aggregation, combined `load_states`, and loaded predicates; keep query helpers read-only.
- Modify `zircon_runtime/src/asset/facade/mod.rs`: re-export `AssetLoadStates` and `DependencyLoadState`.
- Modify `zircon_runtime/src/asset/mod.rs`: re-export the new facade public types from `crate::asset`.
- Modify `zircon_runtime/src/asset/tests/facade.rs`: extend facade coverage for combined states, direct dependency aggregation, wrong-kind/missing roots, non-resident payloads, and predicates.
- Modify `docs/zircon_runtime/asset/facade.md`: document the new combined query surface and update the test coverage text while keeping the existing header metadata current.
- Update `.codex/sessions/20260518-1841-asset-management-optimization-design.md`: record implementation status, validation evidence, and blockers during execution.

## Milestone 1: Typed Load-State Query Model

### Goal

Add the public state DTOs and query APIs needed for root, direct dependency, and recursive dependency state while preserving `ResourceManager` authority and read-only query behavior.

### In-Scope Behaviors

- `DependencyLoadState` mirrors direct dependency aggregate state with variants `NotLoaded`, `Loading`, `Loaded`, `Failed`, and `Reloading`.
- `AssetLoadStates` stores `load_state`, `dependency_load_state`, and `recursive_dependency_load_state`.
- `ProjectAssetManager::dependency_load_state<TAsset>(handle)` aggregates only direct dependencies.
- `ProjectAssetManager::load_states<TAsset>(handle)` returns one DTO with root, direct dependency, and recursive dependency state.
- `ProjectAssetManager::{is_loaded,is_loaded_with_direct_dependencies,is_loaded_with_dependencies}` are thin wrappers over state queries.
- Query methods do not call `ensure_resident` and do not mutate payload residency.

### Dependencies

- Existing `ResourceRecord.dependency_ids` population from scan/import milestones.
- Existing `AssetLoadState::from_resource` projection.
- Existing `ResourceManager::runtime_state`, `get_untyped`, and typed `get` methods.

### Implementation Slices

- [ ] **Slice 1: Extend load-state types**

  Edit `zircon_runtime/src/asset/facade/load_state.rs` to add `DependencyLoadState`, `AssetLoadStates`, and predicate methods. Keep the existing `AssetLoadState` and `RecursiveDependencyLoadState` variants unchanged.

  Implement these exact public shapes:

  ```rust
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum DependencyLoadState {
      NotLoaded,
      Loading,
      Loaded,
      Failed,
      Reloading,
  }

  impl DependencyLoadState {
      pub fn is_loading_class(&self) -> bool {
          matches!(self, Self::Loading | Self::Reloading)
      }

      pub fn is_loaded(&self) -> bool {
          matches!(self, Self::Loaded)
      }

      pub fn is_failed(&self) -> bool {
          matches!(self, Self::Failed)
      }
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct AssetLoadStates {
      pub load_state: AssetLoadState,
      pub dependency_load_state: DependencyLoadState,
      pub recursive_dependency_load_state: RecursiveDependencyLoadState,
  }

  impl AssetLoadStates {
      pub fn is_loaded(&self) -> bool {
          matches!(self.load_state, AssetLoadState::Loaded)
      }

      pub fn is_loaded_with_direct_dependencies(&self) -> bool {
          self.is_loaded() && self.dependency_load_state.is_loaded()
      }

      pub fn is_loaded_with_dependencies(&self) -> bool {
          self.is_loaded_with_direct_dependencies()
              && matches!(
                  self.recursive_dependency_load_state,
                  RecursiveDependencyLoadState::Loaded
              )
      }
  }

  impl From<AssetLoadState> for DependencyLoadState {
      fn from(value: AssetLoadState) -> Self {
          match value {
              AssetLoadState::NotLoaded => Self::NotLoaded,
              AssetLoadState::Loading => Self::Loading,
              AssetLoadState::Loaded => Self::Loaded,
              AssetLoadState::Failed => Self::Failed,
              AssetLoadState::Reloading => Self::Reloading,
          }
      }
  }
  ```

- [ ] **Slice 2: Re-export new facade types**

  Edit `zircon_runtime/src/asset/facade/mod.rs` so the public facade export includes the new DTOs:

  ```rust
  pub use load_state::{
      AssetLoadState, AssetLoadStates, DependencyLoadState, RecursiveDependencyLoadState,
  };
  ```

  Edit `zircon_runtime/src/asset/mod.rs` so the crate-level asset export includes the new types:

  ```rust
  pub use facade::{
      Asset, AssetEvent, AssetEventReceiver, AssetLoadState, AssetLoadStates, Assets,
      DependencyLoadState, Handle, RecursiveDependencyLoadState,
  };
  ```

- [ ] **Slice 3: Add direct dependency aggregation helpers**

  Edit `zircon_runtime/src/asset/facade/manager.rs`.

  Update the import list from `super` to include `AssetLoadStates` and `DependencyLoadState`:

  ```rust
  use super::{
      Asset, AssetEventReceiver, AssetLoadState, AssetLoadStates, Assets, DependencyLoadState,
      Handle, RecursiveDependencyLoadState,
  };
  ```

  Add the public direct dependency and combined state methods near the existing `load_state` methods:

  ```rust
  pub fn dependency_load_state<TAsset: Asset>(
      &self,
      handle: Handle<TAsset>,
  ) -> DependencyLoadState {
      let record = self.resource_manager().registry().get(handle.id()).cloned();
      let Some(record) = record else {
          return DependencyLoadState::NotLoaded;
      };
      if record.kind != TAsset::Marker::KIND {
          return DependencyLoadState::NotLoaded;
      }

      self.aggregate_direct_dependency_state(&record.dependency_ids)
          .unwrap_or(DependencyLoadState::Loaded)
  }

  pub fn load_states<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadStates {
      AssetLoadStates {
          load_state: self.load_state(handle),
          dependency_load_state: self.dependency_load_state(handle),
          recursive_dependency_load_state: self.recursive_dependency_load_state(handle),
      }
  }

  pub fn is_loaded<TAsset: Asset>(&self, handle: Handle<TAsset>) -> bool {
      self.load_states(handle).is_loaded()
  }

  pub fn is_loaded_with_direct_dependencies<TAsset: Asset>(
      &self,
      handle: Handle<TAsset>,
  ) -> bool {
      self.load_states(handle).is_loaded_with_direct_dependencies()
  }

  pub fn is_loaded_with_dependencies<TAsset: Asset>(&self, handle: Handle<TAsset>) -> bool {
      self.load_states(handle).is_loaded_with_dependencies()
  }
  ```

  Add a private helper for one-level dependency aggregation:

  ```rust
  fn aggregate_direct_dependency_state(
      &self,
      dependency_ids: &[AssetId],
  ) -> Option<DependencyLoadState> {
      let mut aggregate = None;
      for dependency_id in dependency_ids {
          let dependency = self
              .resource_manager()
              .registry()
              .get(*dependency_id)
              .cloned();
          let dependency_state = self.dependency_record_load_state(dependency.as_ref());
          aggregate = combine_dependency_state(aggregate, dependency_state.into());
      }
      aggregate
  }
  ```

  Add the direct dependency precedence helpers next to the existing recursive helpers:

  ```rust
  fn combine_dependency_state(
      current: Option<DependencyLoadState>,
      next: DependencyLoadState,
  ) -> Option<DependencyLoadState> {
      Some(match current {
          Some(current) if dependency_rank(&current) >= dependency_rank(&next) => current,
          _ => next,
      })
  }

  fn dependency_rank(state: &DependencyLoadState) -> u8 {
      match state {
          DependencyLoadState::Loaded => 0,
          DependencyLoadState::NotLoaded => 1,
          DependencyLoadState::Loading => 2,
          DependencyLoadState::Reloading => 3,
          DependencyLoadState::Failed => 4,
      }
  }
  ```

  Keep `recursive_dependency_load_state` behavior unchanged except for any shared helper refactor needed by formatting.

- [ ] **Slice 4: Add facade test coverage**

  Edit `zircon_runtime/src/asset/tests/facade.rs`.

  Update the import list to include the new types:

  ```rust
  use crate::asset::{
      AlphaMode, AssetEvent, AssetLoadState, AssetLoadStates, AssetReference, AssetUri, Assets,
      DependencyLoadState, Handle, MaterialAsset, ProjectAssetManager,
      RecursiveDependencyLoadState, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
      TextureAsset, UiLayoutAsset, UiV2ViewAsset,
  };
  ```

  Add this test after `recursive_dependency_load_state_walks_nested_resource_dependencies`:

  ```rust
  #[test]
  fn load_states_separate_root_direct_and_recursive_dependency_state() {
      let manager = ProjectAssetManager::default();
      let resource_manager = manager.resource_manager();
      let texture = record("res://textures/nested.png", ResourceKind::Texture);
      let texture_id = texture.id;
      let texture_handle = manager
          .assets::<TextureAsset>()
          .insert(texture, texture_asset("res://textures/nested.png"))
          .expect("texture handle");
      let mut shader = record("res://shaders/nested.wgsl", ResourceKind::Shader);
      shader.dependency_ids = vec![texture_id];
      let shader_id = shader.id;
      manager
          .assets::<ShaderAsset>()
          .insert(shader, shader_asset("res://shaders/nested.wgsl"))
          .expect("shader handle");
      let mut material = record("res://materials/nested.zmaterial", ResourceKind::Material);
      material.dependency_ids = vec![shader_id];
      let material_handle = manager
          .assets::<MaterialAsset>()
          .insert(material, material_asset("res://shaders/nested.wgsl"))
          .expect("material handle");

      assert_eq!(
          manager.load_states(material_handle),
          AssetLoadStates {
              load_state: AssetLoadState::Loaded,
              dependency_load_state: DependencyLoadState::Loaded,
              recursive_dependency_load_state: RecursiveDependencyLoadState::Loaded,
          }
      );
      assert!(manager.is_loaded(material_handle));
      assert!(manager.is_loaded_with_direct_dependencies(material_handle));
      assert!(manager.is_loaded_with_dependencies(material_handle));

      resource_manager.start_reload(texture_id, Vec::new());
      assert_eq!(
          manager.dependency_load_state(material_handle),
          DependencyLoadState::Loaded,
          "direct dependency stays loaded when only nested dependency reloads"
      );
      assert_eq!(
          manager.recursive_dependency_load_state(material_handle),
          RecursiveDependencyLoadState::Reloading
      );
      assert!(manager.is_loaded_with_direct_dependencies(material_handle));
      assert!(!manager.is_loaded_with_dependencies(material_handle));

      let texture_payload = manager
          .assets::<TextureAsset>()
          .acquire(texture_handle)
          .expect("texture payload");
      drop(texture_payload);
      assert_eq!(
          manager.dependency_load_state(material_handle),
          DependencyLoadState::Loaded,
          "direct dependency aggregation does not walk grandchildren"
      );
  }
  ```

  Add this test after `recursive_dependency_load_state_marks_missing_dependency_as_failed`:

  ```rust
  #[test]
  fn dependency_load_state_applies_direct_precedence_and_missing_records() {
      let manager = ProjectAssetManager::default();
      let resource_manager = manager.resource_manager();
      let loaded_texture = record("res://textures/direct-loaded.png", ResourceKind::Texture);
      let loaded_id = loaded_texture.id;
      manager
          .assets::<TextureAsset>()
          .insert(
              loaded_texture,
              texture_asset("res://textures/direct-loaded.png"),
          )
          .expect("loaded texture handle");
      let pending = record("res://textures/direct-pending.png", ResourceKind::Texture)
          .with_state(ResourceState::Pending);
      let pending_id = pending.id;
      resource_manager.register_record(pending);
      let reloading = record("res://textures/direct-reloading.png", ResourceKind::Texture);
      let reloading_id = reloading.id;
      manager
          .assets::<TextureAsset>()
          .insert(
              reloading,
              texture_asset("res://textures/direct-reloading.png"),
          )
          .expect("reloading texture handle");
      resource_manager.start_reload(reloading_id, Vec::new());
      let missing_id = ResourceId::from_stable_label("direct missing dependency");
      let mut material = record("res://materials/direct.zmaterial", ResourceKind::Material);
      material.dependency_ids = vec![loaded_id, pending_id, reloading_id, missing_id];
      let material_handle = manager
          .assets::<MaterialAsset>()
          .insert(material, material_asset("res://shaders/direct.wgsl"))
          .expect("material handle");

      assert_eq!(
          manager.dependency_load_state(material_handle),
          DependencyLoadState::Failed,
          "missing direct dependencies outrank loading and reloading states"
      );

      let mut material_without_missing =
          record("res://materials/direct-no-missing.zmaterial", ResourceKind::Material);
      material_without_missing.dependency_ids = vec![loaded_id, pending_id, reloading_id];
      let material_without_missing_handle = manager
          .assets::<MaterialAsset>()
          .insert(
              material_without_missing,
              material_asset("res://shaders/direct.wgsl"),
          )
          .expect("material handle");

      assert_eq!(
          manager.dependency_load_state(material_without_missing_handle),
          DependencyLoadState::Reloading,
          "reloading outranks pending/loading when no dependency failed"
      );
  }
  ```

  Add this test near the root load-state tests:

  ```rust
  #[test]
  fn load_states_for_missing_wrong_kind_and_non_resident_roots_do_not_restore_payloads() {
      let manager = ProjectAssetManager::default();
      let resource_manager = manager.resource_manager();
      let missing = Handle::<TextureAsset>::new(ResourceId::new());

      assert_eq!(
          manager.load_states(missing),
          AssetLoadStates {
              load_state: AssetLoadState::NotLoaded,
              dependency_load_state: DependencyLoadState::NotLoaded,
              recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
          }
      );

      let material_record = record("res://materials/wrong-kind.zmaterial", ResourceKind::Material);
      let wrong_kind = Handle::<TextureAsset>::new(material_record.id);
      resource_manager.register_ready(
          material_record,
          material_asset("res://shaders/wrong-kind.wgsl"),
      );
      assert_eq!(
          manager.load_states(wrong_kind),
          AssetLoadStates {
              load_state: AssetLoadState::NotLoaded,
              dependency_load_state: DependencyLoadState::NotLoaded,
              recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
          }
      );

      let texture_record = record("res://textures/non-resident.png", ResourceKind::Texture);
      let non_resident = manager
          .assets::<TextureAsset>()
          .insert(
              texture_record,
              texture_asset("res://textures/non-resident.png"),
          )
          .expect("texture handle");
      let lease = manager
          .assets::<TextureAsset>()
          .acquire(non_resident)
          .expect("resident lease");
      drop(lease);

      assert_eq!(manager.load_state(non_resident), AssetLoadState::NotLoaded);
      assert_eq!(
          manager.load_states(non_resident),
          AssetLoadStates {
              load_state: AssetLoadState::NotLoaded,
              dependency_load_state: DependencyLoadState::Loaded,
              recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
          }
      );
      assert!(!manager.is_loaded(non_resident));
      assert!(!manager.is_loaded_with_direct_dependencies(non_resident));
      assert!(!manager.is_loaded_with_dependencies(non_resident));
      assert!(resource_manager.get_untyped(non_resident.id()).is_none());
  }
  ```

- [ ] **Slice 5: Update module documentation**

  Edit `docs/zircon_runtime/asset/facade.md`.

  Update the public surface section so line-equivalent content includes:

  ```markdown
  - `DependencyLoadState` maps direct dependency aggregate state using the same Zircon state vocabulary as root loads, including explicit `Reloading`.
  - `AssetLoadStates` groups root, direct dependency, and recursive dependency state so status panels and runtime callers can query one coherent snapshot without forcing payload residency.
  - `ProjectAssetManager` exposes `dependency_load_state(handle)`, `load_states(handle)`, `is_loaded(handle)`, `is_loaded_with_direct_dependencies(handle)`, and `is_loaded_with_dependencies(handle)` as read-only typed queries.
  ```

  Update the dependency behavior section so it states:

  ```markdown
  Direct dependency aggregation walks only the root record's `dependency_ids`; recursive aggregation walks the full dependency tree with cycle protection. Both use precedence `Failed > Reloading > Loading > NotLoaded > Loaded`, and both treat missing dependency records as `Failed`. Query APIs never call `ensure_resident`, so polling state cannot reload artifacts or mutate lease-driven residency.
  ```

  Update the test coverage section so it names the new facade tests for `AssetLoadStates`, `DependencyLoadState`, missing/wrong-kind roots, non-resident roots, and loaded predicates.

### Testing Stage: Facade Query Acceptance

- [ ] Run formatting check for the touched Rust files:

  ```powershell
  cargo fmt --all --check
  ```

  Expected: formatting passes. If it fails, run `cargo fmt --all`, inspect the diff, and rerun `cargo fmt --all --check`.

- [ ] Run focused facade tests:

  ```powershell
  cargo test -p zircon_runtime --lib facade --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence
  ```

  Expected: all `facade` lib tests pass, including the new load-state convergence tests.

- [ ] Run scoped runtime lib test compilation/check:

  ```powershell
  cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence
  ```

  Expected: `zircon_runtime` lib and tests check successfully.

- [ ] Run scoped whitespace/diff hygiene:

  ```powershell
  git diff --check -- zircon_runtime/src/asset/facade/load_state.rs zircon_runtime/src/asset/facade/manager.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/facade.rs docs/zircon_runtime/asset/facade.md docs/superpowers/plans/2026-05-18-asset-facade-load-state-convergence.md docs/superpowers/specs/2026-05-18-asset-facade-load-state-convergence-design.md .codex/sessions/20260518-1841-asset-management-optimization-design.md
  ```

  Expected: no whitespace errors in touched files.

- [ ] Debug/correction loop:

  If the focused tests fail, diagnose in this order: `AssetLoadState::from_resource`, direct aggregation, recursive aggregation, predicate semantics, then test fixture setup. Do not patch scan/import, watcher, artifact, render, editor, plugin, or lockfile behavior for this milestone.

- [ ] Acceptance evidence:

  Record the exact commands, pass/fail result, and any blocker in `.codex/sessions/20260518-1841-asset-management-optimization-design.md`. If Windows validation hits the known `wgpu-hal`/`windows` lockfile issue, record it and do not claim workspace-wide success.

### Exit Evidence

- `cargo fmt --all --check` passes or formatting was applied and the check passes after rerun.
- `cargo test -p zircon_runtime --lib facade --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence` passes.
- `cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence` passes.
- `git diff --check -- zircon_runtime/src/asset/facade/load_state.rs zircon_runtime/src/asset/facade/manager.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/facade.rs docs/zircon_runtime/asset/facade.md docs/superpowers/plans/2026-05-18-asset-facade-load-state-convergence.md docs/superpowers/specs/2026-05-18-asset-facade-load-state-convergence-design.md .codex/sessions/20260518-1841-asset-management-optimization-design.md` passes.
- `docs/zircon_runtime/asset/facade.md` documents the new public surface and validation coverage.

## Plan Self-Review

- Spec coverage: The plan covers `DependencyLoadState`, `AssetLoadStates`, combined and direct dependency query APIs, loaded predicates, read-only query behavior, Bevy-aligned state split, Zircon `Reloading` and missing-dependency divergence, docs, and focused validation.
- Placeholder scan: No placeholder commands or unfinished tasks remain; target dir is explicit.
- Type consistency: Public type and method names match the approved design and are re-exported from both `facade/mod.rs` and `asset/mod.rs`.
- Scope check: The plan intentionally excludes async loading, watcher invalidation, dependency persistence, editor UI, plugin workspace changes, and root lockfile changes.
