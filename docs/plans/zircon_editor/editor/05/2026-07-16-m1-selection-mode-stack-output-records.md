# Editor05 M1 selection and mode-stack output record

Plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
Milestone: M1.1
Status: in_progress
Files: ["Cargo.lock", "docs/zircon_editor/scene/modes.md", "docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md", "zircon_editor/Cargo.toml", "zircon_editor/src/scene/mod.rs", "zircon_editor/src/scene/modes/duplicate_scene_mode_error.rs", "zircon_editor/src/scene/modes/editor_scene_mode.rs", "zircon_editor/src/scene/modes/input_outcome.rs", "zircon_editor/src/scene/modes/mod.rs", "zircon_editor/src/scene/modes/scene_mode_ctx.rs", "zircon_editor/src/scene/modes/scene_mode_factory.rs", "zircon_editor/src/scene/modes/scene_mode_registration.rs", "zircon_editor/src/scene/modes/scene_mode_registry.rs", "zircon_editor/src/scene/modes/scene_mode_registry_error.rs", "zircon_editor/src/scene/modes/scene_mode_stack.rs", "zircon_editor/src/scene/modes/tests.rs", "zircon_editor/src/scene/modes/viewport_overlay_builder.rs", "zircon_editor/src/scene/selection/domain_selection.rs", "zircon_editor/src/scene/selection/mod.rs", "zircon_editor/src/scene/selection/selection_model.rs", "zircon_editor/src/scene/selection/tests.rs", "zircon_editor/src/scene/selection/world_domain.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_state.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_state_new.rs"]

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `IN_PROGRESS / VALIDATION + LOCKFILE BLOCKED` | 2026-07-16 | 已完成双域有序 `SelectionModel`、viewport 旧单值存储和 controller 单选 API 删除、`SceneModeStack` 生命周期/路由/overlay 边界、descriptor-backed factory registry 与中性 command-eval 投影，当前聚焦旧证据为 10/10。原审查 P2 已修正；P1 的 28 处 Editor07 consumer 已完成硬切并补齐多选/PIE/history/源码守卫，最终独立复审 `P0/P1/P2=0/0/0`，但 current-source 受管验证被 stale foreign CPU reservation 阻塞，不提前声明关闭。另一项 P1 混合 `Cargo.lock` 仍等待 foreign owner 先行落 HEAD。内建 Select/Transform、生产 command-eval、overlay provider lifecycle 尚未闭环，故 M1.1 保持进行中。 |

## Scope delivered

- Added ordered Edit/Play-domain `SelectionModel` state with primary-entity,
  per-domain generation, and cross-domain revision invariants.
- Removed the viewport's legacy `selected: Option<u64>` storage and the
  controller `selected_node` / `set_selected_node` API. The 28 Editor07
  production consumers now read and mutate the active `SelectionModel` domain
  directly; no alias, shim, deprecated wrapper, or parallel truth remains.
- Added `EditorSceneMode`, `SceneModeCtx`, `SceneModeStack`,
  `InputOutcome`, duplicate-id rejection, and the neutral
  `ViewportOverlayBuilder` boundary.
- Added `SceneModeFactory`, descriptor-backed `SceneModeRegistration`, and
  `SceneModeRegistry`. Registration rejects duplicate typed ids; creation
  rejects unknown ids and factories whose produced mode id differs from the
  registered `ViewportToolModeDescriptor` id.
- Added the neutral `SceneModeStack::project_command_eval_ctx` route. It
  projects the active typed mode id and actual active-domain selection count
  without replacing unrelated command-evaluation fields; production host
  consumption remains pending behind the active Editor07 host lease.
- Added module documentation and focused lifecycle, input-routing, ordering,
  generation, and domain-isolation tests.
- Kept Editor05 M1.1 open: built-in Select/Transform factory registration,
  production command-eval projection, and plugin overlay-provider lifecycle
  wiring are not yet complete.

## Fresh testing evidence

- TDD RED: managed Cargo job
  `dd6316fa17fa4e4da88c3de41a495bf2` produced
  `output-test-lib-zircon_editor` with three expected E0432 groups for the
  missing selection and mode-stack contracts.
