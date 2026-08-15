---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: resolved-glyph-artifact-ui-owner-reverse-dependency
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_runtime/text/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/glyph_artifact.rs
  - zircon_runtime/src/text/ui_style.rs
  - zircon_runtime/src/graphics/text_transport/mod.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/rich_text/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
tests:
  - cargo test -p zircon_runtime --lib --locked visual_glyph_artifact_
  - cargo test -p zircon_runtime --lib --locked render_prepare_rebuilds_missing_or_stale_plain_glyph_artifacts
  - cargo test -p zircon_runtime --lib --locked glyph_artifact_batches_
  - cargo test -p zircon_runtime --lib --locked text_style_from_ui_resolved_style_preserves_layout_fields
  - cargo test -p zircon_runtime --lib --locked
  - python tools/runtime_domain_dependency_audit.py --repo-root .
---

# Text02: resolved glyph artifact has a concrete UI owner

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M3/M4 production dependency re-audit after the composite-font owner correction
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 交接原因：the artifact contains shaped glyph identity, font-generation lineage, and canonical
  re-shaping behavior. UI produces it and graphics consumes it, but neither UI nor graphics is the
  shared implementation owner. Text02 owns shaping projection and visual-order glyph semantics.

## 失败现象与复现证据

The 2026-08-02 production-only dependency audit reported **2,584 references / 79 edges** and a
regressed `graphics -> ui = 3` edge. The three production imports are:

- `graphics/scene/scene_renderer/ui/render.rs` imports `ResolvedTextGlyphArtifact` from
  `crate::ui::text`;
- `graphics/scene/scene_renderer/ui/render/resolved_layout.rs` imports the UI-owned artifact
  resolver;
- `graphics/scene/scene_renderer/ui/render/text_advances.rs` imports the UI-owned artifact rebuild
  operation.

The failure-time concrete implementation was the untracked
`zircon_runtime/src/ui/text/glyph_artifact.rs`. It stored `TextGlyph`, font database generation,
resolved text layout lines, and the style/writing-mode inputs needed to rebuild a line through
`SharedTextLayoutSession`. UI layout/rich-text code registers the artifact into the existing
type-erased `UiRichTextArtifactHandle`; graphics downcasts it and retains the artifact across font
generation changes. The failure-time owner called UI-local `super::text_style` from
`ui/text/adapter.rs`; a physical move that merely imports that adapter would replace the existing
`graphics -> ui` edge with an equally invalid `text -> ui` edge. Graphics tests also construct the
concrete UI-owned type directly.

The conversion used by that adapter was also physically misplaced:
`impl From<&UiResolvedStyle> for TextStyle` lived in
`graphics/text_transport/mod.rs`. Rust coherence makes it visible across the monolith, but after
crate extraction `zr_text` cannot depend on a graphics-owned impl. Moving the artifact without this
impl would either fail the new crate build or invite a duplicate/manual conversion.

At handoff creation, the UI, text, and graphics consumers contained large foreign dirty changes,
and both the artifact owner file and the graphics resolved-layout file were untracked. Frameworks05
therefore published the fixing-plan contract without absorbing those source blobs.

## 最低共享层根因

The type-erased handle is already the correct neutral transport, but the payload implementation was
placed below its UI producer. That placement makes the graphics consumer depend on a neighboring
business implementation and causes UI to own font-generation recovery that is actually part of the
text shaping lineage.

The lowest shared implementation owner is folder-backed `zircon_runtime::text`, because both UI and
graphics already consume text services and the artifact rebuild calls concrete text session/font
state. `core::framework::text` is not the owner: it is reserved for neutral contracts/DTOs and must
not absorb a concrete runtime cache/downcast implementation. The `UiResolvedStyle -> TextStyle`
conversion is owned by `zr_text` under the orphan/coherence rules because `TextStyle` is the local
type. UI's adapter and graphics transport are consumers, not the shared implementation owner.

## 架构修复验收

- Move the implementation physically from `ui/text/glyph_artifact.rs` to a focused
  `text/glyph_artifact.rs` owner and declare it from `text/mod.rs`.
- Move the single `From<&UiResolvedStyle> for TextStyle` impl from `graphics/text_transport/mod.rs`
  into a focused text-owned conversion module in the same hard cut. Preserve every mapped layout
  field and add `text_style_from_ui_resolved_style_preserves_layout_fields`; the unique impl count
  must remain exactly one. The artifact owner must use this text-owned conversion, contain no
  `crate::ui` import, and never call the UI-local adapter.
- Hard-cut UI layout/rich-text producers and graphics render consumers/tests to the direct text
  owner in the same change. Delete the UI module declaration and all UI re-exports; do not leave a
  forwarding import path.
