# Runtime Scene Absorption Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Absorb `LevelSystem` plus the scene runtime orchestration subtree into `zircon_runtime::scene`, keep `World`/serialization/component authority in `zircon_scene`, and eliminate the remaining scene root wildcard leak.

**Architecture:** `zircon_runtime::scene` becomes the owner of `LevelSystem`, `DefaultLevelManager`, `WorldDriver`, and their helper modules. Because `zircon_scene` cannot depend on `zircon_runtime`, the `LevelSystem`-specific `RenderExtractProducer` impl and runtime-semantics traits must move with `LevelSystem`; `zircon_scene` keeps only world/domain authority and explicit scene-domain exports.

**Tech Stack:** Rust, Cargo, crate-local structural tests with `include_str!`/`std::fs`, cross-crate import rewrites, workspace dependency cleanup.

---

## File Map

- Create: `zircon_runtime/src/scene/level_system.rs`
  - Runtime-owned `LevelSystem`, `LevelLifecycleState`, and `LevelMetadata`
- Create: `zircon_runtime/src/scene/render_extract.rs`
  - `RenderExtractProducer for LevelSystem`
- Create: `zircon_runtime/src/scene/semantics.rs`
  - Runtime-owned `RuntimeObject` and `RuntimeSystem` traits plus `LevelSystem` impls
- Create: `zircon_runtime/src/scene/module/mod.rs`
  - Folder-backed scene runtime orchestration entry
- Create: `zircon_runtime/src/scene/module/core_error.rs`
- Create: `zircon_runtime/src/scene/module/default_level_manager.rs`
- Create: `zircon_runtime/src/scene/module/level_display_name.rs`
- Create: `zircon_runtime/src/scene/module/level_manager_facade.rs`
- Create: `zircon_runtime/src/scene/module/level_manager_lifecycle.rs`
- Create: `zircon_runtime/src/scene/module/level_manager_project_io.rs`
- Create: `zircon_runtime/src/scene/module/service_names.rs`
- Create: `zircon_runtime/src/scene/module/world_driver.rs`
  - Runtime-owned manager/driver implementation subtree
- Create: `zircon_runtime/src/scene/tests.rs`
  - Runtime-owned scene manager/lifecycle/semantics tests moved out of `zircon_scene`
- Modify: `zircon_runtime/src/scene/mod.rs`
  - Explicit runtime-owned exports plus explicit `zircon_scene` domain re-exports; remove wildcard
- Modify: `zircon_runtime/src/scene/module.rs`
  - Thin façade that forwards to `module/`
- Modify: `zircon_runtime/src/tests.rs`
  - Structural ownership test for scene absorption
- Modify: `zircon_scene/src/lib.rs`
  - Remove `LevelSystem`/manager exports; keep world/domain authority only
- Modify: `zircon_scene/src/render_extract.rs`
  - Keep only `World` extract behavior
- Modify: `zircon_scene/src/semantics.rs`
  - Keep only `EntityIdentity` and `ComponentData`; remove `LevelSystem`-owned runtime semantics
- Modify: `zircon_scene/src/tests/boundary.rs`
  - Assert runtime orchestration no longer lives in `zircon_scene`
- Modify: `zircon_scene/src/tests/asset_scene.rs`
  - Keep world/serializer coverage only; remove manager-owned assertions
- Modify: `zircon_scene/src/tests/mod.rs`
  - Drop tests that move to runtime
- Modify: `zircon_scene/Cargo.toml`
  - Remove dependencies that become unused after module/lifecycle extraction
- Delete: `zircon_scene/src/level_system.rs`
- Delete: `zircon_scene/src/module.rs`
- Delete: `zircon_scene/src/module/core_error.rs`
- Delete: `zircon_scene/src/module/default_level_manager.rs`
- Delete: `zircon_scene/src/module/level_display_name.rs`
- Delete: `zircon_scene/src/module/level_manager_facade.rs`
- Delete: `zircon_scene/src/module/level_manager_lifecycle.rs`
- Delete: `zircon_scene/src/module/level_manager_project_io.rs`
- Delete: `zircon_scene/src/module/service_names.rs`
- Delete: `zircon_scene/src/module/world_driver.rs`
  - Remove legacy runtime-orchestration ownership from `zircon_scene`
