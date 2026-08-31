---
title: Editor text value and media generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-text-value-media-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host text_layout/value_media projection + shared layouts preview owner`
- 24/24 Rust files source-reviewed. Every node projects flat text/value/media capabilities; collection
  summaries recursively materialize descendants; media-less nodes enter the preview cache mutex.
  M1 makes array/table summaries O(1) and media-less preview lock/hash/path work zero (focused
  contract GREEN 2/2; owned contracts GREEN 49/49). M0/M2-M5 typed-generation/invalidation/profile/
  power/render acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP retained content. Record raw lookups, owned bytes, collection descendant visits,
preview locks/decodes, text layout rebuilds, CPU/allocation/RSS/latency/power and GPU media parity.

## `docs/plans/performance/01/2026-07-17-editor-visual-assets-static-review.md`

Retain ownership of painter resource candidates/pixel caches. Consume the typed resource handle from
this plan; do not create a second image cache in content projection.

## `docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`

Own compiled optional `TextSpec`, `ValueSpec`, `MediaSpec` and component-specific layout metrics.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own compact retained text/value patches and shared resource handles without stable raw TOML
reprojection or decoded-image ownership per generic node.

## `docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`

Own resource identity/generation resolution shared with visible asset-content preview requests.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed text measure/paint/accessibility and image resource invalidation receipts.

## `docs/plans/zircon_runtime/render/13-texture-pipeline.md`

Own bounded decode/raster/upload resources and RenderDoc media upload/draw/pixel parity.

## Acceptance handoff

The handoff requires 24/24 post-change fingerprints, managed behavior tests, the node/content/
collection/media/scale matrix, current-source WPR/power artifacts on D/E/F, visual/accessibility
parity, RenderDoc media parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.