- Preserve the existing `UiRichTextArtifactHandle` transport, glyph/source ranges, visual-order
  projection, font generation, ellipsis fallback, and whole-line rebuild behavior. Do not clone the
  payload into a second graphics DTO or re-shape per visual run.
- Keep implementation visibility crate-private. No new public engine API or cross-crate backend
  contract is required.
- Fresh production audit must report both `graphics -> ui = 0` and `text -> ui = 0`; full source
  scan must find no `crate::ui` reference in `text/glyph_artifact.rs` and no
  `crate::ui::text::{ResolvedTextGlyphArtifact, resolve_resolved_text_glyph_artifact,
  rebuild_resolved_text_glyph_artifact_line}` consumer outside the deleted owner.
- Run the three existing exact non-zero filters from frontmatter for artifact owner, UI rebuild, and
  renderer batch behavior plus the required new style-conversion regression, then run the unfiltered
  package Runtime lib gate on one immutable current-source snapshot. Record selected/passed/failed
  counts for every filter; a missing new regression or zero-test exit is not GREEN.

## 禁止临时方案

- Do not re-export the new text owner from `ui::text` or retain `ui/text/glyph_artifact.rs` as an
  alias/forwarder.
- Do not import `crate::ui::text::text_style`, preserve `super::text_style`, or create another UI
  adapter dependency from the moved text owner.
- Do not leave the `TextStyle` conversion impl in graphics, duplicate it, or replace it with an
  incomplete field-by-field artifact-only helper.
- Do not move concrete session/font-database logic into `core::framework::text` or
  `zircon_runtime_interface`.
- Do not add a parallel graphics-owned artifact, conversion DTO, clone-on-submit path, or fallback
  re-shaping pass.
- Do not absorb the current foreign UI/Text/Graphics worktree blobs into a Frameworks05 commit.
- Do not weaken the full-line glyph identity, visual-order, ellipsis, or font-generation recovery
  regressions.

## 修复结果与回传

Current state: `text02_hard_cut_implemented_secondary_review_complete_managed_acceptance_pending`. The successor
has physically moved the artifact to `text/glyph_artifact.rs`, moved the unique style conversion to
`text/ui_style.rs`, deleted both retired UI owner files, and migrated UI/graphics consumers. The
current read-only runtime-domain audit reports **2,739 production references / 72 domain edges**,
`graphics -> ui = 0` and `text -> ui = 0`;
the four focused filters statically select **3 / 1 / 2 / 1** tests. The direct-artifact forward
repair has now passed an independent re-review with **P0 0 / P1 0 / P2 0**. Canonical managed
focused/full Runtime gates, fixed return, and
Frameworks05 M3/M4 acceptance remain pending, so this artifact stays `open` and does not claim
accepted validation.

## 2026-08-02 Forward Repair Status

- Status: `open / resolving_failure / implementation_complete / secondary_review_complete /
  managed_validation_pending`.
- Completed: the SDF renderer now consumes the text-owned artifact line directly. The former
  `TextGlyph -> ScreenSpaceUiShapedGlyph` projection and its font-refresh counterpart are removed
  for artifact-backed layout. Horizontal and VerticalRl SDF paths consume the exact visual-order
  `TextGlyph` sequence, including its resolved advances, offsets, rotation, source ranges, and
  font handles.
- Completed: font generation refresh replaces the renderer wrapper's current line with a newly
  rebuilt text-owned line rather than rebuilding a graphics DTO. The regression covers direct
  identity, resolved advances, visual source ordering, and the generation-refresh branch.
- Completed: SDF atlas and CPU-run cache snapshots now include an O(1) text-owned artifact-line
  identity (artifact/active-line addresses, line index, and font generation) plus writing mode.
  They invalidate on a refreshed glyph line, glyph identity/fallback change, or vertical-layout
  change without cloning or re-shaping the glyph sequence. Regressions cover atlas replacement
  glyph IDs, same-Arc font-refresh invalidation, horizontal-to-vertical cache invalidation, CPU
  snapshot rejection, and a direct refreshed-line vertical rotated-ligature vertex build.
- Completed: stale test-only `crate::ui::text` artifact consumers now use `crate::text`. Artifact
  SDF vertex code is folder-backed in `sdf_render/artifact_vertices.rs`; the generic vertices owner
  is 789 lines and the render coordinator root is 785 lines after the identity leaf split.
- Static evidence: `rustfmt --edition 2024 --check` and scoped `git diff --check` pass. The
  pre-cache/split production dependency audit reported **2,583 production references / 74 edges**,
  including `graphics -> text = 63`, `ui -> text = 48`, and no `graphics -> ui` or `text -> ui`
  edge. The fresh post-cache/split audit completed on 2026-08-15 reports **2,739 production
  references / 72 edges**, with `ui|graphics -> text = 153` and `text -> ui|graphics = 0`.
  The current cache/split source scan finds no retired `ui::text` artifact consumer or
  `screen_space_glyphs` projection.