- Modify: `zircon_editor/src/core/editing/state/editor_state_construction.rs`
- Modify: `zircon_editor/src/core/editing/state/editor_state_project.rs`
- Modify: `zircon_editor/src/core/editing/state/editor_world_slot.rs`
- Modify: `zircon_editor/src/core/editor_event/runtime/accessors.rs`
- Modify: `zircon_editor/src/core/host/manager/project_access.rs`
- Modify: `zircon_editor/src/tests/editing/state.rs`
- Modify: `zircon_editor/src/tests/editing/support.rs`
- Modify: `zircon_editor/src/tests/editor_event/support.rs`
- Modify: `zircon_editor/src/tests/host/binding_dispatch.rs`
- Modify: `zircon_editor/src/tests/host/manager.rs`
- Modify: `zircon_editor/src/tests/workbench/project.rs`
- Modify: `zircon_editor/src/ui/slint_host/app/tests.rs`
- Modify: `zircon_app/src/entry/runtime_entry_app/mod.rs`
- Modify: `zircon_graphics/src/tests/project_render.rs`
  - Switch all `LevelSystem` / `DefaultLevelManager` imports to `zircon_runtime::scene`
- Modify: `.codex/sessions/20260418-1910-runtime-absorption-boundary-cutover.md`
  - Record the completed slice and new validation evidence

### Task 1: Lock The New Ownership With Failing Structural Tests

**Files:**
- Modify: `E:/Git/ZirconEngine/zircon_runtime/src/tests.rs`
- Modify: `E:/Git/ZirconEngine/zircon_scene/src/tests/boundary.rs`

- [ ] **Step 1: Add the runtime ownership test**

```rust
#[test]
fn scene_runtime_orchestration_and_level_system_are_absorbed_into_runtime_scene_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_mod = runtime_root.join("src/scene/mod.rs");
    let level_system = runtime_root.join("src/scene/level_system.rs");
    let module_dir = runtime_root.join("src/scene/module/mod.rs");
    let legacy_scene_lib = runtime_root.join("../zircon_scene/src/lib.rs");

    let scene_mod_source = std::fs::read_to_string(&scene_mod).unwrap_or_default();
    let legacy_scene_lib_source = std::fs::read_to_string(&legacy_scene_lib).unwrap_or_default();

    assert!(level_system.exists(), "runtime scene should own LevelSystem");
    assert!(module_dir.exists(), "runtime scene should own folder-backed module orchestration");
    assert!(scene_mod_source.contains("LevelSystem"));
    assert!(scene_mod_source.contains("DefaultLevelManager"));
    assert!(scene_mod_source.contains("WorldDriver"));
    assert!(
        !scene_mod_source.contains("pub use zircon_scene::*"),
        "runtime scene root should stop wildcard-re-exporting zircon_scene"
    );
    assert!(
        !legacy_scene_lib_source.contains("pub use level_system"),
        "zircon_scene root should stop exporting LevelSystem after runtime absorption"
    );
}
```

- [ ] **Step 2: Add the legacy scene root boundary test**

```rust
#[test]
fn scene_root_no_longer_owns_runtime_orchestration_files() {
    let scene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = std::fs::read_to_string(scene_root.join("src/lib.rs")).unwrap_or_default();

    assert!(
        !lib_source.contains("DefaultLevelManager"),
        "zircon_scene root should stop exporting DefaultLevelManager"
    );
    assert!(
        !lib_source.contains("WorldDriver"),
        "zircon_scene root should stop exporting WorldDriver"
    );
    assert!(
        !scene_root.join("src/level_system.rs").exists(),
        "LevelSystem should move out of zircon_scene"
    );
    assert!(
        !scene_root.join("src/module/default_level_manager.rs").exists(),
        "scene module manager implementation should move to zircon_runtime"
    );
}
```

- [ ] **Step 3: Run the new structural tests and verify they fail before implementation**

Run: `cargo test -p zircon_runtime scene_runtime_orchestration_and_level_system_are_absorbed_into_runtime_scene_surface --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: FAIL because `zircon_runtime/src/scene/level_system.rs` and `zircon_runtime/src/scene/module/mod.rs` do not exist yet and `zircon_runtime/src/scene/mod.rs` still contains `pub use zircon_scene::*`.

Run: `cargo test -p zircon_scene scene_root_no_longer_owns_runtime_orchestration_files --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: FAIL because `zircon_scene/src/lib.rs` still exports runtime-owned items and the legacy files still exist.

### Task 2: Move `LevelSystem` And Its Companion Runtime Traits Into `zircon_runtime::scene`

