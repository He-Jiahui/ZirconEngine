---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/lines.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/text_markers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/lines.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/text_markers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/tests.rs
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - paint primitive image ownership scan
  - paint primitive image subtree ownership scan
  - paint primitive pixel ownership scan
  - paint primitive root subtree ownership scan
  - touched-file whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-overview
---

# Paint Primitives

`paint_primitives.rs` is the neutral retained-host software primitive surface used after the old `host_contract/painter/` namespace was removed. It now stays as a structural entry for the software primitive API: the file declares the subtree, re-exports image draw entries, and keeps stable wrapper functions for rectangle, rounded-rectangle, border, separator, and text-marker calls so existing painter, presenter, replay, and Workbench callers do not need to know the internal module layout.

`paint_primitives/clip.rs` owns the active paint-clip plus explicit clip intersection policy shared by shape and image drawing. `paint_primitives/shapes.rs` owns rectangle, rounded rectangle, border, rounded border, recording-aware quad/border emission, `PixelRect` target conversion, and the pixel-fill dispatch into the low-level raster module. `paint_primitives/lines.rs` owns separator-line clipping, separator recording, and direct span writes. `paint_primitives/text_markers.rs` owns retained-host fallback text bars and label markers. `paint_primitives/tests.rs` owns the existing clipped rectangle regression coverage that used to live inline in the root module.

`paint_primitives/image.rs` owns the image-specific draw entry and validation surface: RGBA image clipping, public draw entry variants, recording/raster orchestration, and the image regression tests. The parent module only declares the image child and re-exports the production image draw entries needed by callers.

`paint_primitives/image/recording.rs` owns image recording metadata, explicit resource-key recording, atlas recording, content-scoped fallback key hashing, and atlas payload validation. `paint_primitives/image/raster.rs` owns the pixel raster side: opaque identity-row copying, scaled RGBA sampling, identity mapping detection, and per-pixel alpha writes.

`paint_primitives/pixels.rs` owns the low-level software raster fill layer: rectangular span fill, rounded fill, rounded border fill, alpha blending, per-pixel writes, rounded-rect containment, corner-radius clamping, and frame inset helpers. The parent module calls it only after resolving clip/recording policy and `PixelRect` targets.

The 2026-06-18 image split reduced `paint_primitives.rs` from 918 lines to 510 and created `paint_primitives/image.rs` at 420 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive image ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 pixel responsibility split reduced `paint_primitives.rs` from 553 lines to 371 and created `paint_primitives/pixels.rs` at 151 lines while keeping `paint_primitives/image.rs` at 420 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive pixel ownership scan, a touched-file whitespace scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 image subtree split reduced `paint_primitives/image.rs` from 420 lines to 249 lines and created `paint_primitives/image/raster.rs` at 113 lines plus `paint_primitives/image/recording.rs` at 67 lines. The root image module now keeps the draw entries and clip/record orchestration, while raster and recording details are folder-backed children. Validation used `cargo fmt -p zircon_editor --check`, a paint primitive image subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 primitive root subtree split reduced `paint_primitives.rs` from 371 lines to 90 lines. The new child owners are `paint_primitives/shapes.rs` at 184 lines, `paint_primitives/text_markers.rs` at 56 lines, `paint_primitives/lines.rs` at 51 lines, `paint_primitives/clip.rs` at 14 lines, and `paint_primitives/tests.rs` at 74 lines. `paint_primitives/image.rs` now imports `effective_clip` from `clip.rs`, while the public primitive wrappers remain in the root module. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive root subtree ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`; the first type check exposed the moved `effective_clip` import in `image.rs`, which was corrected before the successful rerun. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.