- Independent re-review: **P0 0 / P1 0 / P2 0**. It specifically confirmed that a refreshed line
  on the same artifact `Arc` rebuilds atlas keys and CPU snapshots, that the VerticalRl Cw90 path
  emits SDF vertices from the refreshed text-owned line, and that the render root only wires its
  focused artifact and route-identity children.
- Pending: coordinator-managed Cargo/WGPU gates and the required real framebuffer PNG under
  `docs/tests/runtime/text`. No screenshot was generated and no pending validation is represented
  as acceptance.

## 2026-08-11 Plain no-artifact forward repair

- The Plain render-planning hard cut previously returned after the glyph-artifact helper even when
  that helper returned no batches. A normal resolved ASCII layout without an artifact therefore
  produced neither native nor SDF text, and an existing regression incorrectly required the blank
  result. This violated the MVP requirement that ordinary resolved text remains renderable.
- The fallback is deliberately narrow. Only a line whose exact source slice equals its visual text
  and whose run projection is either empty or one `Plain` run matching the line text, source range,
  visual range, and direction may be rebuilt without an artifact. The empty-run form preserves the
  minimal public resolved-layout DTO without inventing style segmentation. The whole command uses
  all-or-nothing collection, so one Bidi,
  virtual, ellipsized, split-run, or otherwise non-isomorphic line still keeps the prior fail-closed
  behavior. Artifact-backed and synthetic-visual batches remain the first-choice paths.
- The product-planner regression now requires empty-run and single-Plain-run source-isomorphic
  wrapped lines to retain their exact frames and layout advances as native batches. The existing
  multi-run visual-Bidi negative
  regression continues to require zero native/SDF batches without an artifact. Repository-edition
  leaf rustfmt and scoped whitespace checks pass; source-level second review found no remaining
  actionable P0/P1/P2 in this repair.
- Status remains `open / resolving_failure / implementation_complete / secondary_review_complete /
  managed_validation_pending`. No Cargo/WGPU or PNG was run or generated by this repair; the real
  framebuffer proof remains coordinator-owned and must be written only under `docs/tests/runtime/text`.

## 2026-08-15 retained-host artifact consumption reinvestigation

- Status remains `open / resolving_failure / structural-research-complete / implementation-not-started /
  managed-validation-pending`. This is the same artifact-owner failure, not a second handoff: the
  existing acceptance rule already forbids re-shaping a visual run after layout has produced its
  text-owned glyph artifact.
- Current-source audit found that
  `zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs::runtime_text_lines`
  calls `ui::surface::layout_text`, then ignores the layout's `rich_text_artifact` and invokes
  `shape_text_line` once for every resolved visual line. The helper creates a new
  `SharedTextLayoutSession` on each call. Its cache retains an `Arc<ShapedGlyphRun>`, so
  `Arc::unwrap_or_clone` necessarily clones the shaped run before the temporary session drops.
  Wrapped cache misses therefore add one full layout shape plus one full visual-line shape and one
  shaped-glyph copy per line.
- The same retained-host leaf recomputes visual grapheme advances with a glyph-by-grapheme full scan
  and a per-glyph overlap `Vec`. Its local worst case is `O(glyphs * graphemes)` plus temporary
  allocation, independent of Text03's bounded `partition_point` projection. This is a real upper
  consumer regression, not evidence that Text03 should regain a second projection owner.
- The layout-time `ResolvedTextGlyphArtifactLine` already holds exact visual-order `TextGlyph`
  values with glyph ID, source/visual range, advance, position, offset, font face/instance handle,
  rotation, bidi level, and raster visibility. The retained host must consume this immutable line
  through a text-owned facade after exact layout and font-generation matching. It must not synthesize
  a `UiShapedText` glyph hash, create an editor cache, or call `shape_text_line` as a fallback for
  artifact-backed plain text.
- A safe hard cut also needs the missing real-font bridge. The editor's `HostTextFontFace` currently
  resolves an independent fontdb/fontdue snapshot by family, while artifact glyph IDs are valid only
  for their text-owned `TextFontFaceHandle` generation. The bridge must resolve the artifact's
  immutable runtime face bytes and collection index at the matching generation before the retained
  raster path consumes glyph IDs. It must fail closed to the existing non-artifact draw path when
  a line has no artifact, is synthetic/ellipsized, has a stale generation, or its face cannot be
  resolved; it must never reinterpret a glyph ID against an unrelated host font.
