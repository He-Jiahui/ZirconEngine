# Assets Content Scroll and Hover Paint Transform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Assets Activity content rows scroll, clip, hover, and expose a shared scrollbar through the existing retained template-row renderer while preserving pointer/visual geometry ownership.

**Architecture:** Store transient content interaction in `HostPaneInteractionStateData`, then pass an optional node transform through the generic template paint pipeline. A pane-scoped Activity projector recognizes stable generated content ids, translates full source geometry by the stored scroll offset, intersects every content-node clip with the content panel, and marks only the matching row hovered; the native layer paints only the shared scrollbar.

**Tech Stack:** Rust 2021, Slint retained-host models, Zircon runtime text and UI design tokens, Cargo test harness, software-rendered PNG acceptance artifacts.

---

## File and ownership map

- Modify `zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/pane.rs`: own Activity/Browser content scroll and hovered-index fields plus defaults.
- Modify `zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/interaction.rs`: replace four content no-op setters with clamped state writes.
- Create `zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/interaction/tests.rs`: lock state storage, negative-scroll clamping, and `-1` hover sentinel behavior.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs`: expose the internal transform contract and transformed draw entry point.
- Create `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/transform.rs`: own `TemplateNodePaintTransform` and its transformed node/clip result.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs`: keep the current no-transform path and route the optional transform before command collection.
- Create `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/transform.rs`: prove identity behavior, geometry mutation, node-specific clipping, and suppression.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/mod.rs`: mount the focused transform tests.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes.rs`: select the Activity content projector and forward interaction state.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/content.rs`: forward pane interaction into template drawing.
- Create `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/mod.rs`: route Activity-only projection.
- Create `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/identity.rs`: parse stable folder/item row and child ids into a shared folder-first/item-second row index.
- Create `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs`: locate the content panel, translate source frames, intersect clips, apply hover, and suppress outside nodes.
- Create `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/tests.rs`: lock identity, missing-panel fallback, stale hover, clipping, and scroll projection.
- Modify `zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs`: retain non-zero geometry for rows below the initial content viewport.
- Modify `zircon_editor/src/tests/ui/assets_activity/bootstrap_assets.rs`: prove below-viewport rows remain in the source model.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar.rs`: add an Activity content scrollbar entry point.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs`: derive content viewport and extent from projected content-panel/row geometry.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/tests.rs`: lock content extent and fit/overflow behavior.
- Modify `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/content.rs`: paint the Activity content scrollbar with the stored state.
- Modify `zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs`: add a scrolled/hovered screenshot route.
- Create `docs/tests/editor/editor-window-m3-assets-drawer-scrolled-hover-900x620.png`: store visual evidence outside every Cargo target.
- Modify `docs/zircon_editor/ui/retained_host/host_contract/paint_workbench_renderer.md` and `docs/zircon_editor/ui/layouts/views/assets_activity.md`: document transform, clip, interaction, scrollbar, and full-source-geometry ownership.
- Modify `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md` and `docs/plans/zircon_editor/editor_layout/15/2026-07-09-component-standardization-from-primitives-output-records.md`: update checklist/status and one record per completed slice.
- Modify `.codex/sessions/20260710-0523-editor-layout-atomic-ui.md`: publish current milestone evidence and next step.

## Milestone 1: Real content interaction state

- Goal: stop discarding content pointer writeback at the host-state boundary.
- In-scope behaviors: Activity and Browser content scroll storage; non-negative scroll clamping; hovered-index storage including `-1`.
- Dependencies: existing `apply_asset_pointer_state_to_ui(...)` calls and `PaneSurfaceHostContext` state ownership.
- Implementation slices:
  - [x] Add failing unit-test code that creates a host pane context, writes Activity/Browser content state, and reads back exact values.
  - [x] Add four fields to `HostPaneInteractionStateData`, initialize scroll to `0.0` and hover to `-1`, and replace the four content no-op setters with direct state writes.
  - [x] Append exactly one completed output record to the owning editor-layout plan archive.