- GREEN compile: managed Cargo job
  `8b0bc0286e7b44cc8f11a5fd24aa45b6` produced current
  `output-lib-zircon_editor` and `output-test-lib-zircon_editor` fingerprints
  with zero Rust errors.
- Fresh managed compile: EditorLayout15 pool job
  `edd25ded210548dbabfea57f6fcf2087` compiled the shared current source and
  produced `zircon_editor-7cbf6e3f9c684171.exe`; its full 3215-test execution
  later ended with exit 124, so no full-suite pass is claimed.
- GREEN focused behavior on that current managed binary:
  `scene::selection` ran 4 passed / 0 failed / 3211 filtered, including the
  viewport Edit/Play active-domain regression; `scene::modes` ran 4 passed /
  0 failed / 3211 filtered, including active mode plus real active-domain
  selection projection. Focused total is 8/8.
- Registry TDD RED: Editor05 managed job
  `6528f1430ac7434bbc5c3f883f09fcfe` reached current
  `output-test-lib-zircon_editor` and reported exactly three E0432 imports for
  the missing `SceneModeRegistration`, `SceneModeRegistry`, and
  `SceneModeRegistryError`, followed by `aborting due to 3 previous errors`.
- Registry GREEN: Editor05 managed job
  `3d74154fd7944459b81a9a5a8a3ca519` compiled the current test binary. Direct
  focused execution ran `scene::modes` 6 passed / 0 failed / 3211 filtered and
  `scene::selection` 4 passed / 0 failed / 3213 filtered, for 10/10 total. The
  full 3217-test process outlived the 900-second outer validation timeout and
  was identity-checked then stopped; the coordinator job reached `orphaned`
  with no live PIDs, so no full-suite pass is claimed.
- The first fresh Editor05-owned attempt, coordinator job
  `fc06cf4f79dd4c1ba9c1623819f71e3b`, exited during shared runtime
  compilation with no Rust diagnostic; its immediate retry was correctly
  rejected as `cargo_reuse_pool_busy` while the current managed compile owned
  the compatible pool. The later successful current-binary compile and 8/8
  behavior run classify that attempt as validation-environment noise, not an
  Editor05 source fix.
- Coordinator bootstrap then failed in lifecycle orphan recovery because the
  maintenance-hold integrity trigger rejected its recovery transition. The
  external blocker is recorded in
  `docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md`;
  no direct database repair or local Cargo bypass was used. Replacement-daemon
  recovery also proved that a successful resume could leave `explicitStop`
  active and reject `session.register`; that evidence is appended to the same
  Coordinator01 failure. An official hold release plus stop/start restored an
  unscoped schema 36 instance; the root-cause handoff remains open pending code
  and regression coverage.
- `cargo metadata --locked --no-deps --format-version 1`: passed after the
  `indexmap 2.14.0` package dependency was added without lockfile regeneration.
- `git diff --check` on the current Editor05 slice: passed.
- Plan output audit: passed. Failure handoff audit: 158 artifacts, 0 errors.
- A full-stack ordering regression was added after review. It asserts three-mode
  top-down input, bottom-up update/overlay, and LIFO shutdown; current-source
  recompilation is pending and no new pass count is claimed yet.

## Review

Independent review reported `P0=0 / P1=2 / P2=2`. The P2 coverage and plan
status findings are addressed by the new full-stack ordering test and the
parent plan's `in_progress` state. The selection-API P1 has been implemented in
the Editor07 exact support scope: deleting the controller methods now leaves
zero relevant source calls, non-selection commands preserve multi-selection,
and PIE restores the complete dual-domain model. Its final independent review
is `P0/P1/P2=0/0/0`; the managed current-source gate is still pending, so the failure remains open in
`docs/plans/zircon_editor/editor/07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md`.
The remaining P1 is the shared `Cargo.lock`, which contains unrelated
app/catalog/Navigation/VM changes. The Editor07 hard-cut Session owns its exact
scope in `resolving_failure`.
The mixed `Cargo.lock` lease was intentionally released so its app/catalog,
Navigation, and VM owners can land first; Editor05 must reacquire and
reattribute the clean lockfile before prepare.
The inbound command-eval and overlay-provider handoffs also stay open until
production wiring is complete and freshly validated. This slice is not
ready-to-commit.