- Reference check: Unreal Slate returns an immutable `FShapedGlyphSequencePtr` from its font cache
  and carries it through the rendering pipeline specifically to avoid copying. Its HarfBuzz
  finalization also records character/grapheme membership on the shaped entry. Zircon's equivalent
  is the existing `ResolvedTextGlyphArtifactLine`, not another editor-local shaped-run cache.
- Required implementation order: (1) make the text-owned artifact line shareable without cloning
  its glyph storage and expose only an exact-match runtime query facade; (2) add the generation-safe
  runtime-face snapshot bridge required by the retained raster owner; (3) replace the retained-host
  per-line `shape_text_line`, clone, and grapheme overlap projection with direct artifact glyph
  consumption; (4) preserve the current non-artifact/synthetic fail-closed path; (5) add focused
  ownership and behavior regressions before the milestone testing stage.
- Required performance stage after implementation, not before: instrument artifact hit/miss,
  retained-host shape calls, glyph-line clone/allocation count, and artifact projection duration;
  collect p50/p95 for 1/100/1k/10k Latin, CJK, RTL, ligature, and wrapped-label cases. The expected
  artifact-backed result is zero extra text shape calls and zero shaped-glyph-line copies in the
  retained host. Managed Cargo/WGPU and a current-source framebuffer PNG remain separate gates;
  this research added neither runtime measurements nor a screenshot.

### Retained raster bridge detail

- `TextGlyph` is not interchangeable with the retained `ShapedGlyph`: it carries the actual
  `TextFontFaceHandle`, glyph index, position, offset, visual range, and raster flag. The retained
  `RuntimeTextGlyph` and `GlyphRasterKey` currently reduce font identity to the three-value
  `HostTextFontFace` enum, and `runtime_positioned_glyphs(...)` first invokes a one-font
  `fontdue::Layout`. That is exactly the point at which a CJK/Arabic/emoji fallback glyph would be
  reinterpreted against the wrong font. The artifact branch must instead create positioned glyphs
  directly from its exact `TextGlyph` values and carry a per-glyph resolved runtime-font snapshot
  into the raster leaf.
- The bridge cannot be an interface DTO or a private downcast reached by the editor. The opaque
  `UiRichTextArtifactHandle` remains the transport, while `zircon_runtime` needs a narrow,
  non-serializable editor-consumption query: exact `UiResolvedTextLayout`/line equality plus
  shared-font generation must be checked in the text owner, then its result may expose a shared
  glyph slice and the unique actual-face byte/collection-index snapshots. No concrete artifact
  cache is moved into `zircon_runtime_interface`, and no editor-local artifact cache is introduced.
- The snapshot lookup must resolve all handles from one matching shared-font database snapshot;
  `TextFontFaceHandle` generation and the artifact generation must agree before any `FontFaceId`,
  bytes, or collection index is returned. A publish race, missing handle, variable instance that
  the retained raster cannot realize, or any unsupported glyph format rejects the complete
  artifact line and keeps the existing non-artifact rendering path.
- `rasterize_cached_glyph(...)` must receive the resolved per-glyph font identity and key its
  cache by immutable runtime-face identity/generation rather than `HostTextFontFace`. Current
  `SwashContent::Color` is reduced to alpha in the retained raster leaf, so artifact-backed color
  glyphs cannot silently claim pixel parity: they stay on the pre-existing fallback until that leaf
  owns an RGBA compositing path. This is deliberately a correctness boundary, not a request to
  duplicate Text04's atlas/color implementation in the editor layout owner.
- Implementation follows the resulting dependency order: text-owned exact artifact view and
  snapshot batch -> retained font/raster snapshot representation and cache key -> direct artifact
  glyph positioning -> retained draw selection and focused regressions. Only after those contracts
  exist can the per-line shape/clone/overlap code be deleted and its measured optimization claimed.
- Instrumentation preparation is complete without changing the current fallback behavior:
  `ui/surface/text_shape.rs::shape_text_line` now emits one feature-gated
  `runtime/text.surface:shape_text_line` span around the complete temporary-session call. Its
  profiling-feature regression captures exactly one span and resets the shared recorder. This lets
  the required retained-host experiment measure extra surface shaping separately from Text02
  direct shaping and Text03 projection. Scoped Rust formatting/whitespace checks pass; no Cargo
   run, p50/p95 data, framebuffer, or PNG is claimed. The no-Cargo convention audits also report
   guard-rule violations `0` and enforced-member exemption violations `0`.

### Unreal Slate contract confirmation (2026-08-15)

- The local Unreal reference confirms the lifetime and identity boundary required above rather than
  merely suggesting a clone reduction: `SlateCore/Public/Fonts/ShapedTextFwd.h` defines
  `FShapedGlyphSequencePtr` as a shared pointer to a **const** shaped sequence and documents that
  the rendering pipeline uses it to avoid copying. `FSlateFontCache::ShapeBidirectionalText` and
  `ShapeUnidirectionalText` return that shared immutable sequence directly.