**Files:**
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/level_system.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/render_extract.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/semantics.rs`
- Modify: `E:/Git/ZirconEngine/zircon_runtime/src/scene/mod.rs`
- Modify: `E:/Git/ZirconEngine/zircon_scene/src/render_extract.rs`
- Modify: `E:/Git/ZirconEngine/zircon_scene/src/semantics.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/level_system.rs`

- [ ] **Step 1: Create the runtime-owned `LevelSystem` file by moving the current type intact**

```rust
// zircon_runtime/src/scene/level_system.rs
use std::sync::{Arc, Mutex};

use zircon_framework::scene::WorldHandle;
use zircon_scene::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelLifecycleState {
    Loaded,
    Unloaded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelMetadata {
    pub project_root: Option<String>,
    pub asset_uri: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Clone)]
pub struct LevelSystem {
    handle: WorldHandle,
    inner: Arc<Mutex<World>>,
    metadata: Arc<Mutex<LevelMetadata>>,
    lifecycle: Arc<Mutex<LevelLifecycleState>>,
    subsystems: Arc<Mutex<Vec<String>>>,
}
```

- [ ] **Step 2: Move the `LevelSystem`-specific render extract impl into runtime**

```rust
// zircon_runtime/src/scene/render_extract.rs
use zircon_framework::render::{RenderExtractContext, RenderExtractProducer, RenderFrameExtract};

use crate::scene::LevelSystem;

impl RenderExtractProducer for LevelSystem {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        self.with_world(|world| world.build_render_frame_extract(context))
    }
}
```

- [ ] **Step 3: Move `RuntimeObject` and `RuntimeSystem` beside `LevelSystem`**

```rust
// zircon_runtime/src/scene/semantics.rs
use crate::scene::LevelSystem;

pub trait RuntimeObject {
    fn object_kind(&self) -> &'static str;
}

pub trait RuntimeSystem: RuntimeObject {
    fn system_name(&self) -> &'static str;
}

impl RuntimeObject for LevelSystem {
    fn object_kind(&self) -> &'static str {
        "system"
    }
}

impl RuntimeSystem for LevelSystem {
    fn system_name(&self) -> &'static str {
        "LevelSystem"
    }
}
```

- [ ] **Step 4: Reduce `zircon_scene` to world-domain semantics only**

```rust
// zircon_scene/src/semantics.rs
use crate::EntityId;

pub trait EntityIdentity: Copy + Eq + Send + Sync {
    fn entity_id(self) -> EntityId;
}

pub trait ComponentData: Send + Sync + 'static {}

impl EntityIdentity for EntityId {
    fn entity_id(self) -> EntityId {
        self
    }
}
```

```rust
// zircon_scene/src/render_extract.rs
use zircon_framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderWorldSnapshotHandle,
};

use crate::World;

impl World {
    pub fn to_render_frame_extract(&self) -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(0),
            self.to_render_snapshot(),
        )
    }
}

impl RenderExtractProducer for World {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            context.world,
            self.build_viewport_render_packet(&context.request),
        )
    }
}
```

- [ ] **Step 5: Wire the new runtime scene root explicitly instead of wildcard-forwarding**

```rust
// zircon_runtime/src/scene/mod.rs
mod level_system;
mod module;
mod render_extract;
mod semantics;

pub use level_system::{LevelLifecycleState, LevelMetadata, LevelSystem};
pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, SceneModule,
    WorldDriver, DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME,
    WORLD_DRIVER_NAME,
};
pub use semantics::{RuntimeObject, RuntimeSystem};
pub use zircon_scene::{
    default_render_layer_mask, Active, ActiveInHierarchy, ActiveSelf, CameraComponent,
    ComponentData, DirectionalLight, EntityId, EntityIdentity, Hierarchy, LocalTransform,
    MeshRenderer, Mobility, Name, NodeId, NodeKind, NodeRecord, RenderLayerMask, Scene,
    SceneAssetSerializer, SceneNode, SceneProjectError, Schedule, SystemStage, World, WorldMatrix,
    WorldTransform,
};
```

- [ ] **Step 6: Run the targeted tests for the moved `LevelSystem` surface**

Run: `cargo test -p zircon_runtime scene_runtime_orchestration_and_level_system_are_absorbed_into_runtime_scene_surface --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: PASS

Run: `cargo test -p zircon_runtime --lib --locked --offline --target-dir target/codex-shared-b level_system -- --nocapture`

