---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/alert.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/avatar.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/badge.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/chip.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/skeleton.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/timeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/theme.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_alert.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_badge_style.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/alert.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/avatar.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/badge.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/chip.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/skeleton.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/timeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
plan_sources:
  - user: 2026-05-25 continue ZirconEditor MUI Web parity visual/feedback implementation
  - .codex/plans/ZirconEditor MUI Web Parity Plan.md
  - docs/ui-and-layout/material-ui-component-design-matrix.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_alert.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_badge_style.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/avatar.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/badge.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/skeleton.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/timeline.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives.rs zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs zircon_runtime/src/ui/tests/asset_mui_web_style.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs zircon_runtime/src/ui/tests/asset_mui_web_badge_style.rs zircon_runtime/src/ui/tests/mod.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives.rs zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/chip.rs zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs zircon_runtime/src/ui/tests/asset_mui_web_style.rs
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives/alert.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs zircon_editor/src/tests/host/retained_window/native_material_painter_alert.rs zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_display_surface_classes.rs zircon_runtime/src/ui/tests/asset_mui_web_style.rs
  - rustfmt --edition 2021 --check --config skip_children=true zircon_editor/src/ui/retained_host/host_contract/painter/material_primitives.rs zircon_editor/src/tests/host/retained_window/mod.rs
  - cargo metadata --locked --no-deps --format-version 1
  - cargo test -p zircon_editor mui_avatar --lib
  - cargo test -p zircon_editor mui_badge --lib --locked --jobs 1 --message-format short --color never (timed out during dependency compilation on 2026-05-25 without Rust diagnostics)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zirconeditor-mui-badge-check --message-format short --color never (timed out during dependency compilation on 2026-05-25 without Rust diagnostics)
  - cargo test -p zircon_runtime mui_sx_merges_as_high_priority_style_override_and_state_selectors_match --lib --locked --jobs 1 --message-format short --color never (timed out during dependency compilation on 2026-05-25 without Rust diagnostics; matching process cleaned up)
doc_type: module-detail
---

# Material Primitives

`material_primitives.rs` owns retained native painting for Material components whose web shape is not a generic filled surface. It is called from `template_nodes.rs` after feedback primitives and before MUI X primitives, so a recognized Material primitive can replace the fallback surface/text path with exact component geometry.

## Avatar Contract

The Avatar painter follows local MUI `Avatar` source for the root/fallback shape:

- Avatar roots paint as a square control instead of stretching across a wider lab row, matching MUI's fixed 40 px default box while respecting smaller retained layout bounds;
- `variant = "circular"` clips with half-size radius, `variant = "rounded"` uses the resolved theme radius or MUI's 4 px default, and `variant = "square"` removes rounding;
- color-default fallback uses theme-resolved background/foreground/border colors first and a MUI grey fallback when no image is available;
- image avatars rasterize the preview/media source and pre-mask the RGBA payload with the same rounded shape before emitting the image command, so both immediate painting and retained command-stream resources receive clipped pixels;
- fallback initials are centered in the avatar box, and the no-text/no-image path draws a compact person-glyph approximation with the resolved foreground color.

The module returns before the generic template surface path for recognized `Avatar` nodes, preventing the previous wide rounded panel fallback. Border commands are emitted after text/image content so themed Avatar roots keep their visible outline.

## Badge Contract

The Badge painter follows local MUI `Badge` source for the overlay geometry:

- Badge roots remain relative containers; the painter only draws a root panel when retained theme styles provide a background or border, then paints the authored root label if present;
- standard badges use MUI's 20 px height, 20 px minimum width, 6 px horizontal padding, pill radius, and centered count text;
- dot badges use the MUI 8 px by 8 px dot geometry with 4 px radius and no count text;
- circular overlap applies MUI's 14% anchor offset before the half-size translate, while rectangular overlap anchors to the exact root corner;
- bottom/top and left/right anchor tokens from `anchorOrigin` decide the badge center, so bottom-left circular badges can overflow the root bounds like MUI Web;
- `invisible`, hidden zero-count badges, and empty standard badges consume the node without drawing the overlay, matching the transform-to-zero behavior for retained pixels;
- MUI palette color tokens (`primary`, `secondary`, `error`, `info`, `success`, `warning`, `default`) drive the overlay tone independently of the root surface style.

`pane_component_projection` projects `badgeContent` into retained `value_text`, including the `max +` overflow form, and appends `variant`, color, overlap, anchor-origin, and invisible owner-state tokens into `component_variant`. Empty descriptor-default `badgeContent`, omitted non-dot content, and numeric `badgeContent = 0` with `showZero = false` all converge to the same invisible metadata used by the painter; string content such as `"0"` remains visible because local MUI's zero suppression uses strict numeric equality. The runtime style compiler marks `Badge`'s `badge` slot with `muiBadgeSlot` metadata and emits `MuiBadge-invisible` for the same empty/numeric-zero/no-content cases so stylesheet selectors, retained projection, and native painting agree.

## Chip Contract

The Chip painter follows local MUI `Chip` source for the compact pill geometry:

- Chip roots draw before the generic template surface path so filled/outlined backgrounds, one-pixel outlined borders, and full-height pill radii stay tied to MUI's 32 px medium and 24 px small height contract;
- `variant`, `size`, `color`, `clickable`, `deletable`, handler-valued `onDelete`/`on_delete`, `focusVisible`, and slot presence are projected into retained owner-state tokens because `TemplatePaneNodeData` does not carry raw class lists;
- filled color tokens use the local MUI palette main colors and contrast text, while outlined color tokens keep transparent fill plus a color border/foreground;
- avatar slots are drawn as the compact leading circular badge using MUI's 24 px or 18 px slot sizes and color dark fallbacks;
- icon slots draw a compact retained glyph placeholder and delete icons draw a MUI-sized X-like delete affordance at the trailing edge;
- direct `avatar`, `icon`, `label`, and `deleteIcon` slot children receive `muiChipSlot` plus `chipSlot*` metadata from the runtime style bridge, and the primitive painter also consumes `chipSlot*` tokens on their own so slot children no longer duplicate the root-owned pill contents.

The runtime style compiler keeps local `MuiChip-*` utility classes selector-visible while also adding retained-only owner tokens such as `hasAvatar`, `hasIcon`, and `hasDeleteIcon`. This mirrors the local MUI behavior where avatar/icon/deleteIcon are root-owned children with utility classes but the retained painter owns the final compact geometry.

## Alert Contract

The Alert painter follows local MUI `Alert` source for retained feedback geometry:

- Alert roots draw before Chip and generic surface fallback so severity surfaces, borders, icon/message/action layout, and close affordances are owned by one primitive route;
- standard alerts use the severity container color, filled alerts use the severity main color with contrast text, and outlined alerts use transparent fill plus a one-pixel severity border;
- severity/color resolves from retained `component_variant` tokens first, then `validation_level`/`text_tone`, and defaults to MUI's `success` severity when authors omit both `severity` and `color`;
- `hasIcon` draws a compact retained status mark, `hasAction` reserves the trailing action area, and `hasCloseAction` draws the close affordance;
- direct slot children marked with `muiAlertSlot` or `alertSlot*` are consumed by the primitive route so `icon`, `message`, `action`, `closeButton`, and `closeIcon` slots do not duplicate the root-owned Alert visual.

Runtime style application emits local MUI Alert utility classes and appends retained owner tokens for `standard`/`filled`/`outlined`, severity/color, `color*`, icon presence, action presence, and close-action presence. It also tags Alert slot children with `muiAlertSlot` plus `alertSlotIcon`, `alertSlotMessage`, `alertSlotAction`, `alertSlotCloseButton`, or `alertSlotCloseIcon`. The retained pane projection mirrors the same tokens from descriptor props so authored template nodes and runtime-compiled nodes reach the native painter through one metadata contract.

## Skeleton Contract

The Skeleton painter follows local MUI `Skeleton` source for static retained geometry and owner-state metadata:

- Skeleton roots draw text, rectangular, rounded, and circular variants before the generic surface path, using MUI's default dark placeholder tone unless a resolved theme background is provided;
- text Skeleton uses the MUI-style 60% vertical scale within the authored retained frame, while circular Skeleton centers a square inside wider layout bounds;
- rounded Skeleton uses the resolved theme radius or MUI's 4 px fallback, rectangular removes rounding, and circular uses half-size radius;
- `animation = "wave"` emits a translucent retained wave strip over the placeholder so static native pixels preserve the same visible owner state as the web utility class;
- direct or slotted Skeleton children receive `muiSkeletonChild` metadata from the runtime style bridge and are consumed by the primitive painter, matching MUI's hidden child content behavior while leaving the root size/style state available.

`pane_component_projection` now appends Skeleton shape, animation, `withChildren`, `fitContent`, and `heightAuto` owner-state tokens into `component_variant`. The runtime style compiler emits the same local MUI utility classes (`MuiSkeleton-text`, `MuiSkeleton-wave`, `MuiSkeleton-withChildren`, and related shape/height classes) and suppresses generic color/size classes that local `Skeleton.js` does not emit.

## Timeline Contract

The Timeline painter follows local MUI Lab source for the pieces that are actual primitive geometry:

- `TimelineSeparator` is a flex/layout container, so retained painting consumes it without emitting a panel even when prototype metadata carries `surface_variant`;
- `TimelineConnector` is a centered 2 px vertical strip and prefers resolved theme background/foreground/border colors before falling back to MUI grey 400;
- `TimelineDot` is a rounded square/circle with MUI's 2 px outlined border, color-token fallback, and theme-resolved background/border overrides.

This keeps the compact Lab Timeline sample from gaining an extra separator card while preserving themed Timeline root/item/content surfaces through the existing generic path. `TimelineDot` projection appends the `color` prop into `component_variant`, so retained painting can distinguish outlined secondary dots even when no explicit `text_tone` is authored.

## Divider Contract

The current module implements MUI `Divider`. Local MUI source defines Divider as border and pseudo-element geometry:

- horizontal dividers use a one-pixel bottom border;
- vertical dividers use a one-pixel right border;
- `variant = "middle"` applies 16 px horizontal insets or 8 px vertical insets;
- `variant = "inset"` applies the 72 px leading horizontal inset;
- children create before/after line segments with a wrapper gap;
- horizontal `textAlign = "left"` and `textAlign = "right"` skew the before/after segment ratio.

The retained painter represents those rules as explicit `HostPaintCommand::quad` line segments plus optional label text. This avoids drawing a generic panel behind Divider nodes and also makes legacy `surface_variant = "divider"` one-pixel Space nodes render through the same color and clipping path.

## Projection Metadata

Divider projection has to carry owner-state attributes because `TemplatePaneNodeData` does not include raw class lists. `pane_component_projection::projected_component_variant(...)` appends Divider-specific tokens for `orientation`, `flexItem`, `withChildren`, and horizontal text-align utility state. It accepts both MUI camelCase and retained snake_case names where the source assets already use both forms. `textAlign` is projected into `TemplatePaneNodeData::text_align`, with Divider defaulting to `center` when omitted.

`view_projection.rs` mirrors the same Divider role and component-variant normalization for projection paths that still resolve template metadata directly.

## Validation

The focused pixel tests cover the retained visible contract:

- `native_template_painter_draws_mui_divider_middle_horizontal_line` verifies the 16 px middle inset and one-pixel line color.
- `native_template_painter_draws_mui_divider_vertical_with_children_gap` verifies vertical middle insets and the child-wrapper line gap.
- `native_template_painter_draws_mui_timeline_dot_connector_and_separator_geometry` verifies the Timeline separator no-panel route, outlined secondary dot border, and theme-colored connector.
- `native_template_painter_draws_mui_avatar_rounded_fallback_shape` verifies the Avatar rounded color-default fallback shape, centered square sizing, and border overlay.
- `native_template_painter_clips_mui_avatar_image_to_circular_shape` verifies image avatars are masked to circular geometry before the image command is emitted.
- `native_template_painter_draws_mui_badge_standard_bottom_left_overlay` verifies Badge standard pill placement and circular bottom-left overflow.
- `native_template_painter_hides_mui_badge_invisible_dot_and_consumes_badge_slot` verifies invisible Badge nodes draw no overlay and slot nodes are consumed by the primitive route.
- `native_template_painter_draws_mui_chip_outlined_delete_icon_geometry` verifies an outlined small warning Chip uses transparent fill, one-pixel color border, and a trailing delete affordance.
- `native_template_painter_draws_mui_chip_avatar_and_consumes_chip_slot` verifies filled primary Chip root/avatar color geometry and confirms `muiChipSlot` children are consumed before generic Avatar painting.
- `native_template_painter_draws_mui_alert_outlined_icon_message_action_geometry` verifies outlined warning Alert border/icon/action geometry with transparent interior.
- `native_template_painter_draws_mui_alert_filled_close_action_and_consumes_slots` verifies filled error Alert color, close-action rendering, and `alertSlot*` child suppression.
- `native_template_painter_draws_mui_skeleton_text_wave_and_hides_children` verifies text Skeleton scale, retained wave overlay blending, and hidden child consumption.

`runtime_component_projection_preserves_mui_divider_visual_metadata` verifies the projection layer preserves the MUI owner-state tokens that the painter consumes.
`runtime_component_projection_preserves_mui_timeline_dot_color_metadata` verifies TimelineDot `color` metadata reaches the same retained token path.
`runtime_component_projection_preserves_mui_badge_overlay_metadata_and_display_value`, `runtime_component_projection_marks_mui_badge_zero_content_invisible_unless_show_zero`, `runtime_component_projection_marks_mui_badge_empty_content_invisible`, and `runtime_component_projection_keeps_mui_badge_string_zero_visible` verify Badge display-value, `max`, `showZero`, empty-content invisible, string-zero visible, explicit invisible, color, overlap, and anchor-origin metadata projection. `mui_badge_zero_content_slot_invisibility_matches_local_material_contract` verifies the runtime style compiler emits `MuiBadge-invisible` for hidden numeric zero and empty standard badges while keeping `showZero = true` and string `"0"` visible.
`runtime_component_projection_preserves_mui_chip_visual_metadata` verifies Chip `variant`, `size`, `color`, clickable/deletable/delete-icon, and focus-visible owner-state tokens reach the retained painter path. `mui_data_display_utility_classes_match_local_mui_selectors` verifies the runtime style compiler emits Chip root/slot utility classes and retained `muiChipSlot` metadata for the `deleteIcon` slot.
`runtime_component_projection_applies_mui_feedback_visual_defaults` verifies Alert variant/severity/color/icon/action/close metadata reaches the retained painter path, while `mui_feedback_utility_classes_match_alert_and_snackbar_selectors` verifies Alert root/slot utility classes and retained `muiAlertSlot` metadata from runtime style application.
`runtime_component_projection_preserves_mui_skeleton_shape_animation_and_child_tokens` verifies Skeleton shape, animation, and child-size owner-state tokens reach the retained painter path.
