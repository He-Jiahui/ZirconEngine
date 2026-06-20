---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/lines.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/fill.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/span.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes/borders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes/rects.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/lines.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/border.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/fill.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/span.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes/borders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/shapes/rects.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/text_markers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/tests.rs
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - paint primitive image ownership scan
  - paint primitive image subtree ownership scan
  - paint primitive image test-owner ownership scan
  - paint primitive shape rect/border ownership scan
  - paint primitive pixel span/fill/border/geometry/test ownership scan
  - paint primitive pixel ownership scan
  - paint primitive root subtree ownership scan
  - paint primitive root re-export ownership scan
  - touched-file whitespace scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-overview
---

# Paint Primitives

`paint_primitives.rs` is the neutral retained-host software primitive surface used after the old `host_contract/painter/` namespace was removed. It now stays as a structural entry for the software primitive API: the file declares the subtree and re-exports image, shape, line, and text-marker draw entries so existing painter, presenter, replay, and Workbench callers do not need to know the internal module layout.

`paint_primitives/clip.rs` owns the active paint-clip plus explicit clip intersection policy shared by shape and image drawing. `paint_primitives/shapes.rs` is now a structural shape entry. `paint_primitives/shapes/rects.rs` owns rectangle, rounded rectangle, recording-aware quad emission, `PixelRect` target conversion, and direct rectangle fill dispatch. `paint_primitives/shapes/borders.rs` owns rectangular border segment emission, rounded border recording, and rounded border pixel fill dispatch. `paint_primitives/lines.rs` owns separator-line clipping, separator recording, and direct span writes. `paint_primitives/text_markers.rs` owns retained-host fallback text bars and label markers. `paint_primitives/tests.rs` owns the existing clipped rectangle regression coverage that used to live inline in the root module.

`paint_primitives/image.rs` owns the image-specific draw entry and validation surface: RGBA image clipping, public draw entry variants, and recording/raster orchestration. `paint_primitives/image/tests.rs` owns the image regression tests that used to live inline in the image production module. The parent module only declares the image child and re-exports the production image draw entries needed by callers.

`paint_primitives/image/recording.rs` owns image recording metadata, explicit resource-key recording, atlas recording, content-scoped fallback key hashing, and atlas payload validation. `paint_primitives/image/raster.rs` owns the pixel raster side: opaque identity-row copying, scaled RGBA sampling, identity mapping detection, and per-pixel alpha writes.

`paint_primitives/pixels.rs` is now a structural pixel entry for the low-level software raster fill layer. `paint_primitives/pixels/span.rs` owns rectangular span fill, per-pixel writes, and alpha blending; `paint_primitives/pixels/fill.rs` owns direct rectangular and rounded-rect fill loops; `paint_primitives/pixels/border.rs` owns rounded-border pixel fill; `paint_primitives/pixels/geometry.rs` owns rounded-rect containment, corner-radius clamping, ordered float clamping, and frame inset helpers; `paint_primitives/pixels/tests.rs` owns the pixel geometry regression that used to live inline in the production module. The parent shape modules call these owners only after resolving clip/recording policy and `PixelRect` targets.

The 2026-06-18 image split reduced `paint_primitives.rs` from 918 lines to 510 and created `paint_primitives/image.rs` at 420 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive image ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 pixel responsibility split reduced `paint_primitives.rs` from 553 lines to 371 and created `paint_primitives/pixels.rs` at 151 lines while keeping `paint_primitives/image.rs` at 420 lines. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive pixel ownership scan, a touched-file whitespace scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 image subtree split reduced `paint_primitives/image.rs` from 420 lines to 249 lines and created `paint_primitives/image/raster.rs` at 113 lines plus `paint_primitives/image/recording.rs` at 67 lines. The root image module now keeps the draw entries and clip/record orchestration, while raster and recording details are folder-backed children. Validation used `cargo fmt -p zircon_editor --check`, a paint primitive image subtree ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-18 primitive root subtree split reduced `paint_primitives.rs` from 371 lines to 90 lines. The new child owners are `paint_primitives/shapes.rs` at 184 lines, `paint_primitives/text_markers.rs` at 56 lines, `paint_primitives/lines.rs` at 51 lines, `paint_primitives/clip.rs` at 14 lines, and `paint_primitives/tests.rs` at 74 lines. `paint_primitives/image.rs` now imports `effective_clip` from `clip.rs`, while the public primitive wrappers remain in the root module. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive root subtree ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`; the first type check exposed the moved `effective_clip` import in `image.rs`, which was corrected before the successful rerun. Full Cargo test expansion remains deferred to the milestone testing stage per the current implementation cadence.

The 2026-06-20 root re-export split reduced `paint_primitives.rs` from 90 lines to a 19-line structural module entry. Shape, separator-line, and fallback text-marker draw functions are now directly visible from their child owner modules with the same `host_contract`-scoped API surface; the root file only re-exports them alongside image draw entries. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `FrameRect`/`HostRgbaFrame` imports and draw wrapper bodies no longer live in `paint_primitives.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.

The 2026-06-21 image test-owner split reduced `paint_primitives/image.rs` from 249 lines to 115 lines and moved image replay/recording regressions into `paint_primitives/image/tests.rs` at 133 lines. The image production module now keeps only the draw-entry validation and record/raster orchestration, while test-only `HostRecordedPaintKind` assertions live in the child test owner. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive image test-owner ownership scan, scoped whitespace scan, and scoped `git diff --check`; package-level `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was attempted and timed out after 300 seconds before producing actionable editor diagnostics. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-21 shape rect/border split reduced `paint_primitives/shapes.rs` from 202 lines to a 9-line structural entry. `paint_primitives/shapes/rects.rs` is 89 lines and owns rect/rounded-rect draw, quad recording, target frame conversion, and fill dispatch; `paint_primitives/shapes/borders.rs` is 114 lines and owns rectangular border edge emission plus rounded-border recording and fill dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive shape rect/border ownership scan, scoped whitespace scan, and scoped `git diff --check`. Package-level Cargo check remains covered by the earlier 2026-06-21 timeout before actionable editor diagnostics, and full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-21 pixel span/fill/border/geometry/test split reduced `paint_primitives/pixels.rs` from 180 lines to a 14-line structural entry. `paint_primitives/pixels/span.rs` is 53 lines and owns span fill plus alpha blending, `paint_primitives/pixels/fill.rs` is 35 lines and owns rectangular/rounded fill loops, `paint_primitives/pixels/border.rs` is 28 lines and owns rounded-border fill, `paint_primitives/pixels/geometry.rs` is 63 lines and owns containment/clamping/inset helpers, and `paint_primitives/pixels/tests.rs` is 6 lines and owns the rounded-rect clamp regression. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint primitive pixel span/fill/border/geometry/test ownership scan, scoped whitespace scan, and scoped `git diff --check`. Package-level Cargo check remains covered by the earlier 2026-06-21 timeout before actionable editor diagnostics, and full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
