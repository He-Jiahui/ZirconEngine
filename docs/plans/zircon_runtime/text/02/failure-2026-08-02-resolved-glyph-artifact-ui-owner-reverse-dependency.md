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
pre-cache/split read-only audit reports **2,583 references / 74 edges**, `graphics -> ui = 0` and
`text -> ui = 0`;
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
  pre-cache/split production dependency audit reports **2,583 production references / 74 edges**,
  including `graphics -> text = 63`, `ui -> text = 48`, and no `graphics -> ui` or `text -> ui`
  edge. The current cache/split source scan finds no retired `ui::text` artifact consumer or
  `screen_space_glyphs` projection. A fresh dependency audit remains part of managed validation.
- Independent re-review: **P0 0 / P1 0 / P2 0**. It specifically confirmed that a refreshed line
  on the same artifact `Arc` rebuilds atlas keys and CPU snapshots, that the VerticalRl Cw90 path
  emits SDF vertices from the refreshed text-owned line, and that the render root only wires its
  focused artifact and route-identity children.
- Pending: coordinator-managed Cargo/WGPU gates, the fresh post-cache/split dependency audit, and
  the required real framebuffer PNG under `docs/tests/runtime/text`. No screenshot was generated
  and no pending validation is represented as acceptance.
