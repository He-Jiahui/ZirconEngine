# UI Render R9 Offscreen Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first interface-owned R9 parity/golden harness layer so shared UI render fixtures can prove semantic `UiRendererParitySnapshot` equality before future runtime/editor pixel comparison.

**Architecture:** Keep the first implementation in `zircon_runtime_interface::ui::surface::render` and its focused tests. Add reusable test fixture/diff helpers only inside the interface contract test lane, then update docs to describe later runtime/editor adapter and pixel-golden stages without editing those backends.

**Tech Stack:** Rust, serde DTOs, `zircon_runtime_interface` render contracts, focused Cargo validation, Zircon milestone-first workflow.

---

## Source Boundaries

Allowed in the first R9 implementation slice:

- Modify: `zircon_runtime_interface/src/tests/render_contracts.rs`
- Modify if needed: `zircon_runtime_interface/src/ui/surface/render/parity.rs`
- Modify: `docs/zircon_runtime_interface/ui/surface/render.md`
- Modify: `docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md`
- Modify: `docs/superpowers/plans/2026-05-06-ui-render-r9-offscreen-parity.md`
- Modify: `.codex/sessions/20260506-2036-ui-render-next-slice-design.md`

Avoid in the first R9 implementation slice unless a fresh coordination scan and a new approved plan say otherwise:

- `zircon_runtime/src/graphics/scene/scene_renderer/ui/**`
- `zircon_editor/src/ui/slint_host/host_contract/painter/**`
- `zircon_runtime/src/ui/text/**`
- `zircon_runtime/src/ui/layout/pass/**`
- `zircon_runtime_interface/src/ui/layout/**`
- `zircon_runtime/src/ui/surface/input/**`
- Material `.ui.toml` assets and active editor debug-reflector files

## Milestone 1: Interface Parity Fixture Harness

- [x] Add focused parity fixture helpers in `zircon_runtime_interface/src/tests/render_contracts.rs`.

  Add helpers near the existing `solid_command(...)` helper so all fixture construction stays local to the test module. Keep them private to tests; do not add production test-only APIs.

  The helper set should include:

  ```rust
  fn parity_extract_fixture(name: &str, commands: Vec<UiRenderCommand>) -> UiRenderExtract {
      UiRenderExtract {
          tree_id: UiTreeId::new(format!("ui.render.parity.{}", name)),
          list: UiRenderList { commands },
      }
  }
  ```

  ```rust
  fn image_command(node_id: u64, x: f32, resource_id: &str) -> UiRenderCommand {
      UiRenderCommand {
          node_id: UiNodeId::new(node_id),
          kind: UiRenderCommandKind::Image,
          frame: UiFrame::new(x, 8.0, 32.0, 24.0),
          clip_frame: Some(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
          z_index: 4,
          style: UiResolvedStyle::default(),
          text_layout: None,
          text: None,
          image: Some(UiVisualAssetRef::Icon(resource_id.to_string())),
          opacity: 1.0,
      }
  }
  ```

  ```rust
  fn text_command(node_id: u64, x: f32, mode: UiTextRenderMode) -> UiRenderCommand {
      UiRenderCommand {
          node_id: UiNodeId::new(node_id),
          kind: UiRenderCommandKind::Text,
          frame: UiFrame::new(x, 8.0, 56.0, 24.0),
          clip_frame: None,
          z_index: 5,
          style: UiResolvedStyle {
              foreground_color: Some("#ffffff".to_string()),
              text_render_mode: mode,
              ..UiResolvedStyle::default()
          },
          text_layout: None,
          text: Some("Parity".to_string()),
          image: None,
          opacity: 1.0,
      }
  }
  ```

  ```rust
  fn parity_row_summary(snapshot: &UiRendererParitySnapshot) -> Vec<(u64, UiRendererParityPayloadKind, Option<UiTextRenderMode>)> {
      snapshot
          .paint_order
          .iter()
          .map(|row| (row.node_id.value(), row.payload_kind, row.text_render_mode))
          .collect()
  }
  ```

  If `UiNodeId` does not expose `value()`, use the existing ID accessor in the crate. If no accessor exists, compare `UiNodeId::new(...)` directly in each assertion instead of adding an accessor.

