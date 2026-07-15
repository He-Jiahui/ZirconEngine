---
related_code:
  - zircon_runtime/src/text/sdf/offline
  - zircon_runtime/src/font_sdf_build_tool
  - zircon_runtime/src/bin/zircon_font_sdf_bake
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/offline_source.rs
  - tools/zircon_build_font_sdf.py
plan_sources:
  - docs/superpowers/specs/2026-07-13-runtime-zsdf-offline-bake-design.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
status: complete
---

# Runtime `.zsdf` offline bake implementation plan

## SM3-M1: deterministic shared artifact contract

- [x] Add red tests for roundtrip byte identity, sorted glyph lookup, SDF/RGBA page extraction, stale identity rejection, corrupt length/rect rejection, and deterministic store path.
- [x] Implement folder-backed `graphics/text/sdf/offline/{artifact,codec,error,identity,path}.rs`; keep the root declaration-only.
- [x] Include format version, asset GUID, face index, variation hash, source hash, mode, bake parameters, page metadata, glyph metadata, and page bytes.
- [x] Validate all arithmetic with checked operations and reject non-finite metrics, duplicate glyph ids, non-zero reserved bytes, trailing bytes, and checksum mismatch.
- [x] Record one SM3-M1 output row and update module docs.

## SM3-M2: build target and CLI

- [x] Add red Python command-contract tests for explicit subset and all-cmap modes, locked/jobs/target-dir forwarding, and missing identity/input rejection.
- [x] Add feature-gated `font_sdf_build_tool/` that calls the existing fdsm generator and shared shelf allocator; do not copy generator math.
- [x] Add folder-backed `zircon_font_sdf_bake` binary with typed CLI errors and atomic `.zsdf` write.
- [x] Add `tools/zircon_build_font_sdf.py`; keep `tools/zircon_build.py` below its current structure budget by moving target-specific parsing/orchestration into the child.
- [x] Support explicit codepoint/range subsets and full Unicode cmap enumeration, stable deduplication by glyph id, selectable SDF/MSDF/MTSDF, page size, bake em, and pixel range.
- [x] Record one SM3-M2 output row and update build-tool documentation.

## SM3-M3: runtime prefill and dynamic fallback

- [x] Add red renderer tests for covered-glyph offline hit/zero dynamic generation, uncovered-glyph dynamic fallback, stale source/variation rejection, and artifact reuse after glyph-cache eviction.
- [x] Extend loaded font manifest identity with authoritative project `.zmeta` GUID without adding a public UI DTO or compatibility field.
- [x] Add `sdf_font_bake/offline_source.rs` to resolve/cache artifacts and return `RawBakedGlyph` data for the actual face/glyph id.
- [x] Keep precedence memory cache → valid artifact → dynamic generator; expose offline/dynamic counts only through existing bake diagnostics.
- [x] Copy artifact pixels through the existing unified R8/RGBA atlas pages and existing upload commands; add no second GPU atlas or renderer pass.
- [x] Record one SM3-M3 output row and update text atlas/SDF docs.

## SM3-T: acceptance

- [x] Run scoped Python/Rust formatting, diff, conflict-marker, trailing-whitespace, legacy-symbol, and file-budget checks.
- [x] Run Python exact tests and artifact core/build-tool exact tests on Windows.
- [x] Build and run the CLI twice against FiraSans for SDF, MSDF, and MTSDF subsets; prove byte-identical repeats and decode/readback.
- [x] Run a managed Windows `cargo check -p zircon_runtime --lib --no-default-features --features target-client --locked --jobs 1`.
- [x] Run renderer offline-hit/dynamic-fallback exact tests; if unrelated test-only code prevents the lib-test binary, retain the external blocker and use a current-source non-test build plus dedicated integration/CLI evidence without claiming that exact gate passed.
- [x] Run plan-output/failure audits and target hygiene; generated `.zsdf` evidence must not be left in repository `target` or `docs/tests/runtime/text`.
- [x] Update Text05 status/output records. Promote SM-M3 only when artifact, build target, runtime prefill, fallback, and hygiene gates all have direct evidence.

## Status

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| SM3-M1 | design and execution contract | complete | 2026-07-13 | Design fixes single generator owner, deterministic embedded-page format, strict identity matching, and runtime fallback order. |
| SM3-M1 | deterministic embedded-page artifact | passed | 2026-07-13 | Current-source `text_sdf_offline` exact filter passed 6/6, including byte-stable roundtrip, sorted glyph extraction, stale/corrupt rejection, invalid rect/trailing-byte rejection, and versioned path. |
| SM3-M2 | build target, CLI, and deterministic bake | passed | 2026-07-13 | Python font-SDF contracts 5/5 plus existing build-tool regressions 45/45; CLI range red/green 1/1; feature-gated inspection integration 2/2. Fira subset baked twice per SDF/MSDF/MTSDF with byte-identical SHA256 `DD527C14...5902`, `FEBF3620...B132`, and `EF8D1639...A55A`; temporary artifacts removed. |
| SM3-M3 | runtime prefill and dynamic fallback | passed | 2026-07-13 | Renderer exact test passes offline hit/zero dynamic generation, uncovered glyph dynamic fallback, stale variation/source rejection, and artifact reuse after glyph-cache eviction; production feature-gated binary check/build pass. |
| SM3-T | final managed check, audits, and hygiene | in_progress | 2026-07-13 | Artifact hygiene currently reports zero `.zsdf` files in repo target, visual-evidence directory, and active Cargo target. Managed target-client check plus plan/failure audits remain before overall SM-M3 promotion. |
| SM3-T | managed final acceptance | passed | 2026-07-13 | Managed Windows target-client check job `1aaddc58899c4d77a23a88005614eb77` exited 0 in 11m44s and coordinator finish/release removed its target. Scoped format/diff/file-budget/whitespace/conflict checks pass. Plan-output and failure audits were run; remaining diagnostics are five output-placement and 33 handoff-schema/link violations in other editor plans, with coordinator failure audit itself exit 0. Repository target, visual-evidence directory, released managed target, and temporary bake root contain zero `.zsdf` files. Overall SM-M3 is promoted. |