Expected: PASS or zero matching tests if the file contains only compile-owned runtime surface.

### Task 3: Move The Scene Manager/Driver Subtree Into Runtime And Rewrite Consumer Imports

**Files:**
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/mod.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/core_error.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/default_level_manager.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/level_display_name.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/level_manager_facade.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/level_manager_lifecycle.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/level_manager_project_io.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/service_names.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module/world_driver.rs`
- Create: `E:/Git/ZirconEngine/zircon_runtime/src/scene/tests.rs`
- Modify: `E:/Git/ZirconEngine/zircon_runtime/src/scene/module.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/core/editing/state/editor_state_construction.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/core/editing/state/editor_state_project.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/core/editing/state/editor_world_slot.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/runtime/accessors.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/project_access.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/tests/editing/support.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/tests/editor_event/support.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/tests/host/binding_dispatch.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/tests/host/manager.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/tests/workbench/project.rs`
- Modify: `E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs`
- Modify: `E:/Git/ZirconEngine/zircon_app/src/entry/runtime_entry_app/mod.rs`
- Modify: `E:/Git/ZirconEngine/zircon_graphics/src/tests/project_render.rs`
- Modify: `E:/Git/ZirconEngine/zircon_scene/src/tests/asset_scene.rs`
- Modify: `E:/Git/ZirconEngine/zircon_scene/src/tests/mod.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/core_error.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/default_level_manager.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/level_display_name.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/level_manager_facade.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/level_manager_lifecycle.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/level_manager_project_io.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/service_names.rs`
- Delete: `E:/Git/ZirconEngine/zircon_scene/src/module/world_driver.rs`

- [ ] **Step 1: Copy the existing manager/driver subtree into `zircon_runtime/src/scene/module/` with runtime-local imports**

```rust
// zircon_runtime/src/scene/module/default_level_manager.rs
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use zircon_framework::scene::WorldHandle;

use crate::scene::LevelSystem;

#[derive(Debug, Default)]
pub struct DefaultLevelManager {
    pub(super) next_handle: AtomicU64,
    pub(super) levels: Mutex<HashMap<WorldHandle, LevelSystem>>,
}
```

```rust
// zircon_runtime/src/scene/module/level_manager_lifecycle.rs
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use zircon_framework::scene::WorldHandle;
use zircon_scene::World;

use super::DefaultLevelManager;
use crate::scene::{LevelMetadata, LevelSystem};
```

- [ ] **Step 2: Make `zircon_runtime/src/scene/module.rs` a thin façade**

```rust
// zircon_runtime/src/scene/module.rs
mod module;

pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, SceneModule,
    WorldDriver, DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME,
    WORLD_DRIVER_NAME,
};
```

- [ ] **Step 3: Move runtime-owned tests out of `zircon_scene` and into `zircon_runtime::scene`**

```rust
// zircon_runtime/src/scene/tests.rs
use zircon_framework::scene::LevelManager;

use crate::scene::{DefaultLevelManager, RuntimeObject, RuntimeSystem};

#[test]
fn level_manager_produces_level_systems() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    assert!(manager.level(level.handle()).is_some());
}

#[test]
fn runtime_semantics_keep_ecs_roles_explicit() {
    let level = DefaultLevelManager::default().create_default_level();
    assert_eq!(level.object_kind(), "system");
    assert_eq!(level.system_name(), "LevelSystem");
}
```

- [ ] **Step 4: Rewrite every `LevelSystem` / `DefaultLevelManager` import to `zircon_runtime::scene`**

```rust
// before
use zircon_scene::{LevelSystem, NodeKind};

// after
use zircon_runtime::scene::LevelSystem;
use zircon_scene::NodeKind;
```

```rust
// before
use zircon_scene::DefaultLevelManager;

// after
use zircon_runtime::scene::DefaultLevelManager;
```

- [ ] **Step 5: Remove manager-owned assertions from `zircon_scene` tests and keep only world-domain coverage**

```rust
// zircon_scene/src/tests/mod.rs
mod asset_scene;
mod boundary;
mod component_structure;
mod support;
mod world_basics;
```

```rust
// zircon_scene/src/tests/asset_scene.rs
// keep `scene_assets_instantiate_world_with_asset_bound_meshes`
// keep `render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay`
// remove `level_manager_instantiates_and_saves_scene_assets`
```

- [ ] **Step 6: Run the affected crate tests after the import rewrite**

Run: `cargo test -p zircon_runtime --lib --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: PASS