- `FShapedGlyphEntry` records the actual font-face data and glyph index together with source index,
  advance, offset, character/grapheme membership, direction, and visibility. Its atlas lookup key
  is formed from the glyph's own font face and glyph index; it is not keyed by a requested family
  or a single retained-host font. This confirms that a correct Zircon artifact branch must preserve
  per-glyph runtime face identity through rasterization, including fallback runs.
- Therefore the current retained-host conversion to `HostTextFontFace` cannot be retained behind a
  cache optimization. The first writable implementation remains the text-owned immutable line view
  plus same-generation face snapshot batch; the retained raster leaf must accept that identity
  before any artifact-backed draw path is enabled. No runtime behavior, benchmark result, Cargo
  result, or framebuffer evidence is implied by this reference-only confirmation.
- Runtime snapshot audit adds one required distinction: `TextGlyph.font_instance` may legitimately
  carry a default instance even for an ordinary static face. The bridge must resolve its
  face/instance pair against the same `FontDatabase` snapshot and inspect the effective variation
  coordinates. It may reject a non-default variation that the retained raster cannot realize, but
  it must not reject every non-null instance handle and thereby disable the basic artifact path.

### Structural landing rule (2026-08-15)

- The artifact-backed projection lives in a dedicated `draw/layout/artifact.rs` child with a
  narrow result type. The independent runtime-line resolution responsibility now lives in
  `draw/layout/runtime_lines.rs`, leaving `paint_text/draw/layout.rs` at 703 lines as the
  orchestrator that selects exact artifact consumption or its existing fallback. Matching behavior
  cases remain under the folder-backed `draw/layout/tests/{artifact,runtime_lines}.rs` children
  rather than extending the 795-line test root.
- This placement preserves the existing font/text -> UI surface -> editor direction. It does not
  promote `fontdb`, runtime artifact payloads, or editor raster cache state into
  `zircon_runtime_interface`, and it prevents this recovery from reopening the code-structure
  finding while it removes the shaping/clone hotspot.

### Surface artifact-view foundation (2026-08-15)

- The first reusable bridge layer is now implemented in
  `zircon_runtime/src/ui/surface/text_artifact.rs`. The public
  `resolved_text_glyph_artifact_line(layout, index)` facade retains the text-owned artifact `Arc`
  and exposes its canonical visual-order `TextGlyph` slice without cloning glyph storage. It
  accepts only an exact `UiResolvedTextLine` match at the shared font generation, and explicitly
  rejects synthetic/ellipsized visual lines.
- `UiResolvedTextGlyphArtifactLine::glyphs()` and `layout_line()` recheck the captured generation
  before returning borrowed data. A publication after view creation therefore becomes `None`,
  preserving the existing fail-closed fallback contract instead of allowing glyph IDs to cross a
  font-database generation. Folder-backed behavior coverage verifies shared-slice pointer identity,
  mismatched DTO rejection, synthetic-line rejection, stale artifact rejection, and stale-view
  rejection.
- This is deliberately only bridge layer one. The surface facade does not expose backend font data,
  does not replace the graphics-private refresh type yet, and is not consumed by retained raster
  until the same-generation per-face snapshot/cache-key and direct positioning layers exist. Rust
  formatting and scoped diff checks pass; no Cargo test, p50/p95 sample, WGPU run, or PNG is
  claimed while UI12 retains the validation lane. No change is made under `target`.

### Surface face-snapshot foundation (2026-08-15)

- Bridge layer two is now implemented by the layout-scoped
  `UiResolvedTextGlyphArtifactFaceSnapshot`: the retained consumer captures it once from a
  current artifact line, then calls `raster_faces_from_snapshot()` for each visual line. The
  convenience `raster_faces()` remains single-line only. This moves the `FontDatabase` clone from
  `O(visual_lines)` to `O(1)` per resolved layout without allowing the snapshot to escape the
  runtime-owned bridge.
- `raster_faces_from_snapshot()`
  gathers only raster glyphs, de-duplicates their exact `(TextFontFaceHandle, instance handle)`
  pairs with one `HashMap::entry` operation per glyph, and resolves the batch through one matching
  shared `FontDatabase` snapshot. Its resulting `O(glyphs + unique_faces)` table offers O(1)
  lookup by the original glyph pair.
- Each returned runtime face retains an `Arc<[u8]>` source allocation, TTC collection index, stable
  source identity, generation, and effective instance variations. Missing handles, an instance
  mapped to a different base face, database/registry publication races, or a final generation
  mismatch reject the complete table. Coverage additionally asserts that repeated byte access is
  `Arc::ptr_eq`, so this bridge does not create a per-glyph font-byte copy.
