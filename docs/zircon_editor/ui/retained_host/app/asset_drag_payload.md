---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload.rs
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload/content.rs
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload/reference.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/press.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload.rs
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload/content.rs
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload/reference.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app asset-drag payload content/reference ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Asset Drag Payloads

`app/asset_drag_payload.rs` is the structural entry for asset drag payload construction. It keeps the existing app-level function names while separating content-list drags from reference/used-by list drags.

## Content Payloads

`app/asset_drag_payload/content.rs` owns drag payload construction for visible asset content rows. It resolves a row by asset uuid from `AssetWorkspaceSnapshot::visible_assets`, chooses the activity or browser content control id from the surface mode, and builds `UiDragPayloadKind::Asset` plus `UiDragSourceMetadata::asset(...)` using the item locator, uuid, display name, kind, and extension.

## Reference Payloads

`app/asset_drag_payload/reference.rs` owns drag payload construction for selected asset reference and used-by rows. It selects the correct reference list, rejects unknown external references, chooses the source control id for activity/browser and references/used-by, derives an extension from the reference locator, and builds the asset drag metadata for known project assets.

## Boundary Rules

- Keep `app/asset_drag_payload.rs` structural and limited to app-level re-exports.
- Keep content-list asset payload lookup and content source control ids in `app/asset_drag_payload/content.rs`.
- Keep reference/used-by payload lookup, known-project filtering, reference source control ids, and locator extension fallback in `app/asset_drag_payload/reference.rs`.
- Keep pointer press/release event gates in `app/asset_content_pointer/press.rs` and `app/asset_reference_pointer/press.rs`; those modules may request drag payloads but should not duplicate metadata construction.

## Validation Notes

The 2026-06-19 content/reference split reduced `asset_drag_payload.rs` from 113 lines to 4 lines. `asset_drag_payload/content.rs` is 34 lines and owns visible asset content payload construction. `asset_drag_payload/reference.rs` is 81 lines and owns reference/used-by payload construction, source control id mapping, known-project filtering, and locator extension fallback.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-drag payload content/reference ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). The first cargo check exposed app-sibling visibility after the payload functions moved; the exported functions are kept app-internal with `pub(in crate::ui::retained_host::app)`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