- Required state contract:

  ```rust
  pub(crate) struct HostPaneInteractionStateData {
      pub activity_asset_content_scroll_px: f32,
      pub activity_asset_content_hovered_index: i32,
      pub browser_asset_content_scroll_px: f32,
      pub browser_asset_content_hovered_index: i32,
      // existing fields remain unchanged
  }

  pub(crate) fn set_activity_asset_content_scroll_px(&self, value: f32) {
      self.state.borrow_mut().pane_interaction_state
          .activity_asset_content_scroll_px = value.max(0.0);
  }
  ```

  The test constructs `Rc<RefCell<HostContractState::new(PhysicalSize::new(640, 420))>>`, obtains `PaneSurfaceHostContext::from_state(Rc::clone(&state))`, writes positive/negative scroll and hover values, then asserts the four stored fields.
- Lightweight checks: run `cargo fmt --check -- zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/pane.rs zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/interaction.rs` if supported; otherwise run `rustfmt --check` on the touched Rust files.
- Promotion gate: no content setter remains a no-op, and the tests name all four state fields explicitly.

## Milestone 2: Generic template-node paint transform

- Goal: permit a pane-local consumer to mutate a cloned template node, narrow its clip, or suppress it without adding asset-specific logic to the generic painter.
- In-scope behaviors: identity/no-transform path; mutable node transform; clip replacement/intersection; suppression.
- Dependencies: Milestone 1 state model and current `draw_template_nodes` command pipeline.
- Implementation slices:
  - [x] Add transform tests using a small fake transform: one node translates, one receives a narrow clip, one returns suppression, and the original model remains unchanged.
  - [x] Define an internal callback/trait contract equivalent to `fn transform(&self, node: TemplatePaneNodeData, clip: FrameRect) -> Option<(TemplatePaneNodeData, FrameRect)>`.
  - [x] Add `draw_template_nodes_with_transform(...)`; keep `draw_template_nodes(...)` as the unchanged identity wrapper so all unrelated callers preserve behavior.
  - [x] Append exactly one completed output record to the owning editor-layout plan archive.
- Required transform contract:

  ```rust
  pub(in crate::ui::retained_host::host_contract) trait TemplateNodePaintTransform {
      fn transform(
          &self,
          node: TemplatePaneNodeData,
          clip: FrameRect,
      ) -> Option<(TemplatePaneNodeData, FrameRect)>;
  }

  pub(in crate::ui::retained_host::host_contract) fn draw_template_nodes_with_transform(
      frame: &mut HostRgbaFrame,
      nodes: &ModelRc<TemplatePaneNodeData>,
      origin: &FrameRect,
      clip: &FrameRect,
      text_input_focus: Option<&HostTextInputFocusData>,
      transform: Option<&dyn TemplateNodePaintTransform>,
  ) -> bool;
  ```

  `draw_template_nodes(...)` calls this function with `None`. The transformed loop clones the model row into an owned value, applies the callback once, and passes the returned node and clip to `push_template_node_commands(...)`.
- Lightweight checks: `rustfmt --check` for the pipeline owner and focused tests.
- Promotion gate: transformed tests prove suppression and node-specific clip behavior, while an existing no-transform paint test remains byte-identical.

## Milestone 3: Activity content identity, projection, and full source geometry