- At this foundation point the retained renderer was intentionally not switched: source review
  confirmed that its Swash `Content::Color` path reduced RGBA color glyphs to alpha and its
  fontdue fallback took a `HostTextFontSnapshot`. The subsequent MVP branch below adds the
  per-face cache key and RGBA-capable draw path, while preserving complete-layout fallback.
  Focused behavior tests are present but unrun; only rustfmt/scoped diff and no-Cargo convention
  checks (`0` guard violations, `0` enforced-member exemption violations) are current evidence.

### Retained exact-artifact MVP path (2026-08-15)

- The retained-host path now consumes the surface artifact when, and only when, every visual line
  is an exact current artifact line. The runtime owner resolves one de-duplicated face table across
  the complete immutable artifact, capturing one `UiTextGlyphArtifactFaceSnapshot` only when that
  table has raster glyphs, and constructs one indexed raster font entry per distinct runtime
  `(face, instance)` pair. The output glyph vector contains only the original runtime glyph IDs
  and pen positions; it does not clone `TextGlyph` storage or call `shape_text_line` again on an
  artifact-backed line.
- The branch is atomic: synthetic/ellipsized lines, a stale publication, a missing raster face,
  an unsupported rotation, an out-of-range retained raster glyph ID, malformed coordinates, or a
  non-default variation coordinate rejects the complete layout back to the existing host path.
  `fontdue` cannot realize the runtime variation coordinates, so the latter is a deliberate
  correctness fallback, not an attempt to reinterpret a variable-font glyph with default axes.
- A new bounded 64-entry retained runtime-face cache keeps the single `Arc<[u8]>` supplied by the
  runtime and is keyed by source identity, font generation, exact face/instance handles, and TTC
  collection index. The retained layout cache also now includes the current runtime font
  generation, preventing a cached artifact-face table from crossing a font publication.
- The retained raster format now preserves Swash `Content::Color` as RGBA and blends its source
  colors instead of reducing it to an alpha mask and applying the text tint. The run alpha still
  multiplies the source alpha, so a translucent text command preserves color-glyph RGB while
  retaining its requested opacity. Focused unit coverage specifies the runtime pen/offset
  conversion, source-identity cache discrimination, RGBA format preservation, native RGBA
  sampling, alpha-weighted RGBA downsampling, source-color blending, and run-opacity composition.
  The direct preflight/projection additionally emits one feature-gated
  `editor/host_painter:runtime_artifact_glyph_projection` span, complementary to the existing
  fallback `shape_text_line` span. Rustfmt and scoped diff checks pass; Cargo, profiling samples,
  WGPU execution, and framebuffer/PNG evidence remain pending the UI12 validation-lane release.
  No artifact was written under `target`.
- Profiling preparation now also records whole-layout artifact projection hit/miss, projected
  glyphs, artifact candidate lines/glyphs, retained temporary surface-shape lines, and copied
  shaped-glyph lines/glyphs. The direct-artifact profiling fixture requires a non-empty exact
  runtime raster-font set, one projection span, and zero `shape_text_line`/copied-glyph counters.
  These are feature-gated capture counters only; the open handoff still requires managed 1/100/
  1k/10k p50/p95 samples and current-source Cargo/WGPU evidence before it can be returned.
- Structure follow-through: the near-budget retained layout test root now delegates runtime
  resolved-line behavior (single-line advances, ellipsis, and word-wrap) to
  `draw/layout/tests/runtime_lines.rs`. The root is **792 lines** and the child is **109 lines**;
  the moved tests retain their prior runtime-layout assertions, while the root no longer imports
  the child-only surface layout/shaping DTOs. Scoped rustfmt, whitespace, diff, and convention
  checks pass. This is implementation evidence only; the failure remains open pending managed
  Cargo, profiling samples, and the real WGPU framebuffer proof.
- The pending performance stage now has a single ignored, profiling-feature harness in
  `draw/layout/tests/artifact.rs`. It takes 31 forced-cache-miss samples for `1/100/1k/10k`
  semantic units of Latin, CJK, RTL, ligature, and wrapped-label text; each sample must retain
  exact artifact raster faces. It reports p50/p95 only after asserting one artifact-projection hit
  per sample and zero retained `shape_text_line` calls or copied shaped glyphs. The frame position
  is monotonic per sample because it is part of the current host-layout cache key, so the timing
  cannot be reported as a cache-hit result. This prepares the required bottleneck evidence; no
  profiling values, power data, Cargo result, or framebuffer image have been produced yet.