- [x] Add a Tier 1/Tier 2/Tier 3 semantic parity test in `render_contracts.rs`.

  Add a test named `ui_renderer_parity_fixture_tiers_compare_semantic_rows_before_pixels`.

  The test should:

  - Build a fixture with `solid_command(...)`, `image_command(...)`, and `text_command(...)`.
  - Derive `UiRendererParitySnapshot::from_render_extract(...)` twice from the same extract.
  - Assert full snapshot equality.
  - Assert paint rows preserve node order, payload kind, clip/resource/text identities, and stats.
  - Assert the serialized snapshot contains `paint_order` and `batch_order`.

  Expected core assertions:

  ```rust
  let extract = parity_extract_fixture(
      "tiers",
      vec![
          solid_command(101, 0.0, 0.0),
          image_command(102, 56.0, "toolbar.open"),
          text_command(103, 96.0, UiTextRenderMode::Sdf),
      ],
  );
  let expected = UiRendererParitySnapshot::from_render_extract(&extract);
  let actual = UiRendererParitySnapshot::from_render_extract(&extract);

  assert_eq!(actual, expected);
  assert_eq!(actual.paint_order.len(), 3);
  assert_eq!(actual.batch_order.len(), 3);
  assert_eq!(actual.paint_order[0].node_id, UiNodeId::new(101));
  assert_eq!(actual.paint_order[0].payload_kind, UiRendererParityPayloadKind::Brush);
  assert!(actual.paint_order[0].clip_key.is_some());
  assert_eq!(actual.paint_order[1].resource.as_ref().unwrap().kind, UiRenderResourceKind::Icon);
  assert_eq!(actual.paint_order[2].text_render_mode, Some(UiTextRenderMode::Sdf));
  assert_eq!(actual.stats.paint_element_count, 3);
  assert_eq!(actual.stats.resource_bound_paint_count, 1);
  assert_eq!(actual.stats.text_paint_count, 1);
  let json = serde_json::to_string(&actual).unwrap();
  assert!(json.contains("paint_order"));
  assert!(json.contains("batch_order"));
  ```

- [x] Add a material/resource revision parity test in `render_contracts.rs`.

  Add a test named `ui_renderer_parity_fixture_preserves_material_resource_identity`.

  The test should:

  - Start from `solid_command(...).to_paint_element(...)` if direct paint elements are clearer than legacy command construction.
  - Replace the payload with a `UiBrushPayload::material("material/ui/parity-panel")` brush using variant `hdr`, revision `17`, fallback color `#102030`, and a fallback texture resource with revision `3`.
  - Build a `UiBatchPlan` and `UiRendererParitySnapshot::from_paint_elements_batches(...)`.
  - Assert the paint and batch rows carry `UiRenderResourceKind::Material`, id `material/ui/parity-panel#hdr`, revision `Some(17)`, and the fallback texture.

  Expected core assertions:

  ```rust
  let fallback = UiRenderResourceKey::new(UiRenderResourceKind::Texture, "ui/fallback-material")
      .with_revision(3);
  let mut element = solid_command(111, 0.0, 0.0).to_paint_element(0);
  element.payload = UiPaintPayload::Brush {
      brushes: crate::ui::surface::UiBrushSet {
          fill: Some(
              UiBrushPayload::material("material/ui/parity-panel")
                  .with_material_variant("hdr")
                  .with_material_revision(17)
                  .with_fallback_resource(fallback.clone())
                  .with_fallback_color("#102030"),
          ),
          border: None,
      },
  };
  let elements = vec![element];
  let plan = UiBatchPlan::from_paint_elements(&elements);
  let snapshot = UiRendererParitySnapshot::from_paint_elements_batches(
      UiTreeId::new("ui.render.parity.material"),
      &elements,
      &plan,
  );
  let resource = snapshot.paint_order[0].resource.as_ref().unwrap();

  assert_eq!(resource.kind, UiRenderResourceKind::Material);
  assert_eq!(resource.id, "material/ui/parity-panel#hdr");
  assert_eq!(resource.revision, Some(17));
  assert_eq!(resource.fallback.as_deref(), Some(&fallback));
  assert_eq!(snapshot.batch_order[0].resource.as_ref(), Some(resource));
  ```

- [x] Add a semantic-drift diff helper test in `render_contracts.rs` only if the test file remains readable.

  Prefer not to add a production diff type in this milestone. If equality assertions become too opaque, add a private test helper:

  ```rust
  fn parity_snapshot_row_labels(snapshot: &UiRendererParitySnapshot) -> Vec<String> {
      snapshot
          .paint_order
          .iter()
          .map(|row| {
              format!(
                  "paint={} node={:?} payload={:?} batch={:?} resource={:?} text={:?}",
                  row.paint_index,
                  row.node_id,
                  row.payload_kind,
                  row.batch_index,
                  row.resource.as_ref().map(|resource| (&resource.kind, &resource.id, resource.revision)),
                  row.text_render_mode,
              )
          })
          .collect()
  }
  ```

  Add a small assertion in the tier test that the labels mention `payload=Text` or the relevant debug format for `UiRendererParityPayloadKind::Text`. Do not add this helper if direct field assertions are clearer.

  2026-05-06 result: skipped by design because direct row assertions stayed clearer and no production/test diff helper was needed.

