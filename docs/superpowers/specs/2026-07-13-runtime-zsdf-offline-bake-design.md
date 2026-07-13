---
related_code:
  - zircon_runtime/src/graphics/text/sdf/offline
  - zircon_runtime/src/font_sdf_build_tool
  - zircon_runtime/src/bin/zircon_font_sdf_bake
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/offline_source.rs
  - tools/zircon_build_font_sdf.py
plan_sources:
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
status: accepted_for_implementation
---

# Runtime `.zsdf` offline bake design

## Outcome

Text05 SM-M3 adds one deterministic offline distance-field asset path without creating a second glyph generator. Build tooling calls the renderer-neutral `graphics/text/sdf` fdsm owner, writes one `.zsdf` container, and runtime font bake checks that container before dynamic generation. A usable hit supplies exact metrics and glyph pixels to the existing unified atlas copy/upload path; a missing, stale, corrupt, or uncovered glyph keeps the existing dynamic SDF/MSDF/MTSDF fallback.

## Ownership

- `graphics/text/sdf/offline/` owns the versioned binary contract, strict decode validation, request/header matching, glyph lookup, deterministic path construction, and page/rect extraction. It does not know WGPU, UI batches, project managers, or filesystem policy.
- `font_sdf_build_tool/` is a feature-gated tooling API inside `zircon_runtime`. It enumerates a requested codepoint subset or the font cmap, calls the existing shared generator, packs glyphs through the shared shelf allocator, and returns an offline artifact/report. No Python or binary-side generator is allowed.
- `bin/zircon_font_sdf_bake/` owns CLI parsing and filesystem input/output only.
- `tools/zircon_build_font_sdf.py` owns Python build orchestration and command validation. `tools/zircon_build.py` only exposes and dispatches the target; target-specific arguments and behavior stay in the child module so the 992-line root does not become a font tool owner.
- `scene_renderer/ui/sdf_font_bake/offline_source.rs` owns project font identity discovery and the runtime lookup adapter. `sdf_font_bake.rs` retains only the precedence decision: memory cache → valid `.zsdf` glyph → dynamic generator → typed failure.

## `.zsdf` version 1 contract

One little-endian binary contains header, fixed-size page records, fixed-size glyph records, and tightly packed page bytes. Records and glyph generation order are stable-sorted; reserved bytes must be zero; all lengths, offsets, formats, rectangles, glyph ids, and finite metrics are validated before a blob is admitted.

The header contains at least:

- magic and format version;
- canonical font asset GUID;
- face index;
- variation-instance BLAKE3 hash;
- standalone font-source BLAKE3 hash;
- `SdfMode`, bake em size, and pixel range/spread;
- atlas page dimensions/count, glyph count, and section lengths.

MSDF and MTSDF both use RGBA page storage but remain distinct artifact identities. MTSDF alpha is preserved. The artifact embeds page bytes instead of depending on a PNG decoder; this is the plan's "atlas image + metadata" payload in a single atomic file.

## Runtime identity and fallback

For project `res://` font manifests, the runtime reads the authoritative adjacent `.zmeta` GUID and uses the project cache root. The resolved face bytes provide the source hash. The current text path has no variable-axis selection in its raster key, so it uses the shared hash of an empty normalized axis list; any future axis-aware key must provide its real hash rather than aliasing the default.

A blob is usable only when version, GUID, face index, variation hash, font source hash, mode, bake size, and spread all match. Glyph lookup uses actual shaped glyph id; cmap scalar is retained as artifact metadata but is not the cache authority. Fallback faces naturally reject the primary artifact by source hash and continue through dynamic generation.

An accepted artifact is cached in memory per identity. If runtime glyph/page residency later drops a glyph, lookup still checks the accepted artifact before dynamic generation; rejected or missing files never become a success path.

## Acceptance

- `text_sdf_offline_bake_roundtrip`: encode → decode → encode is byte-identical and preserves metrics/page pixels.
- deterministic bake: same font, codepoints, parameters, identity, and page size produce identical bytes.
- strict stale/corrupt rejection: version, GUID, face, variation, source hash, mode/size/range, bounds, offsets, and checksum failures are typed.
- `text_sdf_offline_glyph_hits_skip_dynamic_gen`: renderer bake report records an offline hit and zero dynamic generations for a covered glyph; an uncovered glyph records a dynamic generation.
- Python command/target tests prove subset and all-cmap routing without embedding bake policy in `zircon_build.py`.
- Current-source Windows build/check and an exact CLI artifact roundtrip pass; generated test artifacts stay in managed targets or temporary test roots, never under repository `target` or `docs/tests/runtime/text`.