- Architecture boundary: the retained artifact bridge now follows Unreal Slate's immutable shared
  shaped-sequence rule, including exact per-glyph face identity through rasterization. It does not
  claim to solve the separate editor `PaintTextLayout` cache design: that cache still owns a text
  `String`, absolute `rect.x/y`, a process-global mutex, and a whole-map clear at 2,048 entries.
  Runtime09 owns the required content-layout versus screen-placement split, indexed LRU/byte
  budget, and scroll/RSS evidence. The forced-miss Text02 harness deliberately measures the
  bridge before that cache work and must not be used as evidence for or against the cache redesign.

### Artifact-wide face resolution review (2026-08-15)

- Static hot-path review found a structural multiplicative cost in the prior retained direct path:
  `positioned_artifact_glyphs` called the line-level face resolver once for every visual line,
  while `resolve_font_handle_batch` acquires a font-handle registry snapshot for every batch. A
  wrapped layout therefore acquired `O(visual_lines)` registry snapshots and repeated face-pair
  deduplication, even when every line used the same shaped face.
- The repair retains the runtime-owned immutable artifact as the layout-level resource owner. The
  retained consumer first proves that every line shares that artifact, then asks the owner for one
  de-duplicated exact `(face, instance)` table across all artifact lines. The owner captures one
  font database snapshot only for a non-empty table. The steady-state resolution work is
  `O(total artifact glyphs + unique_face_instances)` with exactly one registry batch/snapshot when
  an exact artifact layout has raster faces (and zero database/registry snapshots for a face-free
  layout); mixed artifacts, a missing artifact line, stale generation, unsupported rotation, or a
  missing face still reject the whole layout to the existing fallback.
- Reference review: Unreal Slate's `FShapedGlyphSequence` is immutable/shared, owns its
  `GlyphFontFaces` set with all glyph entries, and is submitted as one shaped payload; Fyrox's
  `FormattedText` likewise keeps final lines and glyph draw data under one formatted-text owner.
  Zircon deliberately keeps the existing backend-neutral `UiResolvedTextLayout` DTO and opaque
  runtime artifact rather than exposing backend types or adding an editor font owner.
- New focused coverage checks that two visual lines from one artifact use one real registry batch
  and snapshot acquisition. A profiling-feature retained-host fixture requires one
  `artifact_face_snapshot` plus one `artifact_raster_face_resolution` span for a genuinely
  multi-line wrap, while the existing 31-sample scale harness requires exactly one of each span
  per rasterized layout. A face-free artifact regression separately requires zero font-handle
  batches and registry snapshots. These are structural count assertions, not p50/p95, power,
  Cargo, or WGPU evidence; all runtime measurements and the required product framebuffer remain
  pending UI12's validation-lane release.

### Face-free snapshot-bypass plan (2026-08-15)

- A further static trace of the complete artifact path found one remaining empty-work cost: the
  prior retained consumer called `face_snapshot()` before it knew whether the immutable artifact
  contained a raster glyph. A whitespace- or virtual-glyph-only layout therefore avoided
  `resolve_font_handle_batch` but still cloned the shared `FontDatabase`. This was not a registry
  correctness issue and did not justify an editor-local cache; it was a text-owner sequencing
  issue.
- The repair keeps pair discovery and ownership in the runtime artifact. It scans the exact
  artifact once into the existing ordered `(face, instance)` table. An empty table returns the
  existing empty raster-face result without a database snapshot; a non-empty table captures
  exactly one matching snapshot and resolves that same table. Generation checks, complete-layout
  fail-closed behavior, and O(total artifact glyphs + unique face instances) work remain unchanged.
- The focused regression now requires a face-free artifact to produce an empty table with zero
  font-handle batches/snapshots and zero `artifact_face_snapshot` spans. The existing multi-line
  raster fixture remains the guard that a non-empty layout has one snapshot and one resolution
  span. Managed 31-sample p50/p95, power, Cargo, and WGPU framebuffer evidence remain separate
  pending gates and are not claimed here.

### Layout-level face-set architecture review (2026-08-15)

- Reference review distinguishes the required owner from a premature cache: Unreal Slate builds
  the final `FShapedGlyphSequence::GlyphFontFaces` set once from `GlyphsToRender` in
  `SlateCore/Private/Fonts/FontCache.cpp`, retains it with the immutable sequence, and invalidates
  it when a weak face expires. Fyrox similarly keeps its final `FormattedText` lines and glyphs
  under one formatted-text owner. Zircon's equivalent owner is the crate-private
  `ResolvedTextGlyphArtifact`; the public `UiRichTextArtifactHandle` remains opaque, and the
  retained host continues to receive only the runtime-owned face-query result.