Run: `cargo test -p zircon_scene --lib --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: PASS

Run: `cargo test -p zircon_graphics --lib project_render --locked --offline --target-dir target/codex-shared-b`

Expected: PASS

### Task 4: Clean The Legacy Scene Crate Surface, Dependencies, And Validation Evidence

**Files:**
- Modify: `E:/Git/ZirconEngine/zircon_scene/src/lib.rs`
- Modify: `E:/Git/ZirconEngine/zircon_scene/Cargo.toml`
- Modify: `E:/Git/ZirconEngine/.codex/sessions/20260418-1910-runtime-absorption-boundary-cutover.md`

- [ ] **Step 1: Shrink `zircon_scene/src/lib.rs` to world/domain exports only**

```rust
//! ECS worlds, persistence, and render extraction for runtime scene data.

pub type EntityId = u64;
pub type NodeId = EntityId;

mod components;
mod render_extract;
mod semantics;
mod serializer;
mod world;

pub use components::{
    default_render_layer_mask, Active, ActiveInHierarchy, ActiveSelf, CameraComponent,
    DirectionalLight, Hierarchy, LocalTransform, MeshRenderer, Mobility, Name, NodeKind,
    NodeRecord, RenderLayerMask, SceneNode, Schedule, SystemStage, WorldMatrix, WorldTransform,
};
pub use semantics::{ComponentData, EntityIdentity};
pub use serializer::SceneAssetSerializer;
pub use world::{SceneProjectError, World};

pub type Scene = World;
```

- [ ] **Step 2: Remove now-unused `zircon_scene` dependencies**

```toml
[dependencies]
image.workspace = true
serde.workspace = true
serde_json = "1.0.149"
thiserror.workspace = true
zircon_asset = { path = "../zircon_asset" }
zircon_framework = { path = "../zircon_framework" }
zircon_math = { path = "../zircon_math" }
zircon_resource = { path = "../zircon_resource" }
```

- [ ] **Step 3: Run the final validation stack for this slice**

Run: `cargo test -p zircon_runtime --lib --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: PASS

Run: `cargo test -p zircon_scene --lib --locked --offline --target-dir target/codex-shared-b -- --nocapture`

Expected: PASS

Run: `cargo test -p zircon_editor --lib --no-run --locked --offline --target-dir target/codex-shared-b`

Expected: link failure may still stop at missing local `skparagraph.lib`, but there should be no new namespace/import compile errors from the scene absorption slice.

Run: `cargo check --workspace --locked --offline --target-dir target/codex-shared-b`

Expected: PASS or fail only on already-known unrelated external blockers, not on `LevelSystem` / `DefaultLevelManager` ownership changes.

- [ ] **Step 4: Append the implementation result to the active session note**

```markdown
- 本轮把 `LevelSystem`、`DefaultLevelManager`、`WorldDriver` 与 scene module orchestration subtree 物理迁入 `zircon_runtime::scene`。
- `zircon_runtime/src/scene/mod.rs` 已删除 `pub use zircon_scene::*`，改成 runtime-owned items + explicit scene-domain re-exports。
- `zircon_scene` 根级 surface 已收窄为 world/domain authority；`zircon_scene/Cargo.toml` 同步删除 `zircon_core` / `zircon_manager` / `zircon_module` 依赖。
- `zircon_editor`、`zircon_app`、`zircon_graphics` 对 `LevelSystem` / `DefaultLevelManager` 的导入路径已统一切到 `zircon_runtime::scene`。
```

## Self-Review

- Spec coverage: covered `LevelSystem`, `DefaultLevelManager`, `WorldDriver`, root-surface tightening, consumer import rewrites, and validation. The only necessary adjustment from the approved spec is moving `RuntimeObject` / `RuntimeSystem` with `LevelSystem`; that is required to avoid a `zircon_scene -> zircon_runtime` cycle.
- Placeholder scan: no `TODO` / `TBD` markers remain; every task names exact files and commands.
- Type consistency: `LevelSystem`, `LevelLifecycleState`, `LevelMetadata`, `DefaultLevelManager`, and `WorldDriver` are named consistently across ownership, import-rewrite, and validation tasks.

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-04-19-runtime-scene-absorption-slice-implementation.md`.

Preferred execution in this repository is inline from the existing `main` checkout because the repo policy forbids worktrees/feature branches for this workflow. If you want a stop point, stop here; otherwise continue immediately with inline execution against this plan.