- Goal: make the template painter the sole viewport projection owner for all generated Activity content rows.
- In-scope behaviors: `AssetsActivityContentPanel` discovery; folder-first/item-second identity; child-node row association; scroll translation; panel clip intersection; exact one-row hover; stale-index behavior; missing-panel/unrelated-node pass-through; off-viewport suppression; below-viewport source geometry.
- Dependencies: Milestone 2 transform extension and shared `AssetContentLayoutMetrics` used by layout/pointer code.
- Implementation slices:
  - [x] Add pure identity/projector test code for folder and item row families, children, missing panel, stale hover, non-zero scroll, partial intersection, and full suppression.
  - [x] Implement stable id parsing without compatibility aliases. Count projected folder row panels to map item indices into the pointer bridge's shared row order.
  - [x] Implement frame translation and rectangle intersection using pane/body-local coordinates; set `hovered` only on the row panel with the stored shared index.
  - [x] Thread `HostPaneInteractionStateData` through `draw_pane_template_nodes(...)` and select the projector only for `pane.kind == "Assets"`.
  - [x] Remove overflow `hide_controls(...)` from `content_layout.rs`; lay every row at its metric-derived source frame and retain zero-width handling only for genuinely unavailable horizontal space.
  - [x] Add the source-geometry regression assertion: a later row must have positive width/height and a `y` beyond the content panel bottom in a short drawer.
  - [x] Append exactly one completed output record to the owning editor-layout plan archive.
- Required projector contract:

  ```rust
  pub(super) struct ActivityAssetContentProjector {
      content_panel: FrameRect,
      folder_row_count: usize,
      scroll_px: f32,
      hovered_row_index: i32,
  }

  impl TemplateNodePaintTransform for ActivityAssetContentProjector {
      fn transform(
          &self,
          mut node: TemplatePaneNodeData,
          clip: FrameRect,
      ) -> Option<(TemplatePaneNodeData, FrameRect)> {
          let Some(identity) = activity_content_identity(node.control_id.as_str()) else {
              return Some((node, clip));
          };
          node.frame.y -= self.scroll_px;
          if identity.is_row() {
              node.hovered = identity.shared_row_index(self.folder_row_count)
                  == Some(self.hovered_row_index);
          }
          let content_clip = intersect(&clip, &self.content_panel)?;
          intersect(&translated_frame(&node.frame), &content_clip)?;
          Some((node, content_clip))
      }
  }
  ```

  Unknown ids bypass this transform unchanged. Recognized content ids return `None` only when their translated frame has no intersection with the content-panel clip. `AssetsActivityContentPanel` itself is the clip source and is not translated.
- Lightweight checks: `rustfmt --check`; production-only scan for raw colors, font families, absolute window positioning, and new compatibility shims.
- Promotion gate: source rows exist beyond the viewport, only scrolled projected copies move, and no content node can paint outside the panel clip.

## Milestone 4: Shared Activity content scrollbar and interactive evidence

- Goal: expose overflow and hover through the existing Starship scrollbar and prove the normal pointer-to-paint path visually.
- In-scope behaviors: viewport from `AssetsActivityContentPanel`; extent from full source row geometry produced by shared metrics; no scrollbar when content fits/empty; stored scroll offset and active hover state; no custom RGB or duplicate row painting.
- Dependencies: Milestone 3 projector and full source rows.
- Implementation slices:
  - [x] Add scrollbar tests for empty, fitting, and overflowing Activity content plus a non-zero stored offset.
  - [x] Implement `draw_activity_asset_content_scrollbar(...)` and call it from the Assets native-content branch alongside the tree scrollbar.
  - [x] Extend the screenshot fixture to write `editor-window-m3-assets-drawer-scrolled-hover-900x620.png` after applying a real content scroll/hover state through the existing callback route.
  - [ ] Verify the captured image shows a later row, exactly one standard hovered row surface, a content scrollbar, and no content pixels over Preview.
  - [x] Append exactly one completed output record to the owning editor-layout plan archive.
- Required scrollbar contract:

  ```rust
  pub(super) fn draw_activity_asset_content_scrollbar(
      frame: &mut HostRgbaFrame,
      pane: &PaneData,
      body: &FrameRect,
      clip: &FrameRect,
      interaction: &HostPaneInteractionStateData,
  ) -> bool;
  ```

  The helper locates `AssetsActivityContentPanel`, translates its local frame by `body`, derives the full extent from the maximum bottom edge of generated content row panels relative to the panel top plus the shared first-row/padding geometry, and calls `draw_vertical_scrollbar(...)` with `interaction.activity_asset_content_scroll_px` and `interaction.activity_asset_content_hovered_index >= 0`.