- Current MVP behavior is correct but intentionally not yet the final Unreal-shaped storage form:
  artifact construction creates the immutable final glyph lines once, while
  `artifact_raster_faces()` scans those exact artifact lines once per complete retained-layout
  consumption to build the ordered, de-duplicated `(face, instance)` table. This replaces the
  former per-visual-line work and preserves the zero-snapshot face-free path, but it still has
  `O(scanned_glyphs + unique_face_instances)` work and temporary `Vec`/`HashMap` allocation on a
  consuming layout. Static review alone cannot establish that this remaining linear scan is a
  material bottleneck.
- Profiling preparation therefore records feature-gated runtime counters
  `artifact_raster_face_scanned_glyph_count`,
  `artifact_raster_face_candidate_glyph_count`, and
  `artifact_raster_face_unique_pair_count` beside the existing
  `artifact_raster_face_resolution` span. Focused profiling regressions require their exact
  face-free and raster values. The managed 31-sample 1/100/1k/10k Latin, CJK, RTL, ligature, and
  wrapped-label harness must correlate those counts with p50/p95 before any structural cache is
  accepted. It must also retain the existing zero extra `shape_text_line` and zero copied shaped
  glyph assertions, then collect comparable process/power evidence under the same adapter,
  driver, resolution, workload, and thermal conditions before making any cross-engine efficiency
  claim.
- Decision gate: do not add a lazy `OnceLock`, editor-local table, interface DTO, or duplicate
  cache. Only if released managed profiling demonstrates that the resolution span scales with
  scanned glyphs while unique pairs remain stable and that scan cost materially dominates the
  direct-artifact path may Text02 move the ordered face-pair set into
  `ResolvedTextGlyphArtifact` at construction. That follow-up must use a construction helper for
  every synthetic test artifact, preserve current generation invalidation and complete-layout
  fail-closed behavior, prove the stored set equals the final visual glyph sequence, and repeat
  the Cargo, profiling, WGPU framebuffer, and power gates. No such measurement or optimization is
  claimed by this record.

### Artifact-build shaped-cache diagnostic plan (2026-08-15)

- Static call-chain review found a distinct candidate cost before any face resolution: line-width
  measurement shapes `line.text` through `measured_grapheme_widths_with_provider` with the local
  `0..line_len` source range, but `shape_line_for_artifact` later requests the same final line with
  its absolute `UiResolvedTextLine::source_range`. The shaped-cache key intentionally includes the
  source range because glyph source ranges are observable. Therefore a nonzero-start line cannot
  reuse the local metric entry by ignoring that key field; doing so would corrupt source identity.
- The immediate work is diagnostic only. The text-owned artifact build records its final artifact
  line count plus the delta of the shared session's shaped-cache hit and miss counts, alongside a
  `text.artifact:build_resolved_text_glyph_artifact` span. Focused profiling coverage establishes
  the counter contract; released managed 1/100/1k/10k Latin, CJK, RTL, ligature, and wrap samples
  must then correlate the delta with p50/p95 before a structural change is considered.
- Decision gate: do not re-key the existing cache, hide source ranges, or add an editor cache. If
  the released samples prove the artifact delta materially dominates the direct layout path, the
  only candidate repair is a runtime Text layout result that carries the final immutable shaped
  sequence with its absolute source origin into `ResolvedTextGlyphArtifact`. That design must keep
  line-break/ellipsis synthetic fallback, Bidi visual ordering, ligature cluster mapping, font
  generation invalidation, and the opaque interface handle intact. No benchmark, power, Cargo, or
  framebuffer evidence is claimed by this diagnostic plan.

### Managed validation attempt status (2026-08-16)

- Completed, static only: the scoped convention guard reports `0` violations. The retained-host
  artifact projection harness remains the declared 620-sample matrix: 31 forced-cache-miss samples
  for each of 5 semantic workloads at 1/100/1k/10k units. Each result is gated on an artifact hit,
  zero retained `shape_text_line` calls, and zero copied shaped glyphs. This is the measurement
  plan and structural counter contract, not a performance result.
- Attempted, not completed: the managed ignored product WGPU framebuffer test
  `export_runtime_multilingual_text_product_framebuffer_png` was launched. Cargo stopped before
  WGPU initialization on cross-owner `E0277` in
  `zircon_runtime/src/core/runtime/handle/activation/batch.rs:221`: the code iterates
  `&Arc<[RegistryName]>`. That file is leased to
  `runtime-core-lifecycle-m0-veto-atomicity-20260815`; the blocker was recorded through the
  coordinator without creating another failure handoff.
- Current acceptance state: no WGPU frame ran, no profiling sample or power value was collected,
  and `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260801.png` was
  not created. No historical image or text-only substitute is accepted. Resume only after the
  Runtime Core compile failure is closed, then run the same managed product WGPU test, inspect its
  newly generated framebuffer image, and run the declared scale harness before considering any
  additional artifact caching or other structural optimization.
