---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_props.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_props/cargo.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_props/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_props/property.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_props/rails.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene.rs
source_plan:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
validation:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - template viewport scene prop root ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Template Viewport Scene Props

`template_viewport_scene_props.rs` is the retained-host viewport prop detail entry for the 08 M3.S2 editor UI hard-cutover work. It is now a narrow 8-line dispatch/re-export module so viewport scene prop details remain folder-backed instead of growing another all-in-one painter file.

## Ownership

- `template_viewport_scene_props.rs` owns only child-module declaration and the paint functions consumed by `template_viewport_scene.rs`.
- `template_viewport_scene_props/cargo.rs` owns cargo highlight strips, inset shadow quads, cargo inner-frame borders, divider lines, and horizontal midline details.
- `template_viewport_scene_props/property.rs` owns prop top/body base-surface paint plus top highlight, side shadow, edge light, and bottom shadow overlays.
- `template_viewport_scene_props/rails.rs` owns rack vertical/horizontal detail lines and handrail base, bottom rail, and post details.
- `template_viewport_scene_props/primitives.rs` owns shared prop helper geometry: corner radius derivation and clipped one-pixel rect-line command emission.

## Boundary

These modules are still retained-host software-paint details. They do not own viewport scene-kind classification, pixel alignment, command sorting/replay, GPU presentation, or runtime UI extraction. `template_viewport_scene.rs` remains the scene-kind router; `template_viewport_scene_structure.rs` remains the shared base-surface owner; `render_commands.rs` remains the command DTO/replay owner.

## 2026-06-18 Split

The 2026-06-18 slice reduced `template_viewport_scene_props.rs` from 370 lines to an 8-line structural entry. New child modules have these line counts after formatting: `cargo.rs` 151, `property.rs` 108, `rails.rs` 98, and `primitives.rs` 34.

Validation remained feature-first per the user's request. `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, the root ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed; the first check exposed import-depth and re-export visibility drift from the folder move, which was fixed before the passing rerun. Full Cargo test matrix remains deferred to the milestone testing stage.
