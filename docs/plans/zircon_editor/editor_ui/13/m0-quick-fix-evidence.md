# M0 Quick-Fix Evidence

- Gate: design-ready
- Owner session: current session
- Changed scope: 12 source files across `zircon_runtime`, `zircon_runtime_interface`
- Manifest: pending (to be written at milestone batch gate)
- Commands actually run: `git diff --check` on all changed files — exit 0
- Result summary: 10 BUG comments removed, 10 fixes applied; 2 bugs deferred (I5, L6)
- Repaired failures: see table below
- Deferred external checks: `cargo check -p zircon_runtime --lib --locked` (milestone batch gate)

## Fixed items

| ID | Bug tag | File | Change |
| --- | --- | --- | --- |
| R8 | CR-UI-LAYOUTV2-0001 | `ui/layout/pass/clip.rs` | `clip_to_bounds` with no ancestor intersection now returns `UiFrame::ZERO` instead of `Some(frame)`, preventing invisible-area reopening |
| I4 | CR-UI-COMP-0001 | `ui/platform_input/winit_translation.rs` | Wheel event uses `context.last_cursor_position.unwrap_or_default()` instead of hard-coded origin; `UiWindowInputContext` gets new `last_cursor_position: Option<UiPoint>` field and `with_last_cursor_position()` builder (in `zircon_runtime_interface`) |
| I6 | CR-R02-runtime_wave5_surface_navigation_mutation-0001 | `ui/surface/navigation_index/semantics.rs` | BUG comment removed; the OR-return logic was correct and the comment's claim was wrong |
| R3/svg | CR-UI-LAYOUTV2-0005 | `ui/icon_atlas/svg.rs` | `attribute()` now uses word-boundary matching to prevent `data-width` shadowing `width` |
| I2 | CR-UI-TEMPLATE-CONTRACT-0001 | `ui/template/asset/binding/validation.rs` | `validate_binding` now rejects a binding that declares both a top-level `route` and a nested `action.action` with `InvalidTarget` diagnostic |
| L5/grid | CR-UI-TEMPLATE-BUILD-0001 | `ui/template/build/slot_contract.rs` | MUI Grid `offset` and `size` clamped to `MUI_GRID_MAX_COLUMNS = 12` before entering slot placement |
| L5/col | CR-UI-LAYOUTV2-0004 | `ui/layout/pass/responsive_mui.rs` | Responsive column counts capped at `MAX_RESPONSIVE_COLUMNS = 1024` |
| L6/i32 | CR-UI-LAYOUTV2-0002 | `ui/v2/surface_tree/parse.rs` | `parse_i32` clamps out-of-range values to `i32::MIN`/`i32::MAX` instead of silently truncating via `as i32` |
| I7a | CR-R02-runtime_wave5_component_state_contracts-0001 | `ui/component/state_reducer/text_input.rs` | `set_validation_state` skips overwriting `validation_message` when transitioning to normal and an existing non-empty message is present |
| I7b | CR-R02-runtime_wave5_component_state_contracts-0003 | `ui/component/state_reducer/notification_center.rs` | Nested notification index tracks `entries.len()` (flat count so far) instead of per-array offset, fixing `[[a,b],c]` producing indices `0,1,1` instead of `0,1,2` |
| I7c | CR-UI-COMP-0005 | `ui/component/state_reducer/windowing.rs` | `requested_start` clamped to `max(0)` after `saturating_sub(overscan)` |
| I7d | CR-UI-COMP-0006 | `ui/component/state_reducer/world.rs` | `apply_world_transform` now validates all three vectors for finiteness before any state write, not only scale positivity |
| R8/atlas | CR-UI-LAYOUTV2-0003 | `ui/icon_atlas/atlas.rs` | `cell_size` clamped to `max_side_px` before layout, preventing UV > 1.0 when a single icon exceeds the atlas maximum |

## Deferred items

| ID | Reason |
| --- | --- |
| I5 (CR-UI-COMP-0002) | `UiControlResponse` is `#[derive(Serialize, Deserialize)]`; adding a non-serializable `crossbeam::Receiver` field requires an API redesign that is larger than an M0 quick-fix slice. The BUG comment is updated to a tracked comment. |
| L6/row-reverse (CR-UI-LAYOUTV2-0007) | `UiContainerKind` has no `HorizontalBoxReverse` variant. Adding it requires changes across `arrange.rs`, `compute.rs`, and the taffy bridge. Tracked in the comment; mapped to M3 `typed layout` slice. |

## Notes

- `UiWindowInputContext` now has `#[serde(default)]` on the new field, so existing serialized contexts without `last_cursor_position` deserialize to `None` without error.
- Call sites that build `UiWindowInputContext` should call `.with_last_cursor_position(pt)` from their `PointerMoved` handler to populate the field. That wiring belongs to the M1 window-lifecycle slice.
- No Cargo compile or test was run in this slice (policy: slice-level = format+diff-check only; batch Cargo deferred to milestone gate).