- Lightweight checks: `rustfmt --check` and screenshot-path scan before the testing stage.
- Promotion gate: the artifact lives only under `docs/tests/editor`, and the shared scrollbar appears only for overflow.

## Milestone testing stage: Compile, regression, correction, and acceptance

- [x] Run formatting and structural checks:

  ```powershell
  cargo fmt --all -- --check
  git diff --check
  ```

  Expected: exit code `0`; no whitespace errors.

- [x] Compile the affected editor test target once, using an external target directory:

  ```powershell
  $env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-editor-assets-content-scroll-hover-0710'
  cargo test -p zircon_editor --lib --no-run
  ```

  Expected: the editor library test executable is produced outside the repository. If unrelated concurrent owners fail, capture exact diagnostics and use the newest focused executable only when its timestamp proves it contains this slice.

- [x] Run focused tests bottom-up from the compiled executable or Cargo filter:

  ```powershell
  cargo test -p zircon_editor --lib activity_asset_content -- --nocapture
  cargo test -p zircon_editor --lib template_node_paint_transform -- --nocapture
  cargo test -p zircon_editor --lib assets_activity -- --nocapture
  cargo test -p zircon_editor --lib retained_asset_content_scroll -- --nocapture
  ```

  Expected: all newly added state, transform, projector, geometry, scrollbar, and existing pointer tests pass.

- [x] Run the ignored scrolled/hovered capture test with `--exact --ignored --nocapture`, manually inspect the PNG, and iterate only on the lowest failing support owner.
- [x] Run the repository validator selected by `zircon-dev-validation`; record any external-owner failure without claiming full validation success.
- [x] Scan both `E:\Git\ZirconEngine\target` and the external Cargo target for matching PNG names; expected count is `0` in both.
- [x] Re-run the editor-layout plan-output audit and verify every new slice/status id appears exactly in the owning numbered plan/archive locations required by `write-plan-output-records`.
- [x] Update module docs, the editor-layout parent checklist, output archive, and active session note with exact commands, test counts, artifact hash, target scan, and any remaining external blocker.
- [x] Mark milestones complete only after the evidence above passes; leave the broader editor-layout goal active for the next bottom-up component/composite slice.

Testing-stage result (2026-07-11): scoped Rustfmt and diff checks pass; full `cargo fmt --all -- --check` remains blocked by unrelated formatting deltas in current runtime-absorption test fixtures. The current-tree official validator passes locked package build; its default-parallel test process exits 101 without a test summary after excessive thread creation. Direct single-thread execution completes the entire 2928-test binary with 2761 passed, 133 unrelated/current-worktree failures, and 34 ignored; every test owned by this plan passes in that full context. WSL `cargo check -p zircon_editor --lib --no-default-features --locked --offline --jobs 1` passes in 19m13s. The accepted native-route capture is 74069 bytes with SHA256 `45359A656E5EEBADA47685E526C790C965BD2167724C3C85053A17056DC23533`; detailed evidence and the bounded acceptance decision live in `tests/acceptance/editor-assets-content-scroll-hover-paint-transform.md`.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | Activity/Browser 内容交互状态 | completed-accepted | 2026-07-11 | setters 2/2 focused + full-context passed; WSL/package build passed |
| M2 | 通用 template-node paint transform | completed-accepted | 2026-07-11 | transform 2/2 focused + full-context passed; identity path preserved |
| M3 | Activity 内容 identity/projector/full source geometry | completed-accepted | 2026-07-11 | Activity projector/root/empty/full-source tests passed; short drawer 2/2 passed |
| M4 | Activity content shared scrollbar + visual route | completed-accepted | 2026-07-11 | native route regression 1/1; capture 1/1; screenshot SHA256 `45359A65...5333`; target scans zero |