## Milestone 2: R9 Documentation Updates

- [x] Update `docs/zircon_runtime_interface/ui/surface/render.md`.

  Add a short paragraph after the existing `parity.rs` paragraph explaining that R9 uses the parity packet as the first gate for future golden tests. Include these points:

  - Semantic parity comes before pixel comparison.
  - Backend adapters must emit comparable paint and batch rows.
  - Pixel differences are only meaningful after paint order, batch order, clip/resource/text identities, and stats match.

  Also update the `tests:` frontmatter if new validation commands are run.

- [x] Update `docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md`.

  Add an R9 status paragraph near the R1-R8 contract status section. The paragraph should state:

  - R9 is a parity/golden harness step, not renderer cleanup completion.
  - The first slice remains interface-owned and uses hand-authored render extracts.
  - Runtime WGPU and editor painter adapter output are later milestones.
  - The acceptance order is semantic `UiRendererParitySnapshot` equality first, optional backend pixel output second.

  Update the test/evidence list only with commands that were actually run.

- [x] Update `.codex/sessions/20260506-2036-ui-render-next-slice-design.md`.

  Record the approved R9 direction, files touched, validation commands, and blockers. Keep the note active while implementing. Archive or delete it at closeout according to coordination rules.

## Testing Stage

- [x] Before Cargo validation, check whether the chosen target drive has at least 50 GB free.

  Recommended target directory for this isolated slice:

  ```powershell
  D:\cargo-targets\zircon-render-r9-offscreen-parity
  ```

  If the target drive has `<= 50 GB` free, run:

  ```powershell
  cargo clean --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity"
  ```

  2026-05-06 result: D: had 125,702,815,744 bytes free, above the 50 GB cleanup threshold, so no target clean was needed.

- [x] Run rustfmt on touched Rust files.

  ```powershell
  rustfmt --edition 2021 --check "zircon_runtime_interface/src/tests/render_contracts.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs"
  ```

  If `parity.rs` was not modified, omit it from the command.

  Expected: no output and exit code 0.

  2026-05-06 result: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/tests/render_contracts.rs"` passed with no output.

- [x] Run scoped interface check.

  ```powershell
  cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never
  ```

  Expected: `Finished` with no errors. Existing warning noise is acceptable only if unrelated and already known.

  2026-05-06 result: `cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never` finished successfully.

- [x] Run focused render contract tests.

  ```powershell
  cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never -- --nocapture
  ```

  Expected: all `render_contracts` tests pass. The exact test count must be recorded in docs and the session note.

  2026-05-06 result: `cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never -- --nocapture` passed with `23 passed; 0 failed; 32 filtered out`.

- [x] If a test fails, diagnose from the lowest shared layer first.

  Use this order:

  1. Fixture construction in `render_contracts.rs`.
  2. `UiRenderCommand::to_paint_elements(...)` derivation.
  3. `UiBatchPlan::from_paint_elements(...)` batching.
  4. `UiRendererParitySnapshot::from_paint_elements_batches(...)` row export.
  5. Serde/default behavior.

  Do not fix failures by editing runtime WGPU or editor painter code in this milestone.

  2026-05-06 result: no test failure occurred.

## Acceptance Evidence

- [x] Record exact rustfmt/check/test commands and results in `docs/zircon_runtime_interface/ui/surface/render.md`.
- [x] Record exact rustfmt/check/test commands and results in `docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md`.
- [x] Record exact rustfmt/check/test commands and results in `.codex/sessions/20260506-2036-ui-render-next-slice-design.md`.
- [x] State explicitly that R9 did not validate runtime WGPU, editor painter, offscreen pixels, or workspace-wide CI.

## Closeout

- [ ] Run `git status --short` and report only the files touched by this R9 slice.
- [ ] Delete the active session note if no handoff is needed, or move it to `.codex/sessions/archive/` with `status: completed` and a short completion summary if later sessions need the R9 evidence.
- [ ] Do not commit unless the user explicitly asks for a commit.

## Self-Review

- Spec coverage: This plan implements the approved docs-first R9 design with interface parity fixtures, semantic-first diff expectations, documentation updates, and scoped validation. Runtime/editor adapters and pixel goldens are explicitly later milestones.
- Placeholder scan: No placeholder tasks are present; every implementation and validation step names concrete files and commands.
- Type consistency: The plan uses existing R8 types: `UiRendererParitySnapshot`, `UiRendererParityPayloadKind`, `UiRenderExtract`, `UiRenderList`, `UiBatchPlan`, `UiBrushPayload`, `UiPaintPayload`, `UiRenderResourceKey`, `UiRenderResourceKind`, and `UiTextRenderMode`.
