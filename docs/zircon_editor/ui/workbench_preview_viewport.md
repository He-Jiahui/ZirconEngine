---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_preview_viewport.zui
  - zircon_editor/assets/ui/editor/workbench/preview/animation_mannequin_viewport.svg
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace/preview_viewport.rs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_preview_viewport.zui
  - zircon_editor/assets/ui/editor/workbench/preview/animation_mannequin_viewport.svg
plan_sources:
  - user: 2026-07-13 standardize primitives first and improve preview fidelity against Unreal references
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - blend_space_preview_viewport_keeps_mannequin_subject_legible
  - cargo test -p zircon_editor --lib --locked blend_space_workspace -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked zui_asset_governance -- --test-threads=1
  - capture_blend_space_workspace_visual_artifacts
doc_type: module-detail
---

# Workbench Preview Viewport

## Purpose

`WorkbenchPreviewViewport` is the shared non-interactive image primitive used by animation workbenches when a real-time scene viewport is not yet required. It gives dense editor panels a recognizable mannequin, perspective floor, lighting panels, and axis indicator without embedding feature-local drawing commands in an L4 workspace.

## Behavior Model

The ZUI primitive owns one clipped, stretchable `Image` node. It preserves the source aspect ratio and explicitly disables focus, click, and general input participation. The Blend Space workspace supplies the surrounding panel header, Runtime Text overlays, status values, and event routes; the preview primitive does not duplicate those responsibilities.

The SVG has a fixed authored view box, but the UI frame is entirely relative: its parent receives remaining layout space and the image scales proportionally inside that frame. The mannequin and its shadow form one `preview-subject` vector group. That group is enlarged around the authored subject center, while the perspective floor and axis gizmo stay at their original scale. The transform is vector-art composition, not absolute window placement.

## Design and Rationale

The reference editor keeps the animation subject readable even inside a compact secondary preview. Zircon therefore treats subject silhouette area as a primitive-level contract instead of compensating with a larger feature-local panel or screenshot-specific coordinates. The asset remains code-native SVG so it can reuse the established icon/image pipeline, scale without bitmap blur, and stay deterministic in native screenshot tests.

No `<text>` element is embedded in the SVG. Labels such as `Perspective`, `Lit`, and preview status continue through the normal ZUI/Runtime Text projection and painter path. This preserves font discovery, shaping, clipping, and theme ownership in Runtime Text rather than creating a second text renderer inside vector assets.

## Edge Cases and Constraints

- Preserve-aspect scaling may introduce side letterboxing in unusually wide or tall hosts; the subject group must remain centered and unclipped.
- The image primitive is presentation-only. Future orbit, selection, or scene interaction belongs in a typed viewport component rather than hidden SVG hit regions.
- Feature workspaces may resize or hide the preview by responsive tier, but must not introduce pure-pixel window coordinates to enlarge the subject.
- New labels must use Runtime Text components; SVG text and hard-coded platform fonts are not accepted.

## Test Coverage

The native painter regression opens the real 1260x780 Blend Space window, finds the projected shared preview image, paints the production host frame, and counts neutral bright subject pixels inside a relative central region. The pre-change asset produced 430 pixels and failed the 520-pixel minimum; the enlarged shared subject passes that same test. This prevents a future asset edit from making the mannequin unreadable while leaving the frame itself present.

The complete Blend Space group passes `17 passed / 0 failed / 1 ignored`, and ZUI governance passes `75/75`. The ignored production-window capture passes `1/1` and writes only to `docs/tests/editor`. Manual review accepts the larger mannequin at 1260x780 without clipping; 640x520 and 900x620 remain unchanged because their responsive tiers hide this secondary preview. Current hashes are 640 `B75F08CA3555DBA99D75939E20CDE478B19096EF94D4867406DC1521701CC832`, 900 `6CF6A2524B4DB068E4333FE29180F43041EF7095A811248D3C91568CB9F0A2EF`, and 1260 `7720A63EB291FB2D1E0166EE69EAB242409D23D936705DAA3D91FC61FE16D928`.

## Open Issues

This slice improves subject legibility, not whole-window Unreal equivalence. A future typed real-time preview may replace the static SVG after the rendering and interaction contracts are ready; until then, further work should refine the shared primitive rather than add Blend Space-only painter branches.
